use serde::{Deserialize, Serialize};
use chrono::Local;
use std::hash::{Hash, Hasher};

pub const MAX_ITEMS: usize = 50;
pub const MAX_TEXT_SIZE: usize = 4 * 1024 * 1024; // 4MB for text
pub const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024; // 50MB for images
pub const THUMBNAIL_SIZE: u32 = 80; // Thumbnail dimension

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClipboardContent {
    Text(String),
    Image {
        // Store full image as PNG bytes
        png_data: Vec<u8>,
        // Pre-generated thumbnail for fast display
        thumbnail_png: Vec<u8>,
        // Original dimensions
        width: u32,
        height: u32,
    },
}

impl PartialEq for ClipboardContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ClipboardContent::Text(a), ClipboardContent::Text(b)) => a == b,
            (ClipboardContent::Image { png_data: a, .. }, ClipboardContent::Image { png_data: b, .. }) => {
                // For large images, compare hash instead of full data
                if a.len() > 10000 || b.len() > 10000 {
                    // Quick length check first
                    if a.len() != b.len() {
                        return false;
                    }
                    // Sample comparison for speed
                    a.first() == b.first() && a.last() == b.last() && a.len() == b.len()
                } else {
                    a == b
                }
            }
            _ => false,
        }
    }
}

impl Eq for ClipboardContent {}

impl Hash for ClipboardContent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ClipboardContent::Text(text) => {
                0u8.hash(state);
                text.hash(state);
            }
            ClipboardContent::Image { png_data, width, height, .. } => {
                1u8.hash(state);
                width.hash(state);
                height.hash(state);
                png_data.len().hash(state);
                // Hash first and last chunks for speed
                if png_data.len() > 0 {
                    png_data[..png_data.len().min(1024)].hash(state);
                    if png_data.len() > 1024 {
                        png_data[png_data.len() - 1024..].hash(state);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub content: ClipboardContent,
    pub timestamp: String,
    pub pinned: bool,
    pub id: String,
    #[serde(skip)]
    pub content_hash: u64,
}

impl ClipboardItem {
    pub fn new(content: ClipboardContent) -> Self {
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

    pub fn recompute_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        self.content_hash = hasher.finish();
    }
    
    pub fn is_image(&self) -> bool {
        matches!(self.content, ClipboardContent::Image { .. })
    }
    
    pub fn is_text(&self) -> bool {
        matches!(self.content, ClipboardContent::Text(_))
    }
}