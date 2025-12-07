use crate::models::ClipboardItem;
use dirs;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub struct Storage {
    data_dir: PathBuf,
    save_sender: mpsc::Sender<Vec<ClipboardItem>>,
}

impl Storage {
    pub fn new() -> Self {
        let mut data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        data_dir.push("clipboard_manager");
        let _ = fs::create_dir_all(&data_dir);
        
        // Spawn async save thread
        let (tx, rx) = mpsc::channel::<Vec<ClipboardItem>>();
        let save_path = data_dir.join("clipboard_history.bin");
        
        thread::spawn(move || {
            while let Ok(items) = rx.recv() {
                // Only save the latest - skip old queued saves
                let mut latest_items = items;
                while let Ok(newer_items) = rx.try_recv() {
                    latest_items = newer_items;
                }
                
                if let Ok(file) = File::create(&save_path) {
                    let writer = BufWriter::new(file);
                    let _ = bincode::serialize_into(writer, &latest_items);
                }
            }
        });
        
        Self { 
            data_dir,
            save_sender: tx,
        }
    }

    /// Non-blocking async save
    pub fn save_items_async(&self, items: &[ClipboardItem]) {
        let _ = self.save_sender.send(items.to_vec());
    }

    pub fn load_items(&self) -> Vec<ClipboardItem> {
        let path = self.data_dir.join("clipboard_history.bin");
        
        if path.exists() {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(mut items) = bincode::deserialize_from::<_, Vec<ClipboardItem>>(reader) {
                    // Recompute hashes after load
                    for item in &mut items {
                        *item = item.clone().with_hash();
                    }
                    return items;
                }
            }
        }
        Vec::new()
    }
}