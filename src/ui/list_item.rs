use crate::models::{ClipboardItem, ClipboardContent};
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBoxRow, Orientation, Image};

// ensure 'pub' is here
pub fn create_list_row(item: &ClipboardItem) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("clipboard-item");
    
    // Store item ID as data (unsafe)
    unsafe {
        row.set_data("item_id", item.id.clone());
    }
    
    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    
    // --- Pin Button ---
    let pin_button = Button::new();
    pin_button.add_css_class("pin-button");
    pin_button.add_css_class("flat");
    
    let icon_name = if item.pinned { "view-pin-symbolic" } else { "view-pin-symbolic" }; 
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
    
    // --- Content Area ---
    match &item.content {
        ClipboardContent::Text(text) => {
            let vbox = Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            
            let preview = if text.len() > 150 {
                format!("{}...", &text[..150])
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
        ClipboardContent::Image(_) => {
            let vbox = Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            
            let content_row = Box::new(Orientation::Horizontal, 8);
            let img_icon = Image::from_icon_name("image-x-generic-symbolic");
            content_row.append(&img_icon);

            let label = Label::new(Some("Image"));
            label.set_xalign(0.0);
            label.add_css_class("item-text");
            content_row.append(&label);
            
            vbox.append(&content_row);
            
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            timestamp_label.set_xalign(0.0);
            vbox.append(&timestamp_label);
            
            hbox.append(&vbox);
        }
    }
    
    row.set_child(Some(&hbox));
    row
}