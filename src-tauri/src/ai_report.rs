use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::{json, Value};

use crate::models::{AppData, AppSettings};
use crate::utils::normalize_optional_text;

const MAX_AI_USER_PROMPT_CHARS: usize = 12_000;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiApiMode {
    Auto,
    ChatCompletions,
    AnthropicMessages,
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

#[derive(Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicTextBlock>,
}

#[derive(Serialize)]
struct AnthropicTextBlock {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

trait AiProtocolAdapter: Send + Sync {
    fn default_endpoint(&self, base_url: &str) -> String;
    fn request_body(&self, model: &str, system_prompt: &str, user_prompt: &str) -> Value;
    fn apply_headers(&self, request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder;
}

struct ChatCompletionsAdapter;

impl AiProtocolAdapter for ChatCompletionsAdapter {
    fn default_endpoint(&self, base_url: &str) -> String {
        build_chat_completions_url(base_url)
    }

    fn request_body(&self, model: &str, system_prompt: &str, user_prompt: &str) -> Value {
        json!(ChatCompletionRequest {
            model: model.to_string(),
            temperature: 0.2,
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
        })
    }

    fn apply_headers(&self, request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
        request.bearer_auth(api_key)
    }
}

struct AnthropicMessagesAdapter;

impl AiProtocolAdapter for AnthropicMessagesAdapter {
    fn default_endpoint(&self, base_url: &str) -> String {
        build_anthropic_messages_url(base_url)
    }

    fn request_body(&self, model: &str, system_prompt: &str, user_prompt: &str) -> Value {
        json!(AnthropicMessagesRequest {
            model: model.to_string(),
            max_tokens: 1024,
            temperature: None,
            system: system_prompt.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: vec![AnthropicTextBlock {
                    kind: "text".to_string(),
                    text: user_prompt.to_string(),
                }],
            }],
        })
    }

    fn apply_headers(&self, request: reqwest::RequestBuilder, api_key: &str) -> reqwest::RequestBuilder {
        request
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
    }
}

pub async fn generate_ai_report(
    settings: &AppSettings,
    data: &AppData,
    period_raw: &str,
) -> Result<String, String> {
    let period = ReportPeriod::parse(period_raw)?;
    let model = normalize_optional_text(settings.ai_model.as_deref())
        .ok_or_else(|| "请先在设置中配置 AI 模型名称".to_string())?;
    let base_url = normalize_optional_text(settings.ai_base_url.as_deref());
    let endpoint = normalize_optional_text(settings.ai_endpoint.as_deref());
    if base_url.is_none() && endpoint.is_none() {
        return Err("请先在设置中配置 AI 请求地址（请求地址或完整请求地址至少填写一个）".to_string());
    }
    let api_key = normalize_optional_text(settings.ai_api_key.as_deref())
        .ok_or_else(|| "请先在设置中配置 AI API Key".to_string())?;

    let now = Utc::now();
    let (start, end, window_desc) = period.range(now)?;
    let summary = build_report_summary(data, start, end);
    let system_prompt = build_system_prompt(period).to_string();
    let user_prompt = limit_user_prompt_length(
        build_user_prompt(period, &window_desc, start, end, summary),
        MAX_AI_USER_PROMPT_CHARS,
    );

    let (api_mode, allow_fallback) = resolve_mode_plan(
        settings.ai_api_mode.as_deref(),
        base_url.as_deref(),
        endpoint.as_deref(),
    )?;
    let mode_candidates = resolve_mode_candidates(api_mode, allow_fallback);
    let client = Client::new();
    let mut attempt_errors: Vec<String> = Vec::new();

    for (index, mode) in mode_candidates.iter().copied().enumerate() {
        let (status, body) = send_ai_request(
            &client,
            mode,
            base_url.as_deref(),
            endpoint.as_deref(),
            &api_key,
            &model,
            &system_prompt,
            &user_prompt,
        )
        .await?;

        let has_next = index + 1 < mode_candidates.len();
        if status.is_success() {
            let content = match parse_ai_response_text(&body) {
                Ok(content) => content,
                Err(parse_err) => {
                    attempt_errors.push(format!(
                        "{} 模式 (响应解析): {}",
                        ai_api_mode_label(mode),
                        truncate_text(&parse_err, 400)
                    ));
                    if has_next && should_retry_with_fallback_on_parse_error(&parse_err) {
                        continue;
                    }
                    break;
                }
            };
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return Err("AI 返回内容为空，请稍后重试".to_string());
            }
            return Ok(trimmed.to_string());
        }

        attempt_errors.push(format!(
            "{} 模式 (HTTP {}): {}",
            ai_api_mode_label(mode),
            status.as_u16(),
            truncate_text(&body, 400)
        ));

        if !(has_next && should_retry_with_fallback(status)) {
            break;
        }
    }

    if attempt_errors.len() <= 1 {
        return Err(format!("AI 接口返回错误: {}", attempt_errors[0]));
    }

    Err(format!(
        "AI 接口返回错误（自动模式已重试备选协议）: {}",
        attempt_errors.join("；")
    ))
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

fn build_anthropic_messages_url(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/');
    if normalized.ends_with("/v1/messages") {
        return normalized.to_string();
    }
    if normalized.ends_with("/anthropic/v1") {
        return format!("{normalized}/messages");
    }
    if normalized.ends_with("/anthropic") {
        return format!("{normalized}/v1/messages");
    }
    if normalized.ends_with("/v1") {
        return format!("{normalized}/messages");
    }
    format!("{normalized}/v1/messages")
}

fn adapter_for_mode(mode: AiApiMode) -> Box<dyn AiProtocolAdapter> {
    match mode {
        AiApiMode::ChatCompletions => Box::new(ChatCompletionsAdapter),
        AiApiMode::AnthropicMessages => Box::new(AnthropicMessagesAdapter),
        AiApiMode::Auto => Box::new(ChatCompletionsAdapter),
    }
}

fn ai_api_mode_label(mode: AiApiMode) -> &'static str {
    match mode {
        AiApiMode::Auto => "auto",
        AiApiMode::ChatCompletions => "chat_completions",
        AiApiMode::AnthropicMessages => "anthropic_messages",
    }
}

fn resolve_mode_candidates(primary_mode: AiApiMode, allow_fallback: bool) -> Vec<AiApiMode> {
    if !allow_fallback {
        return vec![primary_mode];
    }

    match primary_mode {
        AiApiMode::ChatCompletions => vec![AiApiMode::ChatCompletions, AiApiMode::AnthropicMessages],
        AiApiMode::AnthropicMessages => vec![AiApiMode::AnthropicMessages, AiApiMode::ChatCompletions],
        AiApiMode::Auto => vec![AiApiMode::ChatCompletions, AiApiMode::AnthropicMessages],
    }
}

fn parse_configured_api_mode(raw: Option<&str>) -> Result<AiApiMode, String> {
    let Some(normalized) = normalize_optional_text(raw) else {
        return Ok(AiApiMode::Auto);
    };
    let normalized = normalized.to_lowercase();
    match normalized.as_str() {
        "auto" => Ok(AiApiMode::Auto),
        "chat_completions" | "chat-completions" | "chatcompletions" => {
            Ok(AiApiMode::ChatCompletions)
        }
        "anthropic_messages" | "anthropic-messages" | "anthropicmessages" | "anthropic" => {
            Ok(AiApiMode::AnthropicMessages)
        }
        _ => Err(format!(
            "AI 协议模式无效: {normalized}。支持 auto / chat_completions / anthropic_messages"
        )),
    }
}

fn resolve_mode_plan(
    configured_mode: Option<&str>,
    base_url: Option<&str>,
    endpoint: Option<&str>,
) -> Result<(AiApiMode, bool), String> {
    let parsed = parse_configured_api_mode(configured_mode)?;
    if parsed == AiApiMode::Auto {
        Ok((detect_api_mode(base_url, endpoint), true))
    } else {
        Ok((parsed, false))
    }
}

fn resolve_request_url(
    adapter: &dyn AiProtocolAdapter,
    base_url: Option<&str>,
    endpoint: Option<&str>,
) -> Result<String, String> {
    if let Some(custom_endpoint) = endpoint {
        let trimmed = custom_endpoint.trim();
        if !trimmed.is_empty() {
            if should_treat_endpoint_as_base_url(trimmed) {
                return Ok(adapter.default_endpoint(trimmed));
            }
            return Ok(trimmed.to_string());
        }
    }

    let base = base_url.ok_or_else(|| "请先在设置中配置 AI 请求地址".to_string())?;
    Ok(adapter.default_endpoint(base))
}

fn is_likely_full_request_endpoint(url: &str) -> bool {
    let normalized = url.trim().to_lowercase();
    normalized.contains("/chat/completions")
        || normalized.contains("/v1/messages")
        || normalized.contains("/responses")
}

fn should_treat_endpoint_as_base_url(url: &str) -> bool {
    if is_likely_full_request_endpoint(url) {
        return false;
    }

    let normalized = url.trim().trim_end_matches('/').to_lowercase();
    if normalized.is_empty() {
        return true;
    }

    if normalized.ends_with("/v1")
        || normalized.ends_with("/anthropic")
        || normalized.ends_with("/anthropic/v1")
    {
        return true;
    }

    if let Ok(parsed) = reqwest::Url::parse(&normalized) {
        let path = parsed.path().trim_end_matches('/');
        return path.is_empty();
    }

    false
}

async fn send_ai_request(
    client: &Client,
    mode: AiApiMode,
    base_url: Option<&str>,
    endpoint: Option<&str>,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<(StatusCode, String), String> {
    let adapter = adapter_for_mode(mode);
    let request_url = resolve_request_url(adapter.as_ref(), base_url, endpoint)?;
    let request_body = adapter.request_body(model, system_prompt, user_prompt);
    let request_builder = client.post(request_url);
    let response = adapter
        .apply_headers(request_builder, api_key)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("请求 AI 接口失败: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("读取 AI 响应失败: {e}"))?;
    Ok((status, body))
}

fn should_retry_with_fallback(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::BAD_REQUEST
            | StatusCode::NOT_FOUND
            | StatusCode::METHOD_NOT_ALLOWED
            | StatusCode::UNSUPPORTED_MEDIA_TYPE
            | StatusCode::UNPROCESSABLE_ENTITY
    )
}

fn should_retry_with_fallback_on_parse_error(err: &str) -> bool {
    let normalized = err.to_lowercase();
    normalized.contains("404")
        || normalized.contains("not_found")
        || normalized.contains("not found")
        || normalized.contains("method not allowed")
        || normalized.contains("unsupported")
        || normalized.contains("invalid_request")
        || normalized.contains("参数有误")
}

fn detect_api_mode(base_url: Option<&str>, endpoint: Option<&str>) -> AiApiMode {
    if endpoint
        .map(looks_like_anthropic_endpoint)
        .unwrap_or(false)
    {
        return AiApiMode::AnthropicMessages;
    }

    if base_url
        .map(looks_like_anthropic_endpoint)
        .unwrap_or(false)
    {
        return AiApiMode::AnthropicMessages;
    }

    AiApiMode::ChatCompletions
}

fn looks_like_anthropic_endpoint(raw: &str) -> bool {
    let normalized = raw.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }

    if normalized.contains("anthropic.com") {
        return true;
    }

    if normalized.contains("/anthropic") {
        return true;
    }

    normalized.ends_with("/v1/messages") || normalized.contains("/v1/messages?")
}

fn parse_ai_response_text(body: &str) -> Result<String, String> {
    let parsed: Value =
        serde_json::from_str(body).map_err(|e| format!("解析 AI 响应失败: {e}"))?;

    if let Some(error_message) = extract_ai_error_message(&parsed) {
        return Err(format!("AI 接口返回错误: {error_message}"));
    }

    let content = extract_openai_response_text(&parsed)
        .or_else(|| extract_anthropic_response_text(&parsed))
        .or_else(|| extract_generic_response_text(&parsed))
        .ok_or_else(|| {
            format!(
                "AI 响应格式异常，未找到文本内容。响应片段: {}",
                truncate_text(body, 240)
            )
        })?;

    Ok(content)
}

fn extract_ai_error_message(payload: &Value) -> Option<String> {
    if let Some(error) = payload.get("error") {
        if let Some(msg) = error.as_str() {
            return Some(msg.to_string());
        }
        if let Some(msg) = error.get("message").and_then(Value::as_str) {
            if let Some(err_type) = error.get("type").and_then(Value::as_str) {
                return Some(format!("{err_type}: {msg}"));
            }
            return Some(msg.to_string());
        }
        if let Some(msg) = error.get("msg").and_then(Value::as_str) {
            return Some(msg.to_string());
        }
    }

    if payload.get("choices").is_none() && payload.get("content").is_none() {
        if let Some(msg) = payload.get("message").and_then(Value::as_str) {
            return Some(msg.to_string());
        }
        if let Some(msg) = payload.get("msg").and_then(Value::as_str) {
            return Some(msg.to_string());
        }
    }

    None
}

fn extract_openai_response_text(payload: &Value) -> Option<String> {
    let first = payload
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())?;

    if let Some(content) = first.get("message").and_then(|message| message.get("content")) {
        return extract_text_content(content);
    }

    first
        .get("text")
        .and_then(Value::as_str)
        .map(|text| text.to_string())
}

fn extract_anthropic_response_text(payload: &Value) -> Option<String> {
    if let Some(output_text) = payload.get("output_text").and_then(Value::as_str) {
        return Some(output_text.to_string());
    }

    if let Some(completion) = payload.get("completion").and_then(Value::as_str) {
        return Some(completion.to_string());
    }

    payload.get("content").and_then(extract_text_content)
}

fn extract_generic_response_text(payload: &Value) -> Option<String> {
    if let Some(message_content) = payload.get("message").and_then(|message| message.get("content"))
    {
        return extract_text_content(message_content);
    }

    if let Some(text) = payload.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    None
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

fn limit_user_prompt_length(prompt: String, max_chars: usize) -> String {
    let char_count = prompt.chars().count();
    if char_count <= max_chars {
        return prompt;
    }

    let mut limited: String = prompt.chars().take(max_chars).collect();
    limited.push_str("\n\n[注：输入数据过长，已自动截断以满足模型上下文限制。]");
    limited
}

fn truncate_text(text: &str, max_len: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_len {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(max_len).collect();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::{
        adapter_for_mode, build_anthropic_messages_url, build_chat_completions_url, detect_api_mode,
        is_likely_full_request_endpoint, limit_user_prompt_length, parse_ai_response_text,
        resolve_mode_plan, resolve_request_url, should_retry_with_fallback_on_parse_error,
        should_treat_endpoint_as_base_url, truncate_text, AiApiMode,
    };
    use serde_json::json;

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

    #[test]
    fn build_anthropic_messages_url_from_base_url() {
        assert_eq!(
            build_anthropic_messages_url("https://open.bigmodel.cn/api/anthropic"),
            "https://open.bigmodel.cn/api/anthropic/v1/messages"
        );
        assert_eq!(
            build_anthropic_messages_url("https://open.bigmodel.cn/api/anthropic/v1"),
            "https://open.bigmodel.cn/api/anthropic/v1/messages"
        );
        assert_eq!(
            build_anthropic_messages_url("https://open.bigmodel.cn/api/anthropic/v1/messages"),
            "https://open.bigmodel.cn/api/anthropic/v1/messages"
        );
    }

    #[test]
    fn detect_api_mode_from_base_url() {
        assert!(matches!(
            detect_api_mode(Some("https://open.bigmodel.cn/api/anthropic"), None),
            AiApiMode::AnthropicMessages
        ));
        assert!(matches!(
            detect_api_mode(Some("https://api.openai.com/v1"), None),
            AiApiMode::ChatCompletions
        ));
    }

    #[test]
    fn detect_api_mode_for_official_anthropic_v1_url() {
        assert!(matches!(
            detect_api_mode(Some("https://api.anthropic.com/v1"), None),
            AiApiMode::AnthropicMessages
        ));
    }

    #[test]
    fn resolve_api_mode_respects_explicit_configuration() {
        let (mode, allow_fallback) = resolve_mode_plan(
                Some("chat_completions"),
                Some("https://api.anthropic.com/v1"),
                None
            )
            .expect("parse mode");
        assert!(matches!(mode, AiApiMode::ChatCompletions));
        assert!(!allow_fallback);

        let (mode, allow_fallback) =
            resolve_mode_plan(Some("anthropic_messages"), Some("https://api.openai.com/v1"), None)
                .expect("parse mode");
        assert!(matches!(mode, AiApiMode::AnthropicMessages));
        assert!(!allow_fallback);
    }

    #[test]
    fn resolve_request_url_prefers_endpoint_override() {
        let adapter = adapter_for_mode(AiApiMode::ChatCompletions);
        let url = resolve_request_url(
            adapter.as_ref(),
            Some("https://api.openai.com/v1"),
            Some("https://gateway.example.com/custom/path"),
        )
        .expect("resolve endpoint");
        assert_eq!(url, "https://gateway.example.com/custom/path");
    }

    #[test]
    fn resolve_request_url_treats_endpoint_as_base_when_path_missing() {
        let chat_adapter = adapter_for_mode(AiApiMode::ChatCompletions);
        let chat_url = resolve_request_url(
            chat_adapter.as_ref(),
            Some("https://api.openai.com/v1"),
            Some("https://api.anthropic.com/v1"),
        )
        .expect("resolve chat endpoint");
        assert_eq!(chat_url, "https://api.anthropic.com/v1/chat/completions");

        let anthropic_adapter = adapter_for_mode(AiApiMode::AnthropicMessages);
        let anthropic_url = resolve_request_url(
            anthropic_adapter.as_ref(),
            Some("https://api.openai.com/v1"),
            Some("https://api.anthropic.com/v1"),
        )
        .expect("resolve anthropic endpoint");
        assert_eq!(anthropic_url, "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn detect_full_request_endpoint_works_for_common_paths() {
        assert!(is_likely_full_request_endpoint(
            "https://api.openai.com/v1/chat/completions"
        ));
        assert!(is_likely_full_request_endpoint(
            "https://api.anthropic.com/v1/messages"
        ));
        assert!(!is_likely_full_request_endpoint("https://api.anthropic.com/v1"));
    }

    #[test]
    fn treat_only_base_like_endpoint_as_base_url() {
        assert!(should_treat_endpoint_as_base_url("https://api.anthropic.com/v1"));
        assert!(should_treat_endpoint_as_base_url("https://api.openai.com"));
        assert!(!should_treat_endpoint_as_base_url(
            "https://gateway.example.com/custom/path"
        ));
    }

    #[test]
    fn retry_fallback_on_retryable_parse_error() {
        assert!(should_retry_with_fallback_on_parse_error(
            "AI 接口返回错误: 404 NOT_FOUND"
        ));
        assert!(should_retry_with_fallback_on_parse_error(
            "AI 接口返回错误: API 调用参数有误，请检查文档。"
        ));
        assert!(!should_retry_with_fallback_on_parse_error(
            "AI 接口返回错误: unauthorized"
        ));
    }

    #[test]
    fn truncate_text_is_utf8_safe() {
        let input = "你好，世界，hello";
        let result = truncate_text(input, 5);
        assert_eq!(result, "你好，世界...");
    }

    #[test]
    fn limit_user_prompt_length_appends_notice_when_truncated() {
        let input = "a".repeat(20);
        let limited = limit_user_prompt_length(input, 8);
        assert!(limited.starts_with("aaaaaaaa"));
        assert!(limited.contains("已自动截断"));
    }

    #[test]
    fn parse_openai_compatible_response() {
        let body = json!({
            "choices": [
                {
                    "message": {
                        "content": "本周完成了三个核心需求交付。"
                    }
                }
            ]
        })
        .to_string();

        let text = parse_ai_response_text(&body).expect("should parse");
        assert_eq!(text, "本周完成了三个核心需求交付。");
    }

    #[test]
    fn parse_anthropic_response() {
        let body = json!({
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "本周完成了 A/B/C 三项重点工作。"
                }
            ]
        })
        .to_string();

        let text = parse_ai_response_text(&body).expect("should parse");
        assert_eq!(text, "本周完成了 A/B/C 三项重点工作。");
    }

    #[test]
    fn parse_error_payload_returns_message() {
        let body = json!({
            "error": {
                "type": "invalid_request_error",
                "message": "model not found"
            }
        })
        .to_string();

        let err = parse_ai_response_text(&body).expect_err("should fail");
        assert!(err.contains("invalid_request_error"));
        assert!(err.contains("model not found"));
    }
}
