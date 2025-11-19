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
use std::time::Duration;

pub fn build_ui(app: &adw::Application) -> adw::ApplicationWindow {
    let manager = Arc::new(Mutex::new(ClipboardManager::new()));
    let manager_clone = Arc::clone(&manager);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(450)
        .default_height(600)
        .decorated(false)
        .resizable(false)
        .build();

    apply_styles();

    // Prevent the window from being destroyed when the user clicks 'X' or closes it
    window.connect_close_request(|win| {
        win.set_visible(false); // Replaces win.hide()
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

    let list_box_ref = Arc::new(Mutex::new(list_box.clone()));
    let list_box_clone = Arc::clone(&list_box_ref);

    {
        let mgr = manager.lock().unwrap();
        refresh_list(mgr.get_items(), &list_box);
    }

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
                
                // HIDE instead of CLOSE
                window_clone.set_visible(false); 
            }
        }
    });

    main_box.append(&scrolled_window);
    window.set_content(Some(&main_box));

    let key_controller = EventControllerKey::new();
    let window_clone = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_clone.set_visible(false); // HIDE
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    window.connect_is_active_notify(move |win| {
        if !win.is_active() {
            win.set_visible(false); // HIDE
        }
    });

    glib::timeout_add_local(Duration::from_millis(500), move || {
        let mut mgr = manager_clone.lock().unwrap();
        if mgr.check_clipboard().is_some() {
            let list = list_box_clone.lock().unwrap();
            refresh_list(mgr.get_items(), &list);
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