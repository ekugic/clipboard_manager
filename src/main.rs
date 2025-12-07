mod clipboard;
mod storage;
mod ui;
mod models;

use gtk4::prelude::*; 
use libadwaita as adw;
use std::cell::RefCell;

const APP_ID: &str = "com.example.ClipboardManager";

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    
    let window_ref: RefCell<Option<adw::ApplicationWindow>> = RefCell::new(None);
    
    app.connect_command_line(|app, _| {
        app.activate();
        0
    });
    
    app.connect_activate(move |app| {
        let mut window_opt = window_ref.borrow_mut();
        
        let window = if let Some(win) = window_opt.as_ref() {
            win.clone()
        } else {
            let win = ui::window::build_ui(app);
            *window_opt = Some(win.clone());
            win
        };
        
        if window.is_visible() {
            window.set_visible(false);
        } else {
            window.present();
        }
    });

    app.run();
}