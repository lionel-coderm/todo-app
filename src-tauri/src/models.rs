use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryItem {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub priority: i32, // 1=High, 2=Medium, 3=Low
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "isDeleted", skip_serializing_if = "Option::is_none")]
    pub is_deleted: Option<bool>,
    #[serde(rename = "deletedAt", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub todos: Vec<TodoItem>,
    pub categories: Vec<CategoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// "json" 或 "sqlite"
    #[serde(rename = "storageType")]
    pub storage_type: String,
    /// 自定义数据目录，None 或空字符串表示使用系统默认目录
    #[serde(rename = "dataDir", skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTodosQuery {
    pub query: String,
    pub filter: String,
    #[serde(rename = "selectedCategoryId")]
    pub selected_category_id: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodoInput {
    pub title: String,
    pub priority: i32,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            storage_type: "json".to_string(),
            data_dir: None,
        }
    }
}
