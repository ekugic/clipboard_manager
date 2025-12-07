use crate::clipboard::SharedClipboardManager;
use crate::models::ClipboardItem;
use crate::ui::list_item::create_list_row;
use crate::ui::styles::apply_styles;
use gtk4::prelude::*;
use gtk4::{
    glib, Box, ListBox, Orientation, ScrolledWindow, 
    SelectionMode, PolicyType, EventControllerKey, gdk,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crossbeam_channel::{bounded, Sender, Receiver};

enum UiMessage {
    ItemsChanged(Vec<ClipboardItem>),
}

pub fn build_ui(app: &adw::Application) -> adw::ApplicationWindow {
    let manager = SharedClipboardManager::new();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(450)
        .default_height(600)
        .decorated(false)
        .resizable(false)
        .build();

    apply_styles();

    window.connect_close_request(|win| {
        win.set_visible(false);
        glib::Propagation::Stop
    });

    let main_box = Box::new(Orientation::Vertical, 0);
    main_box.add_css_class("popup-window");
    main_box.set_margin_top(6);
    main_box.set_margin_bottom(6);
    main_box.set_margin_start(6);
    main_box.set_margin_end(6);

    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::None);
    list_box.add_css_class("popup-list");
    scrolled_window.set_child(Some(&list_box));

    // Initial load
    {
        let mgr = manager.0.read();
        refresh_list(mgr.get_items(), &list_box);
    }

    // Click handling
    let window_clone = window.clone();
    let manager_click = Arc::clone(&manager);
    
    list_box.connect_row_activated(move |list, row| {
        let item_id = unsafe { row.data::<String>("item_id") };
        
        if let Some(id_ptr) = item_id {
            let id_str = unsafe { id_ptr.as_ref() }.to_string();
            let is_pin_click = unsafe { row.data::<bool>("is_pin_click") }.is_some();

            if is_pin_click {
                let mut mgr = manager_click.0.write();
                mgr.toggle_pin(&id_str);
                refresh_list(mgr.get_items(), list);
            } else {
                let mut mgr = manager_click.0.write();
                let _ = mgr.paste_item(&id_str);
                drop(mgr);
                window_clone.set_visible(false); 
            }
        }
    });

    main_box.append(&scrolled_window);
    window.set_content(Some(&main_box));

    // Keyboard handling
    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_clone.set_visible(false);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    window.connect_is_active_notify(move |win| {
        if !win.is_active() {
            win.set_visible(false);
        }
    });

    // Background polling
    let (sender, receiver): (Sender<UiMessage>, Receiver<UiMessage>) = bounded(4);
    let manager_thread = Arc::clone(&manager);

    thread::spawn(move || {
        let mut refresh_counter = 0u32;
        
        loop {
            // 25ms polling - fast but not too aggressive
            thread::sleep(Duration::from_millis(25));
            
            let mut mgr = manager_thread.0.write();
            
            if mgr.check_clipboard_fast() {
                let items = mgr.get_items().to_vec();
                let _ = sender.try_send(UiMessage::ItemsChanged(items));
            }
            
            // Refresh clipboard connection every ~5 seconds
            refresh_counter += 1;
            if refresh_counter >= 200 {
                refresh_counter = 0;
                mgr.refresh_clipboard();
            }
        }
    });

    // UI update receiver
    let (glib_sender, glib_receiver) = glib::MainContext::channel(glib::Priority::HIGH);
    
    thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            let _ = glib_sender.send(msg);
        }
    });

    let list_box_clone = list_box.clone();
    glib_receiver.attach(None, move |msg| {
        match msg {
            UiMessage::ItemsChanged(items) => {
                refresh_list(&items, &list_box_clone);
            }
        }
        glib::ControlFlow::Continue
    });

    window
}

fn refresh_list(items: &[ClipboardItem], list_box: &ListBox) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    
    for item in items {
        let row = create_list_row(item);
        list_box.append(&row);
    }
}