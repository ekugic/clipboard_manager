use crate::models::ClipboardItem;
use dirs;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
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
        path.push("clipboard_history.bin"); // Changed extension
        
        // Use BufWriter for performance
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        // Serialize using Bincode (fast binary format)
        bincode::serialize_into(writer, items)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn load_items(&self) -> std::io::Result<Vec<ClipboardItem>> {
        let mut path = self.data_dir.clone();
        path.push("clipboard_history.bin");
        
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            
            // deserialization
            match bincode::deserialize_from(reader) {
                Ok(items) => Ok(items),
                Err(_) => {
                    // Fallback: If format changed or corrupt, start fresh
                    Ok(Vec::new()) 
                }
            }
        } else {
            Ok(Vec::new())
        }
    }
}