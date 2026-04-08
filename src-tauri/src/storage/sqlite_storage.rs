use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

use crate::models::{AppData, CategoryItem, CreateTodoInput, SearchTodosQuery, TodoItem};
use crate::storage::StorageBackend;

const SQLITE_SCHEMA_VERSION: i32 = 1;

pub struct SqliteStorage {
    pub db_path: PathBuf,
}

impl SqliteStorage {
    fn has_table(conn: &Connection, table: &str) -> SqlResult<bool> {
        let mut stmt =
            conn.prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1")?;
        let mut rows = stmt.query([table])?;
        Ok(rows.next()?.is_some())
    }

    fn has_column(conn: &Connection, table: &str, column: &str) -> SqlResult<bool> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn column_decl_type(conn: &Connection, table: &str, column: &str) -> SqlResult<Option<String>> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column) {
                let decl_type: String = row.get(2)?;
                return Ok(Some(decl_type));
            }
        }
        Ok(None)
    }

    fn priority_to_int_sql_expr(column: &str) -> String {
        format!(
            "
            CASE
                WHEN typeof({0}) = 'integer' AND {0} IN (1, 2, 3) THEN {0}
                WHEN lower(trim(CAST({0} AS TEXT))) = 'high' THEN 1
                WHEN lower(trim(CAST({0} AS TEXT))) = 'medium' THEN 2
                WHEN lower(trim(CAST({0} AS TEXT))) = 'low' THEN 3
                WHEN trim(CAST({0} AS TEXT)) IN ('1', '2', '3') THEN CAST(trim(CAST({0} AS TEXT)) AS INTEGER)
                ELSE 3
            END
            ",
            column
        )
    }

    fn normalize_priority_column(conn: &Connection) -> SqlResult<()> {
        let priority_expr = Self::priority_to_int_sql_expr("priority");
        conn.execute_batch(&format!(
            "
            UPDATE todos
            SET priority = {priority_expr};
            "
        ))?;
        Ok(())
    }

    fn ensure_todos_columns(conn: &Connection) -> SqlResult<()> {
        if !Self::has_column(conn, "todos", "created_at")? {
            conn.execute("ALTER TABLE todos ADD COLUMN created_at TEXT", [])?;
        }
        if !Self::has_column(conn, "todos", "completed_at")? {
            conn.execute("ALTER TABLE todos ADD COLUMN completed_at TEXT", [])?;
        }
        if !Self::has_column(conn, "todos", "is_deleted")? {
            conn.execute(
                "ALTER TABLE todos ADD COLUMN is_deleted INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        if !Self::has_column(conn, "todos", "deleted_at")? {
            conn.execute("ALTER TABLE todos ADD COLUMN deleted_at TEXT", [])?;
        }
        Ok(())
    }

    fn rebuild_todos_table(conn: &Connection) -> SqlResult<()> {
        let priority_expr = Self::priority_to_int_sql_expr("priority");
        conn.execute_batch(&format!(
            "
            DROP TABLE IF EXISTS todos_new;

            CREATE TABLE todos_new (
                id           INTEGER PRIMARY KEY,
                title        TEXT NOT NULL,
                completed    INTEGER NOT NULL DEFAULT 0,
                priority     INTEGER NOT NULL DEFAULT 3,
                category_id  TEXT NOT NULL DEFAULT '',
                description  TEXT,
                is_deleted   INTEGER NOT NULL DEFAULT 0,
                deleted_at   TEXT,
                created_at   TEXT,
                completed_at TEXT
            );

            INSERT INTO todos_new (
                id, title, completed, priority, category_id, description,
                is_deleted, deleted_at, created_at, completed_at
            )
            SELECT
                id,
                title,
                completed,
                {priority_expr},
                category_id,
                description,
                COALESCE(is_deleted, 0),
                deleted_at,
                created_at,
                completed_at
            FROM todos;

            DROP TABLE todos;
            ALTER TABLE todos_new RENAME TO todos;
            "
        ))?;

        Ok(())
    }

    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            db_path: data_dir.join("todos.db"),
        }
    }

    fn open_conn(&self) -> Result<Connection, String> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败: {e}"))?;
        }
        Connection::open(&self.db_path).map_err(|e| format!("打开 SQLite 数据库失败: {e}"))
    }

    fn migrate(conn: &Connection) -> SqlResult<()> {
        let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if current_version >= SQLITE_SCHEMA_VERSION {
            return Ok(());
        }

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
                priority     INTEGER NOT NULL DEFAULT 3,
                category_id  TEXT NOT NULL DEFAULT '',
                description  TEXT,
                is_deleted   INTEGER NOT NULL DEFAULT 0,
                deleted_at   TEXT,
                created_at   TEXT,
                completed_at TEXT
            );
            ",
        )?;

        if !Self::has_table(conn, "todos")? {
            conn.execute_batch(&format!("PRAGMA user_version = {SQLITE_SCHEMA_VERSION};"))?;
            return Ok(());
        }

        Self::ensure_todos_columns(conn)?;

        let has_due_label = Self::has_column(conn, "todos", "due_label")?;
        let priority_decl_type = Self::column_decl_type(conn, "todos", "priority")?
            .unwrap_or_default()
            .to_uppercase();
        let priority_not_integer_decl = !priority_decl_type.contains("INT");

        if has_due_label || priority_not_integer_decl {
            Self::rebuild_todos_table(conn)?;
        } else {
            Self::normalize_priority_column(conn)?;
        }

        conn.execute_batch(&format!("PRAGMA user_version = {SQLITE_SCHEMA_VERSION};"))?;
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

    fn row_to_todo_item(row: &rusqlite::Row<'_>) -> SqlResult<TodoItem> {
        Ok(TodoItem {
            id: row.get(0)?,
            title: row.get(1)?,
            completed: row.get::<_, i32>(2)? != 0,
            priority: row.get::<_, i32>(3)?,
            created_at: row.get(4)?,
            completed_at: row.get(5)?,
            category_id: row.get(6)?,
            description: row.get(7)?,
            is_deleted: Some(row.get::<_, i32>(8)? != 0),
            deleted_at: row.get(9)?,
        })
    }

    fn normalize_priority_input(priority: i32) -> i32 {
        match priority {
            1..=3 => priority,
            _ => 3,
        }
    }

    fn fetch_todo_by_id(conn: &Connection, id: i64) -> Result<Option<TodoItem>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, title, completed, priority, created_at, completed_at, category_id, description, is_deleted, deleted_at
                 FROM todos WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;

        stmt.query_row([id], Self::row_to_todo_item)
            .optional()
            .map_err(|e| e.to_string())
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
        let sort_order = params.sort_order.as_deref().unwrap_or("created");

        let mut stmt = conn
            .prepare(
                "
                SELECT id, title, completed, priority, created_at, completed_at, category_id, description, is_deleted, deleted_at
                FROM todos
                WHERE
                    (?1 = '' OR instr(lower(title), ?1) > 0 OR instr(lower(COALESCE(description, '')), ?1) > 0)
                    AND (?2 = '' OR category_id = ?2)
                    AND (
                        (?3 = 'trash' AND is_deleted = 1)
                        OR (?3 != 'trash' AND is_deleted = 0)
                    )
                    AND (
                        ?3 = 'all'
                        OR ?3 = 'trash'
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
                    CASE WHEN ?5 = 'priority' THEN priority END ASC,
                    id DESC
                ",
            )
            .map_err(|e| e.to_string())?;

        let mapped = stmt
            .query_map(
                params![
                    normalized_query,
                    selected_category_id,
                    filter,
                    like_query,
                    sort_order
                ],
                Self::row_to_todo_item,
            )
            .map_err(|e| e.to_string())?;

        let mut todos: Vec<TodoItem> = Vec::new();
        for item in mapped {
            todos.push(item.map_err(|e| e.to_string())?);
        }

        Ok(todos)
    }

    pub fn add_todo(&self, input: CreateTodoInput) -> Result<TodoItem, String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO todos
             (title, completed, priority, created_at, completed_at, category_id, description, is_deleted, deleted_at)
             VALUES (?1, 0, ?2, ?3, NULL, ?4, ?5, 0, NULL)",
            params![
                input.title,
                Self::normalize_priority_input(input.priority),
                input.created_at,
                input.category_id,
                input.description
            ],
        )
        .map_err(|e| e.to_string())?;

        let id = tx.last_insert_rowid();
        let created = Self::fetch_todo_by_id(&tx, id)?
            .ok_or_else(|| format!("新增 todo 后未找到记录: {id}"))?;
        tx.commit().map_err(|e| e.to_string())?;

        Ok(created)
    }

    pub fn update_todo(&self, todo: &TodoItem) -> Result<TodoItem, String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let affected = tx
            .execute(
                "UPDATE todos SET
                    title = ?2,
                    completed = ?3,
                    priority = ?4,
                    created_at = ?5,
                    completed_at = ?6,
                    category_id = ?7,
                    description = ?8,
                    is_deleted = ?9,
                    deleted_at = ?10
                 WHERE id = ?1",
                params![
                    todo.id,
                    todo.title,
                    todo.completed as i32,
                    Self::normalize_priority_input(todo.priority),
                    todo.created_at,
                    todo.completed_at,
                    todo.category_id,
                    todo.description,
                    todo.is_deleted.unwrap_or(false) as i32,
                    todo.deleted_at,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("更新 todo 失败，记录不存在: {}", todo.id));
        }

        let updated = Self::fetch_todo_by_id(&tx, todo.id)?
            .ok_or_else(|| format!("更新 todo 后未找到记录: {}", todo.id))?;
        tx.commit().map_err(|e| e.to_string())?;

        Ok(updated)
    }

    pub fn toggle_todo(&self, id: i64) -> Result<TodoItem, String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let affected = tx
            .execute(
                "
                UPDATE todos
                SET
                    completed = CASE WHEN completed = 0 THEN 1 ELSE 0 END,
                    completed_at = CASE
                        WHEN completed = 0 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                        ELSE NULL
                    END
                WHERE id = ?1
                ",
                [id],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("切换 todo 状态失败，记录不存在: {id}"));
        }

        let updated = Self::fetch_todo_by_id(&tx, id)?
            .ok_or_else(|| format!("切换 todo 状态后未找到记录: {id}"))?;
        tx.commit().map_err(|e| e.to_string())?;

        Ok(updated)
    }

    pub fn delete_todo(&self, id: i64) -> Result<(), String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        conn.execute("DELETE FROM todos WHERE id = ?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_category(&self, category: &CategoryItem) -> Result<(), String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        conn.execute(
            "INSERT INTO categories (id, name, color, icon) VALUES (?1, ?2, ?3, ?4)",
            params![category.id, category.name, category.color, category.icon],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_category(&self, category: &CategoryItem) -> Result<(), String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let affected = conn
            .execute(
                "UPDATE categories SET name = ?2, color = ?3, icon = ?4 WHERE id = ?1",
                params![category.id, category.name, category.color, category.icon],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("更新分类失败，记录不存在: {}", category.id));
        }

        Ok(())
    }

    pub fn delete_category(&self, category_id: &str) -> Result<(), String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM todos WHERE category_id = ?1", [category_id])
            .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM categories WHERE id = ?1", [category_id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SqliteStorage, SQLITE_SCHEMA_VERSION};
    use rusqlite::Connection;

    #[test]
    fn migrate_upgrades_legacy_todos_schema() {
        let conn = Connection::open_in_memory().expect("open sqlite");
        conn.execute_batch(
            "
            CREATE TABLE todos (
                id           INTEGER PRIMARY KEY,
                title        TEXT NOT NULL,
                completed    INTEGER NOT NULL DEFAULT 0,
                priority     TEXT NOT NULL DEFAULT 'low',
                due_label    TEXT NOT NULL DEFAULT '',
                category_id  TEXT NOT NULL DEFAULT '',
                description  TEXT
            );

            INSERT INTO todos (id, title, completed, priority, due_label, category_id) VALUES
                (1, 't1', 0, 'high', 'd', 'cat'),
                (2, 't2', 0, '2', 'd', 'cat'),
                (3, 't3', 0, 'unknown', 'd', 'cat');
            ",
        )
        .expect("seed legacy schema");

        SqliteStorage::migrate(&conn).expect("migrate");

        let has_due_label =
            SqliteStorage::has_column(&conn, "todos", "due_label").expect("check due_label");
        assert!(!has_due_label);

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("read version");
        assert_eq!(version, SQLITE_SCHEMA_VERSION);

        let mut stmt = conn
            .prepare("SELECT id, priority, typeof(priority) FROM todos ORDER BY id ASC")
            .expect("prepare");
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .expect("query")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect");

        assert_eq!(
            rows,
            vec![
                (1_i64, 1_i64, "integer".to_string()),
                (2_i64, 2_i64, "integer".to_string()),
                (3_i64, 3_i64, "integer".to_string()),
            ]
        );
    }

    #[test]
    fn migrate_is_idempotent_for_latest_schema() {
        let conn = Connection::open_in_memory().expect("open sqlite");
        SqliteStorage::migrate(&conn).expect("first migrate");
        SqliteStorage::migrate(&conn).expect("second migrate");
    }
}

impl StorageBackend for SqliteStorage {
    fn load(&self) -> Result<AppData, String> {
        let conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let categories = Self::load_categories(&conn)?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, completed, priority, created_at, completed_at, category_id, description
                 , is_deleted, deleted_at
                 FROM todos ORDER BY id DESC",
            )
            .map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map([], SqliteStorage::row_to_todo_item)
            .map_err(|e| e.to_string())?;

        let mut todos: Vec<TodoItem> = Vec::new();
        for item in mapped {
            todos.push(item.map_err(|e| e.to_string())?);
        }

        Ok(AppData { todos, categories })
    }

    fn save(&self, data: &AppData) -> Result<(), String> {
        let mut conn = self.open_conn()?;
        Self::migrate(&conn).map_err(|e| format!("数据库迁移失败: {e}"))?;

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute("DELETE FROM todos", [])
            .map_err(|e| e.to_string())?;
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
                 (id, title, completed, priority, created_at, completed_at, category_id, description, is_deleted, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    todo.id,
                    todo.title,
                    todo.completed as i32,
                    todo.priority,
                    todo.created_at,
                    todo.completed_at,
                    todo.category_id,
                    todo.description,
                    todo.is_deleted.unwrap_or(false) as i32,
                    todo.deleted_at,
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }
}
