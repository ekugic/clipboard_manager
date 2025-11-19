use serde::{Deserialize, Serialize};
use chrono::Local;

pub const MAX_ITEMS: usize = 25;
pub const MAX_ITEM_SIZE: usize = 4 * 1024 * 1024; // 4MB

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClipboardContent {
    Text(String),
    // Image support removed for performance testing
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: ClipboardContent,
    pub timestamp: String,
    pub pinned: bool,
    pub id: String,
}

impl ClipboardItem {
    pub fn new(content: ClipboardContent) -> Self {
        Self {
            content,
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            pinned: false,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_pin(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }
}