mod clipboard;
mod storage;
mod ui;
mod models;

use gtk4::prelude::*; 
use libadwaita as adw;

const APP_ID: &str = "com.example.ClipboardManager";

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    
    app.connect_command_line(|app, _| {
        app.activate();
        0
    });

    
    app.connect_activate(move |app| {
        
        if let Some(window) = app.windows().first() {
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.present();
            }
        } else {
            let window = ui::window::build_ui(app);
            window.present();
        }
    });

    app.run();
}