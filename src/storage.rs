use crate::models::ClipboardItem;
use dirs;
use std::fs;
use std::path::PathBuf;

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("clipboard_manager");
        let _ = fs::create_dir_all(&data_dir);
        
        Self { data_dir }
    }

    pub fn save_items(&self, items: &[ClipboardItem]) -> std::io::Result<()> {
        let mut path = self.data_dir.clone();
        path.push("clipboard_history.json");
        let json = serde_json::to_string_pretty(&items)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_items(&self) -> std::io::Result<Vec<ClipboardItem>> {
        let mut path = self.data_dir.clone();
        path.push("clipboard_history.json");
        
        if path.exists() {
            let json = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&json).unwrap_or_default())
        } else {
            Ok(Vec::new())
        }
    }
}