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
    
    match &item.content {
        ClipboardContent::Text(text) => {
            let vbox = Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            
            // Fast preview with proper UTF-8 handling
            let preview = truncate_string(text, 150);
            
            let label = Label::new(Some(&preview));
            label.set_xalign(0.0);
            label.set_wrap(true);
            label.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            label.set_max_width_chars(50);
            label.set_lines(3);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.add_css_class("item-text");
            vbox.append(&label);
            
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            timestamp_label.set_xalign(0.0);
            vbox.append(&timestamp_label);
            
            hbox.append(&vbox);
        }
    }
    
    // Pin Button
    let pin_button = Button::new();
    pin_button.add_css_class("pin-button");
    pin_button.add_css_class("flat");
    pin_button.set_valign(Align::Center); 
    pin_button.set_halign(Align::Center);

    let pin_icon = Image::from_icon_name("view-pin-symbolic");
    pin_button.set_child(Some(&pin_icon));

    if item.pinned {
        pin_button.add_css_class("pinned");
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

#[inline]
fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}