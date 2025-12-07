use crate::models::{ClipboardContent, ClipboardItem, MAX_ITEMS, MAX_ITEM_SIZE};
use crate::storage::Storage;
use arboard::Clipboard;
use parking_lot::RwLock;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Fast hash computation for duplicate detection
fn compute_hash(content: &ClipboardContent) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

pub struct ClipboardManager {
    items: Vec<ClipboardItem>,
    last_hash: u64,
    storage: Storage,
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let storage = Storage::new();
        let items = storage.load_items();
        
        // Pre-initialize clipboard
        let clipboard = Clipboard::new().ok();
        
        // Get initial hash
        let last_hash = clipboard.as_ref()
            .and_then(|c| {
                // Create a temporary mutable reference
                let mut temp_clipboard = Clipboard::new().ok()?;
                temp_clipboard.get_text().ok()
            })
            .map(|text| compute_hash(&ClipboardContent::Text(text)))
            .unwrap_or(0);
        
        Self {
            items,
            last_hash,
            storage,
            clipboard,
        }
    }

    pub fn add_item(&mut self, content: ClipboardContent) -> bool {
        let new_hash = compute_hash(&content);
        
        // Fast hash-based duplicate check
        if new_hash == self.last_hash {
            return false;
        }

        // Check size
        let size = match &content {
            ClipboardContent::Text(text) => text.len(),
        };

        if size > MAX_ITEM_SIZE || size == 0 {
            return false;
        }

        // Remove duplicate if exists (but not if pinned) - use hash for speed
        self.items.retain(|existing| {
            existing.pinned || existing.content_hash != new_hash
        });

        let item = ClipboardItem::new(content);
        
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

        self.last_hash = new_hash;
        
        // Async save - doesn't block!
        self.storage.save_items_async(&self.items);
        true
    }

    pub fn toggle_pin(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.pinned = !item.pinned;
            self.items.sort_by_key(|item| !item.pinned);
            self.storage.save_items_async(&self.items);
        }
    }

    #[inline]
    pub fn get_items(&self) -> &[ClipboardItem] {
        &self.items
    }

    /// Ultra-fast clipboard check - returns true if changed
    #[inline]
    pub fn check_clipboard_fast(&mut self) -> bool {
        // Re-create clipboard if needed (they can become stale on X11)
        let clipboard = match &mut self.clipboard {
            Some(c) => c,
            None => {
                self.clipboard = Clipboard::new().ok();
                match &mut self.clipboard {
                    Some(c) => c,
                    None => return false,
                }
            }
        };
        
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let content = ClipboardContent::Text(text);
                let new_hash = compute_hash(&content);
                
                // Quick hash check first
                if new_hash != self.last_hash {
                    return self.add_item(content);
                }
            }
        }
        
        false
    }

    pub fn paste_item(&mut self, id: &str) -> Result<(), String> {
        let item = self.items.iter()
            .find(|i| i.id == id)
            .ok_or("Item not found")?;
        
        // Re-create clipboard for paste
        let clipboard = self.clipboard.as_mut()
            .ok_or("Clipboard not available")?;
        
        match &item.content {
            ClipboardContent::Text(text) => {
                clipboard.set_text(text.clone()).map_err(|e| e.to_string())?;
            }
        }
        
        // Update last hash to prevent re-adding what we just pasted
        self.last_hash = item.content_hash;
        
        Ok(())
    }
    
    /// Refresh clipboard connection (for X11 issues)
    pub fn refresh_clipboard(&mut self) {
        self.clipboard = Clipboard::new().ok();
    }
}

/// Thread-safe wrapper with fast RwLock
pub struct SharedClipboardManager(pub RwLock<ClipboardManager>);

impl SharedClipboardManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(RwLock::new(ClipboardManager::new())))
    }
}