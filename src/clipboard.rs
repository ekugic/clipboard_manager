use crate::models::{ClipboardContent, ClipboardItem, MAX_ITEMS, MAX_ITEM_SIZE};
use crate::storage::Storage;
use arboard::Clipboard;

pub struct ClipboardManager {
    items: Vec<ClipboardItem>,
    last_content: Option<ClipboardContent>,
    storage: Storage,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let storage = Storage::new();
        let items = storage.load_items().unwrap_or_default();
        
        Self {
            items,
            last_content: None,
            storage,
        }
    }

    pub fn add_item(&mut self, content: ClipboardContent) -> bool {
        // Check if it's the same as the last item
        if let Some(last) = &self.last_content {
            if content == *last {
                return false;
            }
        }

        // Check size
        let size = match &content {
            ClipboardContent::Text(text) => text.len(),
            ClipboardContent::Image(data) => data.len(),
        };

        if size > MAX_ITEM_SIZE {
            return false;
        }

        // Remove duplicate if exists (but not if it's pinned)
        self.items.retain(|existing| {
            existing.pinned || existing.content != content
        });

        let item = ClipboardItem::new(content.clone());
        
        // Find position after pinned items
        let pinned_count = self.items.iter().filter(|i| i.pinned).count();
        self.items.insert(pinned_count, item);

        // Keep only MAX_ITEMS (excluding pinned)
        let mut non_pinned_count = 0;
        self.items.retain(|item| {
            if item.pinned {
                true
            } else {
                non_pinned_count += 1;
                non_pinned_count <= MAX_ITEMS
            }
        });

        self.last_content = Some(content);
        let _ = self.storage.save_items(&self.items);
        true
    }

    pub fn toggle_pin(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.pinned = !item.pinned;
            
            // Re-sort: pinned items first
            self.items.sort_by_key(|item| !item.pinned);
            
            let _ = self.storage.save_items(&self.items);
        }
    }

    pub fn get_items(&self) -> &[ClipboardItem] {
        &self.items
    }

    pub fn check_clipboard(&mut self) -> Option<ClipboardContent> {
        let mut clipboard = Clipboard::new().ok()?;
        
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let content = ClipboardContent::Text(text);
                if self.add_item(content.clone()) {
                    return Some(content);
                }
            }
        }
        
        None
    }

    pub fn paste_item(&self, id: &str) -> Result<(), String> {
        let item = self.items.iter()
            .find(|i| i.id == id)
            .ok_or("Item not found")?;
        
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
        
        match &item.content {
            ClipboardContent::Text(text) => {
                clipboard.set_text(text.clone()).map_err(|e| e.to_string())?;
            }
            ClipboardContent::Image(_data) => {
                return Err("Image clipboard not yet implemented".to_string());
            }
        }
        
        Ok(())
    }
}