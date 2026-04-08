pub mod json_storage;
pub mod sqlite_storage;

use crate::models::{AppData, CategoryItem, CreateTodoInput, SearchTodosQuery, TodoItem};

pub trait StorageBackend: Send + Sync {
    fn load(&self) -> Result<AppData, String>;
    fn save(&self, data: &AppData) -> Result<(), String>;

    fn search_todos(&self, _params: SearchTodosQuery) -> Result<Vec<TodoItem>, String> {
        Err("当前存储后端不支持 search_todos".to_string())
    }

    fn add_todo(&self, _input: CreateTodoInput) -> Result<TodoItem, String> {
        Err("当前存储后端不支持 add_todo".to_string())
    }

    fn update_todo(&self, _todo: &TodoItem) -> Result<TodoItem, String> {
        Err("当前存储后端不支持 update_todo".to_string())
    }

    fn toggle_todo(&self, _id: i64) -> Result<TodoItem, String> {
        Err("当前存储后端不支持 toggle_todo".to_string())
    }

    fn delete_todo(&self, _id: i64) -> Result<(), String> {
        Err("当前存储后端不支持 delete_todo".to_string())
    }

    fn add_category(&self, _category: &CategoryItem) -> Result<(), String> {
        Err("当前存储后端不支持 add_category".to_string())
    }

    fn update_category(&self, _category: &CategoryItem) -> Result<(), String> {
        Err("当前存储后端不支持 update_category".to_string())
    }

    fn delete_category(&self, _category_id: &str) -> Result<(), String> {
        Err("当前存储后端不支持 delete_category".to_string())
    }
}
