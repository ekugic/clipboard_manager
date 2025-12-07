use serde::{Deserialize, Serialize};
use chrono::Local;

pub const MAX_ITEMS: usize = 50;
pub const MAX_ITEM_SIZE: usize = 4 * 1024 * 1024; // 4MB

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ClipboardContent {
    Text(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: ClipboardContent,
    pub timestamp: String,
    pub pinned: bool,
    pub id: String,
    // For fast duplicate detection
    #[serde(skip)]
    pub content_hash: u64,
}

impl ClipboardItem {
    pub fn new(content: ClipboardContent) -> Self {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = hasher.finish();
        
        Self {
            content,
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            pinned: false,
            id: uuid::Uuid::new_v4().to_string(),
            content_hash,
        }
    }

    pub fn with_hash(mut self) -> Self {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        self.content_hash = hasher.finish();
        self
    }
}