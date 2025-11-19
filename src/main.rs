mod clipboard;
mod storage;
mod ui;
mod models;

use gtk4::prelude::*; // Essential for GtkApplicationExt and WidgetExt
use libadwaita as adw;

const APP_ID: &str = "com.example.ClipboardManager";

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // 1. Handle command line arguments.
    // When you trigger your shortcut, it runs the command again.
    // This signal ensures the existing instance handles the request.
    app.connect_command_line(|app, _| {
        app.activate();
        0
    });

    // 2. Handle activation (Launch or Toggle)
    app.connect_activate(move |app| {
        // Instead of a global static, we ask the application for its windows.
        // If the window is hidden, it is STILL in this list.
        if let Some(window) = app.windows().first() {
            // Window exists! Toggle visibility.
            if window.is_visible() {
                window.set_visible(false);
            } else {
                window.present();
            }
        } else {
            // No window found (First run). Create it.
            let window = ui::window::build_ui(app);
            window.present();
        }
    });

    app.run();
}