use std::fs;
use std::path::PathBuf;

use crate::models::AppData;
use crate::storage::StorageBackend;

pub struct JsonStorage {
    pub file_path: PathBuf,
}

impl JsonStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            file_path: data_dir.join("todos.json"),
        }
    }
}

impl StorageBackend for JsonStorage {
    fn load(&self) -> Result<AppData, String> {
        if !self.file_path.exists() {
            return Ok(AppData::default());
        }
        let content =
            fs::read_to_string(&self.file_path).map_err(|e| format!("读取 JSON 文件失败: {e}"))?;
        let data: AppData =
            serde_json::from_str(&content).map_err(|e| format!("解析 JSON 数据失败: {e}"))?;
        Ok(data)
    }

    fn save(&self, data: &AppData) -> Result<(), String> {
        // 确保父目录存在
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
        }
        let content =
            serde_json::to_string_pretty(data).map_err(|e| format!("序列化数据失败: {e}"))?;
        fs::write(&self.file_path, content).map_err(|e| format!("写入 JSON 文件失败: {e}"))?;
        Ok(())
    }
}
