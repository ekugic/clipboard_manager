use gtk4::CssProvider;

pub fn apply_styles() {
    let css = CssProvider::new();
    css.load_from_string(
        r#"
        .popup-window {
            background: @theme_bg_color;
            border-radius: 12px;
            border: 1px solid alpha(@theme_fg_color, 0.2);
        }
        
        .popup-list {
            background: transparent;
        }
        
        .clipboard-item {
            padding: 6px;
            margin: 2px 4px; 
            border-radius: 6px;
            transition: background 150ms ease;
        }
        
        .clipboard-item:hover {
            background: alpha(@theme_fg_color, 0.05);
        }
        
        .clipboard-item:active {
            background: alpha(@theme_fg_color, 0.1);
        }
        
        /* Pin Button Updates */
        .pin-button {
            min-width: 32px;   /* Fixed width */
            min-height: 32px;  /* Fixed height */
            padding: 0;        /* Remove padding to prevent oval shapes */
            border-radius: 999px; /* Perfect circle */
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
            background: alpha(@theme_selected_bg_color, 0.1);
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