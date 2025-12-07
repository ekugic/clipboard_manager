use crate::models::{ClipboardItem, ClipboardContent};
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBoxRow, Orientation, Image, Align};

pub fn create_list_row(item: &ClipboardItem) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("clipboard-item");
    
    unsafe {
        row.set_data("item_id", item.id.clone());
    }
    
    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    
    // --- Content Area (Now FIRST) ---
    match &item.content {
        ClipboardContent::Text(text) => {
            let vbox = Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            
            // FIX: Use char_indices() to respect UTF-8 boundaries
            let preview = if text.len() > 150 {
                let mut end_idx = 150;
                // Find the last valid char boundary before 150 bytes
                for (idx, _) in text.char_indices() {
                    if idx > 150 {
                        break;
                    }
                    end_idx = idx;
                }
                format!("{}...", &text[..end_idx])
            } else {
                text.clone()
            };
            
            let label = Label::new(Some(&preview));
            label.set_xalign(0.0);
            label.set_wrap(true);
            label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            label.set_max_width_chars(50);
            label.add_css_class("item-text");
            vbox.append(&label);
            
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            timestamp_label.set_xalign(0.0);
            vbox.append(&timestamp_label);
            
            hbox.append(&vbox);
        }
    }
    
    // --- Pin Button (Now SECOND) ---
    let pin_button = Button::new();
    pin_button.add_css_class("pin-button");
    pin_button.add_css_class("flat");
    
    pin_button.set_valign(Align::Center); 
    pin_button.set_halign(Align::Center);

    let icon_name = "view-pin-symbolic";
    let pin_icon = Image::from_icon_name(icon_name);
    pin_button.set_child(Some(&pin_icon));

    if item.pinned {
        pin_button.add_css_class("pinned");
    } else {
        pin_button.add_css_class("unpinned"); 
    }
    
    let row_weak = row.downgrade();
    pin_button.connect_clicked(move |_| {
        if let Some(row) = row_weak.upgrade() {
            unsafe {
                row.set_data("is_pin_click", true);
            }
            row.activate();
        }
    });
    
    hbox.append(&pin_button);
    
    row.set_child(Some(&hbox));
    row
}