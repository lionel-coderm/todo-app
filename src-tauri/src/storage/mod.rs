pub mod json_storage;
pub mod sqlite_storage;

use crate::models::AppData;

pub trait StorageBackend {
    fn load(&self) -> Result<AppData, String>;
    fn save(&self, data: &AppData) -> Result<(), String>;
}
