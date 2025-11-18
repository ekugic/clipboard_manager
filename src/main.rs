mod clipboard;
mod storage;
mod ui;
mod models;

use gtk4::prelude::*;
use libadwaita as adw;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

const APP_ID: &str = "com.example.ClipboardManager";

static WINDOW_VISIBLE: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Handle command line to ensure single instance
    app.connect_command_line(|app, _| {
        app.activate();
        0
    });

    app.connect_activate(move |app| {
        let mut visible = WINDOW_VISIBLE.lock().unwrap();
        
        if *visible {
            // Window already exists, just toggle it
            if let Some(window) = app.active_window() {
                window.close();
                *visible = false;
            }
        } else {
            // Create new window
            ui::window::build_ui(app);
            *visible = true;
        }
    });

    app.run();
}