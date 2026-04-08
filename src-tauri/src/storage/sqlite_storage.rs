use std::path::PathBuf;

use rusqlite::{params, Connection, Result as SqlResult};

use crate::models::{AppData, CategoryItem, SearchTodosQuery, TodoItem};
use crate::storage::StorageBackend;

pub struct SqliteStorage {
    pub db_path: PathBuf,
}

impl SqliteStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            db_path: data_dir.join("todos.db"),
        }
    }

    fn open_conn(&self) -> Result<Connection, String> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建数据库目录失败: {e}"))?;
        }
        Connection::open(&self.db_path)
            .map_err(|e| format!("打开 SQLite 数据库失败: {e}"))
    }

    fn migrate(conn: &Connection) -> SqlResult<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS categories (
                id   TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                icon TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS todos (
                id           INTEGER PRIMARY KEY,
                title        TEXT NOT NULL,
                completed    INTEGER NOT NULL DEFAULT 0,
                priority     TEXT NOT NULL DEFAULT 'low',
                due_label    TEXT NOT NULL DEFAULT '',
                category_id  TEXT NOT NULL DEFAULT '',
                description  TEXT
            );
            ",
        )?;

        let _ = conn.execute("ALTER TABLE todos ADD COLUMN created_at TEXT", []);
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN completed_at TEXT", []);

        Ok(())
    }

    fn load_categories(conn: &Connection) -> Result<Vec<CategoryItem>, String> {
        let mut stmt = conn
            .prepare("SELECT id, name, color, icon FROM categories")
            .map_err(|e| e.to_string())?;
        let categories: Vec<CategoryItem> = stmt
            .query_map([], |row| {
                Ok(CategoryItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(categories)
    }

    pub fn search(&self, params: SearchTodosQuery) -> Result<Vec<TodoItem>, String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let normalized_query = params.query.trim().to_lowercase();
        let like_query = if normalized_query.is_empty() {
            String::new()
        } else {
            format!("%{normalized_query}%")
        };

        let filter = params.filter.trim().to_lowercase();
        let selected_category_id = params.selected_category_id.as_deref().unwrap_or("");

        let mut stmt = conn
            .prepare(
                "
                SELECT id, title, completed, priority, due_label, created_at, completed_at, category_id, description
                FROM todos
                WHERE
                    (?1 = '' OR instr(lower(title), ?1) > 0 OR instr(lower(COALESCE(description, '')), ?1) > 0)
                    AND (?2 = '' OR category_id = ?2)
                    AND (
                        ?3 = 'all'
                        OR (?3 = 'active' AND completed = 0)
                        OR (?3 = 'completed' AND completed = 1)
                    )
                ORDER BY
                    CASE
                        WHEN ?1 = '' THEN 0
                        WHEN lower(title) = ?1 THEN 0
                        WHEN lower(title) LIKE ?4 THEN 1
                        WHEN instr(lower(title), ?1) > 0 THEN 2
                        WHEN instr(lower(COALESCE(description, '')), ?1) > 0 THEN 3
                        ELSE 4
                    END,
                    id DESC
                ",
            )
            .map_err(|e| e.to_string())?;

        let todos: Vec<TodoItem> = stmt
            .query_map(
                params![normalized_query, selected_category_id, filter, like_query],
                |row| {
                    Ok(TodoItem {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        completed: row.get::<_, i32>(2)? != 0,
                        priority: row.get(3)?,
                        due_label: row.get(4)?,
                        created_at: row.get(5)?,
                        completed_at: row.get(6)?,
                        category_id: row.get(7)?,
                        description: row.get(8)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(todos)
    }
}

impl StorageBackend for SqliteStorage {
    fn load(&self) -> Result<AppData, String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let categories = Self::load_categories(&conn)?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, completed, priority, due_label, created_at, completed_at, category_id, description
                 FROM todos ORDER BY id DESC",
            )
            .map_err(|e| e.to_string())?;
        let todos: Vec<TodoItem> = stmt
            .query_map([], |row| {
                Ok(TodoItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    completed: row.get::<_, i32>(2)? != 0,
                    priority: row.get(3)?,
                    due_label: row.get(4)?,
                    created_at: row.get(5)?,
                    completed_at: row.get(6)?,
                    category_id: row.get(7)?,
                    description: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(AppData { todos, categories })
    }

    fn save(&self, data: &AppData) -> Result<(), String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute("DELETE FROM todos", []).map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM categories", [])
            .map_err(|e| e.to_string())?;

        for cat in &data.categories {
            tx.execute(
                "INSERT OR REPLACE INTO categories (id, name, color, icon) VALUES (?1, ?2, ?3, ?4)",
                params![cat.id, cat.name, cat.color, cat.icon],
            )
            .map_err(|e| e.to_string())?;
        }

        for todo in &data.todos {
            tx.execute(
                "INSERT OR REPLACE INTO todos
                 (id, title, completed, priority, due_label, created_at, completed_at, category_id, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    todo.id,
                    todo.title,
                    todo.completed as i32,
                    todo.priority,
                    todo.due_label,
                    todo.created_at,
                    todo.completed_at,
                    todo.category_id,
                    todo.description,
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }
}
