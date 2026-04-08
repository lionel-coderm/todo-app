mod ai_report;
mod models;
mod storage;

use models::{AppData, AppSettings, CategoryItem, CreateTodoInput, SearchTodosQuery, TodoItem};
use storage::{json_storage::JsonStorage, sqlite_storage::SqliteStorage, StorageBackend};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

// ─── 辅助函数 ──────────────────────────────────────────

type SharedBackend = Arc<dyn StorageBackend>;

#[derive(Default)]
struct SqliteBackendCache {
    backends: Mutex<HashMap<PathBuf, SharedBackend>>,
}

impl SqliteBackendCache {
    fn get_or_create(&self, data_dir: PathBuf) -> Result<SharedBackend, String> {
        let db_path = data_dir.join("todos.db");
        let mut guard = self
            .backends
            .lock()
            .map_err(|_| "获取 SQLite backend 缓存锁失败".to_string())?;

        if let Some(existing) = guard.get(&db_path) {
            return Ok(Arc::clone(existing));
        }

        let backend: SharedBackend = Arc::new(SqliteStorage::new(data_dir)?);
        guard.insert(db_path, Arc::clone(&backend));
        Ok(backend)
    }
}

#[derive(Default)]
struct SettingsCache {
    settings: Mutex<Option<AppSettings>>,
}

impl SettingsCache {
    fn get_or_load(&self, system_dir: &PathBuf) -> Result<AppSettings, String> {
        let mut guard = self
            .settings
            .lock()
            .map_err(|_| "获取 settings 缓存锁失败".to_string())?;

        if let Some(existing) = guard.as_ref() {
            return Ok(existing.clone());
        }

        let loaded = read_settings(system_dir);
        *guard = Some(loaded.clone());
        Ok(loaded)
    }

    fn set(&self, settings: AppSettings) -> Result<(), String> {
        let mut guard = self
            .settings
            .lock()
            .map_err(|_| "更新 settings 缓存失败".to_string())?;
        *guard = Some(settings);
        Ok(())
    }
}

/// 获取系统默认应用数据目录（用于存放 settings.json）
fn get_system_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取系统数据目录失败: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建系统数据目录失败: {e}"))?;
    Ok(dir)
}

/// 设置文件路径（永远在系统目录）
fn settings_path(system_dir: &PathBuf) -> PathBuf {
    system_dir.join("settings.json")
}

/// 读取当前设置
fn read_settings(system_dir: &PathBuf) -> AppSettings {
    let path = settings_path(system_dir);
    if !path.exists() {
        return AppSettings::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 展开 ~ 为真实 home 目录
fn expand_tilde(path: &str) -> Result<PathBuf, String> {
    if path.starts_with("~/") || path == "~" {
        let home = std::env::var("HOME").map_err(|_| "无法获取 HOME 目录环境变量".to_string())?;
        Ok(PathBuf::from(home).join(path.trim_start_matches("~/")))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// 根据设置获取实际的数据目录（自定义 or 系统默认）
fn effective_data_dir(app: &tauri::AppHandle, settings: &AppSettings) -> Result<PathBuf, String> {
    if let Some(custom) = &settings.data_dir {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let expanded = expand_tilde(trimmed)?;
            fs::create_dir_all(&expanded).map_err(|e| format!("创建自定义数据目录失败: {e}"))?;
            return Ok(expanded);
        }
    }
    // 无自定义路径，回退到系统默认目录
    get_system_dir(app)
}

fn normalize_optional_field(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_settings(mut settings: AppSettings) -> AppSettings {
    settings.data_dir = normalize_optional_field(settings.data_dir.take());
    settings.ai_model = normalize_optional_field(settings.ai_model.take());
    settings.ai_base_url = normalize_optional_field(settings.ai_base_url.take());
    settings.ai_api_key = normalize_optional_field(settings.ai_api_key.take());
    settings
}

fn backend_from_settings(
    app: &tauri::AppHandle,
    cache: &SqliteBackendCache,
    settings: &AppSettings,
) -> Result<SharedBackend, String> {
    let data_dir = effective_data_dir(app, settings)?;
    if settings.storage_type == "sqlite" {
        cache.get_or_create(data_dir)
    } else {
        Ok(Arc::new(JsonStorage::new(data_dir)))
    }
}

fn backend_for_current_settings(
    app: &tauri::AppHandle,
    cache: &SqliteBackendCache,
    settings_cache: &SettingsCache,
) -> Result<SharedBackend, String> {
    let system_dir = get_system_dir(app)?;
    let settings = settings_cache.get_or_load(&system_dir)?;
    backend_from_settings(app, cache, &settings)
}

fn sqlite_backend_for_current_settings(
    app: &tauri::AppHandle,
    cache: &SqliteBackendCache,
    settings_cache: &SettingsCache,
) -> Result<SharedBackend, String> {
    let system_dir = get_system_dir(app)?;
    let settings = settings_cache.get_or_load(&system_dir)?;

    if settings.storage_type != "sqlite" {
        return Err("当前不是 SQLite 存储模式，无法使用 SQLite 命令".to_string());
    }

    backend_from_settings(app, cache, &settings)
}

// ─── Tauri 命令 ────────────────────────────────────────

/// 读取 todos + categories
#[tauri::command]
fn load_app_data(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
) -> Result<AppData, String> {
    let backend = backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.load()
}

/// 保存 todos + categories
#[tauri::command]
fn save_app_data(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    data: AppData,
) -> Result<(), String> {
    let backend = backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.save(&data)
}

/// 在 SQLite 模式下执行任务搜索
#[tauri::command]
fn search_todos(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    query: String,
    filter: String,
    selected_category_id: Option<String>,
    sort_order: Option<String>,
) -> Result<Vec<TodoItem>, String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.search_todos(SearchTodosQuery {
        query,
        filter,
        selected_category_id,
        sort_order,
    })
}

#[tauri::command]
fn add_todo(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    input: CreateTodoInput,
) -> Result<TodoItem, String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.add_todo(input)
}

#[tauri::command]
fn update_todo(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    todo: TodoItem,
) -> Result<TodoItem, String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.update_todo(&todo)
}

#[tauri::command]
fn toggle_todo(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    id: i64,
) -> Result<TodoItem, String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.toggle_todo(id)
}

#[tauri::command]
fn delete_todo(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    id: i64,
) -> Result<(), String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.delete_todo(id)
}

#[tauri::command]
fn add_category(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    category: CategoryItem,
) -> Result<(), String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.add_category(&category)
}

#[tauri::command]
fn update_category(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    category: CategoryItem,
) -> Result<(), String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.update_category(&category)
}

#[tauri::command]
fn delete_category(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    id: String,
) -> Result<(), String> {
    let backend = sqlite_backend_for_current_settings(&app, &sqlite_cache, &settings_cache)?;
    backend.delete_category(&id)
}

#[tauri::command]
async fn generate_ai_report(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    period: String,
) -> Result<String, String> {
    let system_dir = get_system_dir(&app)?;
    let settings = settings_cache.get_or_load(&system_dir)?;
    let backend = backend_from_settings(&app, &sqlite_cache, &settings)?;
    let data = backend.load()?;
    ai_report::generate_ai_report(&settings, &data, &period).await
}

/// 读取应用设置
#[tauri::command]
fn load_settings(
    app: tauri::AppHandle,
    settings_cache: tauri::State<'_, SettingsCache>,
) -> Result<AppSettings, String> {
    let system_dir = get_system_dir(&app)?;
    settings_cache.get_or_load(&system_dir)
}

/// 保存应用设置（存储类型或数据目录变化时，自动将数据迁移到新位置）
#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    sqlite_cache: tauri::State<'_, SqliteBackendCache>,
    settings_cache: tauri::State<'_, SettingsCache>,
    settings: AppSettings,
    data: AppData,
) -> Result<(), String> {
    let settings = normalize_settings(settings);
    let system_dir = get_system_dir(&app)?;
    let old_settings = settings_cache.get_or_load(&system_dir)?;

    let type_changed = old_settings.storage_type != settings.storage_type;
    let dir_changed = old_settings.data_dir != settings.data_dir;

    // 只要类型或目录有变化，就把数据写入新位置
    if type_changed || dir_changed {
        let backend = backend_from_settings(&app, &sqlite_cache, &settings)?;
        backend.save(&data)?;
    }

    // 将新设置写入系统目录
    let path = settings_path(&system_dir);
    let content =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("序列化设置失败: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("写入设置失败: {e}"))?;
    settings_cache.set(settings)?;

    Ok(())
}

/// 返回系统默认数据目录路径（供前端展示占位符）
#[tauri::command]
fn get_default_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let dir = get_system_dir(&app)?;
    Ok(dir.display().to_string())
}

#[tauri::command]
fn show_main_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| format!("显示主窗口失败: {e}"))?;
    Ok(())
}

// ─── 入口点 ────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SqliteBackendCache::default())
        .manage(SettingsCache::default())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_app_data,
            save_app_data,
            search_todos,
            add_todo,
            update_todo,
            toggle_todo,
            delete_todo,
            add_category,
            update_category,
            delete_category,
            generate_ai_report,
            load_settings,
            save_settings,
            get_default_data_dir,
            show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
