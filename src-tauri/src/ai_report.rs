use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::{AppData, AppSettings};

#[derive(Debug, Clone, Copy)]
enum ReportPeriod {
    Weekly,
    Monthly,
}

impl ReportPeriod {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_lowercase().as_str() {
            "weekly" | "week" => Ok(Self::Weekly),
            "monthly" | "month" => Ok(Self::Monthly),
            _ => Err("报告类型无效，仅支持 weekly 或 monthly".to_string()),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Weekly => "周报",
            Self::Monthly => "月报",
        }
    }

    fn range(self, now: DateTime<Utc>) -> Result<(DateTime<Utc>, DateTime<Utc>, String), String> {
        let end = now;
        match self {
            Self::Weekly => {
                let start = now - Duration::days(7);
                Ok((start, end, "最近 7 天".to_string()))
            }
            Self::Monthly => {
                let month_start_date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                    .ok_or_else(|| "无法计算本月起始日期".to_string())?;
                let month_start = month_start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| "无法计算本月起始时间".to_string())?;
                Ok((
                    DateTime::<Utc>::from_naive_utc_and_offset(month_start, Utc),
                    end,
                    "本月".to_string(),
                ))
            }
        }
    }
}

#[derive(Default)]
struct CategoryStats {
    created: usize,
    completed: usize,
    pending: usize,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    temperature: f32,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    #[serde(default)]
    content: Value,
}

pub async fn generate_ai_report(
    settings: &AppSettings,
    data: &AppData,
    period_raw: &str,
) -> Result<String, String> {
    let period = ReportPeriod::parse(period_raw)?;
    let model = normalize_config_field(settings.ai_model.as_deref())
        .ok_or_else(|| "请先在设置中配置 AI 模型名称".to_string())?;
    let base_url = normalize_config_field(settings.ai_base_url.as_deref())
        .ok_or_else(|| "请先在设置中配置 AI 请求地址".to_string())?;
    let api_key = normalize_config_field(settings.ai_api_key.as_deref())
        .ok_or_else(|| "请先在设置中配置 AI API Key".to_string())?;

    let now = Utc::now();
    let (start, end, window_desc) = period.range(now)?;
    let summary = build_report_summary(data, start, end);
    let user_prompt = build_user_prompt(period, &window_desc, start, end, summary);

    let request = ChatCompletionRequest {
        model,
        temperature: 0.2,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: build_system_prompt(period).to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
    };

    let url = build_chat_completions_url(&base_url);
    let client = Client::new();
    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("请求 AI 接口失败: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("读取 AI 响应失败: {e}"))?;

    if !status.is_success() {
        return Err(format!(
            "AI 接口返回错误 (HTTP {}): {}",
            status.as_u16(),
            truncate_text(&body, 400)
        ));
    }

    let parsed: ChatCompletionResponse =
        serde_json::from_str(&body).map_err(|e| format!("解析 AI 响应失败: {e}"))?;
    let first = parsed
        .choices
        .first()
        .ok_or_else(|| "AI 响应为空，没有可用内容".to_string())?;
    let content = extract_text_content(&first.message.content)
        .ok_or_else(|| "AI 响应格式异常，未找到文本内容".to_string())?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("AI 返回内容为空，请稍后重试".to_string());
    }

    Ok(trimmed.to_string())
}

fn normalize_config_field(raw: Option<&str>) -> Option<String> {
    raw.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_rfc3339_utc(raw: &Option<String>) -> Option<DateTime<Utc>> {
    raw.as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn in_window(ts: Option<DateTime<Utc>>, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
    ts.map(|dt| dt >= start && dt <= end).unwrap_or(false)
}

fn build_report_summary(data: &AppData, start: DateTime<Utc>, end: DateTime<Utc>) -> Value {
    let category_name_by_id: HashMap<&str, &str> = data
        .categories
        .iter()
        .map(|category| (category.id.as_str(), category.name.as_str()))
        .collect();

    let mut created_count = 0usize;
    let mut completed_count = 0usize;
    let mut deleted_count = 0usize;
    let mut carry_over_pending_count = 0usize;
    let mut high_priority_pending_count = 0usize;
    let mut completed_items: Vec<(Option<DateTime<Utc>>, Value)> = Vec::new();
    let mut new_items: Vec<(Option<DateTime<Utc>>, Value)> = Vec::new();
    let mut pending_items: Vec<(Option<DateTime<Utc>>, Value)> = Vec::new();
    let mut category_stats: HashMap<String, CategoryStats> = HashMap::new();

    for todo in &data.todos {
        let created_at = parse_rfc3339_utc(&todo.created_at);
        let completed_at = parse_rfc3339_utc(&todo.completed_at);
        let deleted_at = parse_rfc3339_utc(&todo.deleted_at);
        let is_deleted = todo.is_deleted.unwrap_or(false);
        let is_completed = todo.completed;
        let category_name = category_name_by_id
            .get(todo.category_id.as_str())
            .copied()
            .unwrap_or("未分类")
            .to_string();
        let category_entry = category_stats.entry(category_name.clone()).or_default();

        if in_window(created_at, start, end) {
            created_count += 1;
            category_entry.created += 1;
            new_items.push((
                created_at,
                json!({
                    "title": todo.title,
                    "priority": priority_label(todo.priority),
                    "category": category_name,
                    "createdAt": todo.created_at,
                }),
            ));
        }

        if in_window(completed_at, start, end) {
            completed_count += 1;
            category_entry.completed += 1;
            completed_items.push((
                completed_at,
                json!({
                    "title": todo.title,
                    "priority": priority_label(todo.priority),
                    "category": category_name,
                    "completedAt": todo.completed_at,
                }),
            ));
        }

        if in_window(deleted_at, start, end) {
            deleted_count += 1;
        }

        if !is_completed && !is_deleted {
            category_entry.pending += 1;
            if created_at.map(|t| t < start).unwrap_or(false) {
                carry_over_pending_count += 1;
            }
            if todo.priority == 1 {
                high_priority_pending_count += 1;
                pending_items.push((
                    created_at,
                    json!({
                        "title": todo.title,
                        "category": category_name,
                        "createdAt": todo.created_at,
                        "description": todo.description,
                    }),
                ));
            }
        }
    }

    sort_by_time_desc(&mut completed_items);
    sort_by_time_desc(&mut new_items);
    sort_by_time_desc(&mut pending_items);

    let category_breakdown: Vec<Value> = category_stats
        .into_iter()
        .map(|(category, stats)| {
            json!({
                "category": category,
                "created": stats.created,
                "completed": stats.completed,
                "pending": stats.pending,
            })
        })
        .collect();

    json!({
        "timeRange": {
            "start": start.to_rfc3339(),
            "end": end.to_rfc3339(),
        },
        "overview": {
            "totalTodos": data.todos.len(),
            "createdInPeriod": created_count,
            "completedInPeriod": completed_count,
            "deletedInPeriod": deleted_count,
            "carryOverPending": carry_over_pending_count,
            "highPriorityPending": high_priority_pending_count,
        },
        "completedItems": take_values(completed_items, 20),
        "newItems": take_values(new_items, 20),
        "highPriorityPendingItems": take_values(pending_items, 15),
        "categoryBreakdown": category_breakdown,
    })
}

fn sort_by_time_desc(items: &mut [(Option<DateTime<Utc>>, Value)]) {
    items.sort_by(|a, b| b.0.cmp(&a.0));
}

fn take_values(items: Vec<(Option<DateTime<Utc>>, Value)>, max_len: usize) -> Vec<Value> {
    items
        .into_iter()
        .take(max_len)
        .map(|(_, value)| value)
        .collect()
}

fn priority_label(priority: i32) -> &'static str {
    match priority {
        1 => "高",
        2 => "中",
        _ => "低",
    }
}

fn build_system_prompt(period: ReportPeriod) -> &'static str {
    match period {
        ReportPeriod::Weekly => {
            "你是一名严谨且务实的资深团队负责人，请用中文输出高质量工作周报。输出必须使用 Markdown，且包含：1) 本周概览 2) 关键完成事项 3) 风险与阻塞 4) 下周计划 5) 需要协同支持。内容要具体、可执行，避免空话。"
        }
        ReportPeriod::Monthly => {
            "你是一名严谨且务实的资深团队负责人，请用中文输出高质量工作月报。输出必须使用 Markdown，且包含：1) 本月概览 2) 关键成果与里程碑 3) 问题与风险复盘 4) 下月重点计划 5) 需要协同支持。内容要具体、可执行，避免空话。"
        }
    }
}

fn build_user_prompt(
    period: ReportPeriod,
    window_desc: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    summary: Value,
) -> String {
    let summary_pretty =
        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string());
    format!(
        "请基于下面的任务统计数据，生成一份专业、真实、可落地的{}。\n\
时间范围：{}（{} ~ {}，UTC）\n\
要求：\n\
- 内容贴合数据，不允许虚构不存在的成果。\n\
- 对高优先级未完成任务给出明确推进建议。\n\
- 输出为 Markdown。\n\
\n\
任务统计 JSON：\n{}",
        period.label(),
        window_desc,
        start.to_rfc3339(),
        end.to_rfc3339(),
        summary_pretty
    )
}

fn build_chat_completions_url(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/');
    if normalized.ends_with("/chat/completions") {
        normalized.to_string()
    } else {
        format!("{normalized}/chat/completions")
    }
}

fn extract_text_content(content: &Value) -> Option<String> {
    match content {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => {
            let mut chunks = Vec::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    chunks.push(text.to_string());
                } else if let Some(text) = part.as_str() {
                    chunks.push(text.to_string());
                }
            }
            if chunks.is_empty() {
                None
            } else {
                Some(chunks.join("\n"))
            }
        }
        _ => None,
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }
    let mut truncated = text[..max_len].to_string();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::build_chat_completions_url;

    #[test]
    fn build_chat_url_from_base_url() {
        assert_eq!(
            build_chat_completions_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            build_chat_completions_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            build_chat_completions_url("https://example.com/chat/completions"),
            "https://example.com/chat/completions"
        );
    }
}
