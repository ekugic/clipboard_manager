use crate::clipboard::ClipboardManager;
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
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn build_ui(app: &adw::Application) -> adw::ApplicationWindow {
    // Arc<Mutex<>> is still useful for sharing state between UI callbacks
    let manager = Arc::new(Mutex::new(ClipboardManager::new()));

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

    // Initial Load
   // In build_ui(), change initial load:
    {
        let mgr = manager.lock().unwrap();
        let items = mgr.get_items();
        let recent_items = if items.len() > 15 {
            &items[..15]  // Only show 15 most recent on startup
        } else {
            items
        };
        refresh_list(recent_items, &list_box);
    }

    
    // --- CLICK HANDLING ---
    let window_clone = window.clone();
    let manager_click = Arc::clone(&manager);
    
    list_box.connect_row_activated(move |list, row| {
        let item_id = unsafe { row.data::<String>("item_id") };
        
        if let Some(id_ptr) = item_id {
            let id_str = unsafe { id_ptr.as_ref() }.to_string();
            let is_pin_click = unsafe { row.data::<bool>("is_pin_click") }.is_some();

            if is_pin_click {
                let mut mgr = manager_click.lock().unwrap();
                mgr.toggle_pin(&id_str);
                refresh_list(mgr.get_items(), list);
            } else {
                let mgr = manager_click.lock().unwrap();
                let _ = mgr.paste_item(&id_str);
                drop(mgr);
                window_clone.set_visible(false); 
            }
        }
    });

    main_box.append(&scrolled_window);
    window.set_content(Some(&main_box));

    // --- KEYBOARD HANDLING ---
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

    // --- BACKGROUND THREAD FOR CLIPBOARD POLLING ---
    // We use a channel to communicate from the background thread to the UI thread.
    let (sender, receiver) = glib::MainContext::channel(glib::Priority::DEFAULT);
    let manager_thread = Arc::clone(&manager);

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            
            let mut mgr = manager_thread.lock().unwrap();
            // This check happens in the background. Heavy I/O here won't freeze UI.
            if mgr.check_clipboard().is_some() {
                // If we found something, send a signal to the UI thread
                let items = mgr.get_items().to_vec(); // Clone items to send across threads
                let _ = sender.send(items);
            }
        }
    });

    // Listen for updates on the UI thread
    let list_box_clone = list_box.clone();
    receiver.attach(None, move |items| {
        refresh_list(&items, &list_box_clone);
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