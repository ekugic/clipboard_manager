use gtk4::CssProvider;
use gtk4::prelude::*;

pub fn apply_styles() {
    let css = CssProvider::new();
    css.load_from_string(
        r#"
        /* Main Container - Minimalist Box */
        .popup-window {
            background: @theme_bg_color;
            border-radius: 12px;
            border: 1px solid alpha(@theme_fg_color, 0.2);
        }
        
        .popup-list {
            background: transparent;
        }
        
        /* List Items */
        .clipboard-item {
            padding: 6px;
            margin: 2px 4px; /* Tighter spacing */
            border-radius: 6px;
            transition: background 150ms ease;
        }
        
        .clipboard-item:hover {
            background: alpha(@theme_fg_color, 0.05);
        }
        
        .clipboard-item:active {
            background: alpha(@theme_fg_color, 0.1);
        }
        
        /* Pin Button */
        .pin-button {
            padding: 8px;
            border-radius: 100%;
            opacity: 0.3;
            transition: all 200ms;
        }
        
        .pin-button:hover {
            opacity: 1.0;
            background: alpha(@theme_fg_color, 0.1);
        }
        
        .pin-button.pinned {
            opacity: 1.0;
            color: @theme_selected_bg_color;
        }
        
        .item-text {
            font-size: 13px;
            color: @theme_fg_color;
        }
        
        .timestamp {
            font-size: 11px;
            margin-top: 2px;
            opacity: 0.5;
        }
        "#
    );
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}