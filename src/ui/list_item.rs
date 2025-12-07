use crate::models::{ClipboardContent, ClipboardItem};
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBoxRow, Orientation, Image, Align, Picture};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::glib::Bytes;

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
        ClipboardContent::Image { thumbnail_png, width, height, .. } => {
            let vbox = Box::new(Orientation::Vertical, 4);
            vbox.set_hexpand(true);
            
            // Create image from thumbnail PNG
            let image_widget = create_image_from_png(thumbnail_png);
            image_widget.set_halign(Align::Start);
            image_widget.add_css_class("thumbnail");
            vbox.append(&image_widget);
            
            // Show dimensions and timestamp
            let info_box = Box::new(Orientation::Horizontal, 8);
            
            let dim_label = Label::new(Some(&format!("{}×{}", width, height)));
            dim_label.add_css_class("image-dimensions");
            info_box.append(&dim_label);
            
            let timestamp_label = Label::new(Some(&item.timestamp));
            timestamp_label.add_css_class("timestamp");
            info_box.append(&timestamp_label);
            
            vbox.append(&info_box);
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

fn create_image_from_png(png_data: &[u8]) -> Picture {
    let picture = Picture::new();
    
    // Load PNG data into Pixbuf
    let bytes = Bytes::from(png_data);
    let stream = gtk4::gio::MemoryInputStream::from_bytes(&bytes);
    
    if let Ok(pixbuf) = Pixbuf::from_stream(&stream, gtk4::gio::Cancellable::NONE) {
        let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
        picture.set_paintable(Some(&texture));
    }
    
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(true);
    picture.set_content_fit(gtk4::ContentFit::Contain);
    
    picture
}

#[inline]
fn truncate_string(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        // Also clean up excessive whitespace/newlines for preview
        s.lines()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(max_chars)
            .collect()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated.trim())
    }
}