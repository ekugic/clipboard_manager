use crate::models::{ClipboardContent, ClipboardItem, MAX_ITEMS, MAX_TEXT_SIZE, MAX_IMAGE_SIZE, THUMBNAIL_SIZE};
use crate::storage::Storage;
use arboard::{Clipboard, ImageData};
use parking_lot::RwLock;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::borrow::Cow;

fn compute_hash(content: &ClipboardContent) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Create a thumbnail from RGBA data
fn create_thumbnail(rgba_data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    use image::{RgbaImage, DynamicImage, imageops::FilterType};
    
    let img = RgbaImage::from_raw(width, height, rgba_data.to_vec())?;
    let dynamic_img = DynamicImage::ImageRgba8(img);
    
    // Calculate thumbnail size maintaining aspect ratio
    let (thumb_w, thumb_h) = if width > height {
        let ratio = height as f32 / width as f32;
        (THUMBNAIL_SIZE, (THUMBNAIL_SIZE as f32 * ratio) as u32)
    } else {
        let ratio = width as f32 / height as f32;
        ((THUMBNAIL_SIZE as f32 * ratio) as u32, THUMBNAIL_SIZE)
    };
    
    let thumbnail = dynamic_img.resize(thumb_w.max(1), thumb_h.max(1), FilterType::Triangle);
    
    // Encode to PNG
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    thumbnail.write_to(&mut cursor, image::ImageOutputFormat::Png).ok()?;
    
    Some(png_bytes)
}

/// Convert RGBA to PNG
fn rgba_to_png(rgba_data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    use image::{RgbaImage, DynamicImage};
    
    let img = RgbaImage::from_raw(width, height, rgba_data.to_vec())?;
    let dynamic_img = DynamicImage::ImageRgba8(img);
    
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    dynamic_img.write_to(&mut cursor, image::ImageOutputFormat::Png).ok()?;
    
    Some(png_bytes)
}

/// Convert PNG back to RGBA for clipboard
fn png_to_rgba(png_data: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    use image::io::Reader as ImageReader;
    use std::io::Cursor;
    
    let img = ImageReader::new(Cursor::new(png_data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    
    Some((rgba.into_raw(), width, height))
}

pub struct ClipboardManager {
    items: Vec<ClipboardItem>,
    last_text_hash: u64,
    last_image_hash: u64,
    storage: Storage,
    clipboard: Option<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let storage = Storage::new();
        let items = storage.load_items();
        let clipboard = Clipboard::new().ok();
        
        Self {
            items,
            last_text_hash: 0,
            last_image_hash: 0,
            storage,
            clipboard,
        }
    }

    pub fn add_item(&mut self, content: ClipboardContent) -> bool {
        let new_hash = compute_hash(&content);
        
        // Check if duplicate based on content type
        let is_duplicate = match &content {
            ClipboardContent::Text(_) => new_hash == self.last_text_hash,
            ClipboardContent::Image { .. } => new_hash == self.last_image_hash,
        };
        
        if is_duplicate {
            return false;
        }

        // Check size
        let size = match &content {
            ClipboardContent::Text(text) => text.len(),
            ClipboardContent::Image { png_data, .. } => png_data.len(),
        };

        let max_size = match &content {
            ClipboardContent::Text(_) => MAX_TEXT_SIZE,
            ClipboardContent::Image { .. } => MAX_IMAGE_SIZE,
        };

        if size > max_size || size == 0 {
            return false;
        }

        // Remove duplicate if exists (but not if pinned)
        self.items.retain(|existing| {
            existing.pinned || existing.content_hash != new_hash
        });

        let item = ClipboardItem::new(content.clone());
        
        // Find position after pinned items
        let pinned_count = self.items.iter().filter(|i| i.pinned).count();
        self.items.insert(pinned_count, item);

        // Keep only MAX_ITEMS
        let mut non_pinned_count = 0;
        self.items.retain(|item| {
            if item.pinned {
                true
            } else {
                non_pinned_count += 1;
                non_pinned_count <= MAX_ITEMS
            }
        });

        // Update appropriate hash
        match &content {
            ClipboardContent::Text(_) => self.last_text_hash = new_hash,
            ClipboardContent::Image { .. } => self.last_image_hash = new_hash,
        }
        
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

    /// Check clipboard for text and images
    pub fn check_clipboard_fast(&mut self) -> bool {
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
        
        // Try to get image first (usually what user wants to capture)
        if let Ok(img) = clipboard.get_image() {
            let width = img.width as u32;
            let height = img.height as u32;
            let rgba_data: Vec<u8> = img.bytes.into_owned();
            
            // Quick hash check on raw data
            let mut hasher = DefaultHasher::new();
            width.hash(&mut hasher);
            height.hash(&mut hasher);
            rgba_data.len().hash(&mut hasher);
            if rgba_data.len() > 0 {
                rgba_data[..rgba_data.len().min(1024)].hash(&mut hasher);
            }
            let quick_hash = hasher.finish();
            
            if quick_hash != self.last_image_hash {
                // Convert to PNG and create thumbnail (done in background-ish)
                if let (Some(png_data), Some(thumbnail)) = (
                    rgba_to_png(&rgba_data, width, height),
                    create_thumbnail(&rgba_data, width, height)
                ) {
                    let content = ClipboardContent::Image {
                        png_data,
                        thumbnail_png: thumbnail,
                        width,
                        height,
                    };
                    return self.add_item(content);
                }
            }
        }
        
        // Try text
        if let Ok(text) = clipboard.get_text() {
            if !text.is_empty() {
                let content = ClipboardContent::Text(text);
                let new_hash = compute_hash(&content);
                
                if new_hash != self.last_text_hash {
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
        
        let clipboard = self.clipboard.as_mut()
            .ok_or("Clipboard not available")?;
        
        match &item.content {
            ClipboardContent::Text(text) => {
                clipboard.set_text(text.clone()).map_err(|e| e.to_string())?;
                self.last_text_hash = item.content_hash;
            }
            ClipboardContent::Image { png_data, width, height, .. } => {
                // Convert PNG back to RGBA for clipboard
                if let Some((rgba, w, h)) = png_to_rgba(png_data) {
                    let img_data = ImageData {
                        width: w as usize,
                        height: h as usize,
                        bytes: Cow::Owned(rgba),
                    };
                    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
                    self.last_image_hash = item.content_hash;
                } else {
                    return Err("Failed to decode image".to_string());
                }
            }
        }
        
        Ok(())
    }
    
    pub fn refresh_clipboard(&mut self) {
        self.clipboard = Clipboard::new().ok();
    }
    
    pub fn delete_item(&mut self, id: &str) {
        self.items.retain(|item| item.id != id);
        self.storage.save_items_async(&self.items);
    }
}

pub struct SharedClipboardManager(pub RwLock<ClipboardManager>);

impl SharedClipboardManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(RwLock::new(ClipboardManager::new())))
    }
}