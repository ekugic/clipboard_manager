use gtk4::{gdk, CssProvider};

pub fn apply_styles() {
    let css = CssProvider::new();
    css.load_from_string(
        r#"
        .popup-window {
            background: @theme_bg_color;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
            border: 1px solid alpha(@theme_fg_color, 0.1);
        }
        
        .popup-header {
            background: linear-gradient(to bottom, 
                alpha(@theme_selected_bg_color, 0.1),
                transparent);
            padding: 4px 8px;
        }
        
        .popup-title {
            font-size: 1.1em;
            font-weight: 600;
        }
        
        .popup-separator {
            background: alpha(@theme_fg_color, 0.1);
            min-height: 1px;
        }
        
        .popup-list {
            background: transparent;
        }
        
        .clipboard-item {
            padding: 4px;
            margin: 4px 8px;
            border-radius: 8px;
            transition: background 150ms ease;
        }
        
        .clipboard-item:hover {
            background: alpha(@theme_fg_color, 0.08);
        }
        
        .clipboard-item:active {
            background: alpha(@theme_fg_color, 0.15);
        }
        
        .pin-button {
            min-width: 32px;
            min-height: 32px;
            padding: 4px;
            border-radius: 50%;
            opacity: 0.6;
            transition: all 150ms ease;
        }
        
        .pin-button:hover {
            opacity: 1.0;
            background: alpha(@theme_fg_color, 0.08);
        }
        
        .pin-button.pinned {
            opacity: 1.0;
            color: @theme_selected_bg_color;
        }
        
        .item-text {
            color: @theme_fg_color;
        }
        
        .timestamp {
            color: alpha(@theme_fg_color, 0.5);
            font-size: 0.85em;
        }
        "#
    );

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}