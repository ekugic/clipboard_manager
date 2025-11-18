use gtk4::prelude::*;
use gtk4::{
    glib, Box, Label, ListBox,
    ListBoxRow, Orientation, ScrolledWindow, SelectionMode,
    PolicyType, EventControllerKey, gdk, CssProvider,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use dirs;
use std::fs;
use std::path::PathBuf;
use chrono::Local;

const MAX_ITEMS: usize = 25;
const MAX_ITEM_SIZE: usize = 4 * 1024 * 1024; // 4MB
const APP_ID: &str = "com.example.ClipboardManager";

#[derive(Clone, Debug, Serialize, Deserialize)]
enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClipboardItem {
    content: ClipboardContent,
    timestamp: String,
}

struct ClipboardManager {
    items: Vec<ClipboardItem>,
    last_content: Option<ClipboardContent>,
}

impl ClipboardManager {
    fn new() -> Self {
        let items = Self::load_items().unwrap_or_default();
        Self {
            items,
            last_content: None,
        }
    }

    fn add_item(&mut self, content: ClipboardContent) {
        // Check if it's the same as the last item
        if let Some(last) = &self.last_content {
            if self.content_equals(last, &content) {
                return;
            }
        }

        // Check size
        let size = match &content {
            ClipboardContent::Text(text) => text.len(),
            ClipboardContent::Image(data) => data.len(),
        };

        if size > MAX_ITEM_SIZE {
            return;
        }

        let item = ClipboardItem {
            content: content.clone(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
        };

        // Remove duplicate if exists - fixed borrow checker issue
        let mut indices_to_remove = Vec::new();
        for (i, existing) in self.items.iter().enumerate() {
            if self.content_equals(&existing.content, &content) {
                indices_to_remove.push(i);
            }
        }
        for i in indices_to_remove.iter().rev() {
            self.items.remove(*i);
        }

        // Add new item at the beginning
        self.items.insert(0, item);

        // Keep only MAX_ITEMS
        if self.items.len() > MAX_ITEMS {
            self.items.truncate(MAX_ITEMS);
        }

        self.last_content = Some(content);
        let _ = self.save_items();
    }

    fn content_equals(&self, a: &ClipboardContent, b: &ClipboardContent) -> bool {
        match (a, b) {
            (ClipboardContent::Text(t1), ClipboardContent::Text(t2)) => t1 == t2,
            (ClipboardContent::Image(i1), ClipboardContent::Image(i2)) => i1 == i2,
            _ => false,
        }
    }

    fn get_data_dir() -> PathBuf {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("clipboard_manager");
        let _ = fs::create_dir_all(&path);
        path
    }

    fn save_items(&self) -> std::io::Result<()> {
        let mut path = Self::get_data_dir();
        path.push("clipboard_history.json");
        let json = serde_json::to_string(&self.items)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load_items() -> std::io::Result<Vec<ClipboardItem>> {
        let mut path = Self::get_data_dir();
        path.push("clipboard_history.json");
        if path.exists() {
            let json = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&json).unwrap_or_default())
        } else {
            Ok(Vec::new())
        }
    }
}

fn build_ui(app: &adw::Application) {
    let manager = Arc::new(Mutex::new(ClipboardManager::new()));
    let manager_clone = Arc::clone(&manager);

    // Create main window using libadwaita ApplicationWindow
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Clipboard Manager")
        .default_width(400)
        .default_height(500)
        .build();

    // Apply CSS for modern GNOME theme
    let css = CssProvider::new();
    css.load_from_string(
        "
        .clipboard-window {
            background: @theme_bg_color;
            border-radius: 12px;
        }
        .clipboard-item {
            padding: 12px;
            border-radius: 8px;
            margin: 4px 8px;
        }
        .clipboard-item:hover {
            background: alpha(@theme_fg_color, 0.08);
        }
        .clipboard-item:active {
            background: alpha(@theme_fg_color, 0.12);
        }
        .timestamp {
            color: alpha(@theme_fg_color, 0.6);
            font-size: 0.9em;
        }
        "
    );

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Create main container
    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("clipboard-window");

    // Header
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("Clipboard History"))));
    main_box.append(&header);

    // Create scrolled window
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .build();

    // Create list box
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::Single);
    scrolled_window.set_child(Some(&list_box));

    // Populate list with items
    let manager_locked = manager.lock().unwrap();
    for item in &manager_locked.items {
        let row = create_list_row(item);
        list_box.append(&row);
    }
    drop(manager_locked);

    // Handle row activation (click)
    let window_clone = window.clone();
    list_box.connect_row_activated(move |_, row| {
        let index = row.index();
        if index >= 0 {
            let manager_locked = manager.lock().unwrap();
            if let Some(item) = manager_locked.items.get(index as usize) {
                let mut clipboard = Clipboard::new().unwrap();
                match &item.content {
                    ClipboardContent::Text(text) => {
                        let _ = clipboard.set_text(text.clone());
                    }
                    ClipboardContent::Image(_data) => {
                        // For images, we'll just show a message for now
                        // Full image clipboard support would require more complex handling
                        println!("Image clipboard paste not fully implemented");
                    }
                }
                
                // Close window after selection
                window_clone.close();
            }
        }
    });

    main_box.append(&scrolled_window);
    window.set_content(Some(&main_box));

    // Set up keyboard shortcut (Escape to close)
    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_clone.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    // Start clipboard monitor in background
    let window_weak = window.downgrade();
    glib::timeout_add_local(Duration::from_millis(500), move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut manager_locked = manager_clone.lock().unwrap();
        
        // Check for new text
        if let Ok(text) = clipboard.get_text() {
            let should_add = if let Some(ClipboardContent::Text(last_text)) = &manager_locked.last_content {
                &text != last_text
            } else {
                true
            };
            
            if should_add && !text.is_empty() {
                manager_locked.add_item(ClipboardContent::Text(text));
                
                // Update UI if window still exists
                if let Some(window) = window_weak.upgrade() {
                    if let Some(content) = window.content() {
                        if let Some(_main_box) = content.downcast_ref::<Box>() {
                            // Find and update the list box
                            // This is a simplified version - in production you'd want better state management
                        }
                    }
                }
            }
        }
        
        glib::ControlFlow::Continue
    });

    window.present();
}

fn create_list_row(item: &ClipboardItem) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("clipboard-item");
    
    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    
    match &item.content {
        ClipboardContent::Text(text) => {
            let vbox = Box::new(Orientation::Vertical, 4);
            
            // Text preview (truncated)
            let preview = if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text.clone()
            };
            let label = Label::new(Some(&preview));
            label.set_xalign(0.0);
            label.set_wrap(true);
            label.set_max_width_chars(50);
            vbox.append(&label);
            
            // Timestamp
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            timestamp_label.set_xalign(0.0);
            vbox.append(&timestamp_label);
            
            hbox.append(&vbox);
        }
        ClipboardContent::Image(_data) => {
            let label = Label::new(Some("📷 Image"));
            label.set_xalign(0.0);
            hbox.append(&label);
            
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            hbox.append(&timestamp_label);
        }
    }
    
    row.set_child(Some(&hbox));
    row
}

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}




