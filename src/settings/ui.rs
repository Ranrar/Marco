use gtk4::prelude::*;
use gtk4::{Button, HeaderBar, Image, Window, ApplicationWindow};

/// Create a settings button with burger icon for the header bar
pub fn create_settings_button() -> Button {
    let button = Button::new();
    
    // Try to use system hamburger icon, fallback to Unicode
    let icon_theme = gtk4::IconTheme::for_display(&gtk4::gdk::Display::default().unwrap());
    let icon = icon_theme.lookup_icon("open-menu-symbolic", &[], 16, 1, gtk4::TextDirection::None, gtk4::IconLookupFlags::empty());
    
    let image = Image::from_paintable(Some(&icon));
    button.set_child(Some(&image));
    
    button.set_tooltip_text(Some("Settings"));
    // Use the same styling as toolbar buttons - remove "flat" class to get default button styling
    
    button
}

/// Add settings button to header bar
pub fn add_settings_button_to_header_bar(
    header_bar: &HeaderBar, 
    parent_window: &ApplicationWindow,
    editor: &crate::editor::MarkdownEditor,
    theme_manager: &crate::theme::ThemeManager,
) {
    let settings_button = create_settings_button();
    
    // Connect button to show settings dialog
    settings_button.connect_clicked({
        let parent_window = parent_window.clone();
        let editor = editor.clone();
        let theme_manager = theme_manager.clone();
        move |_| {
            let window = parent_window.upcast_ref::<Window>();
            crate::settings::dialog::show_settings_dialog(window, &editor, &theme_manager);
        }
    });
    
    // Add button to the left side of header bar (before minimize/maximize/close)
    header_bar.pack_end(&settings_button);
}

/// Create a styled section header for settings pages
pub fn create_settings_section_header(title: &str, description: Option<&str>) -> gtk4::Box {
    let section_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    
    let title_label = gtk4::Label::new(Some(title));
    title_label.set_halign(gtk4::Align::Start);
    title_label.add_css_class("heading");
    section_box.append(&title_label);
    
    if let Some(desc) = description {
        let desc_label = gtk4::Label::new(Some(desc));
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.add_css_class("dim-label");
        desc_label.set_wrap(true);
        section_box.append(&desc_label);
    }
    
    section_box.set_margin_bottom(8);
    section_box
}

/// Create a settings row with label and control
pub fn create_settings_row(
    label: &str,
    control: &impl IsA<gtk4::Widget>,
    description: Option<&str>,
) -> gtk4::Box {
    let row_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    main_box.set_homogeneous(false);
    
    let label_widget = gtk4::Label::new(Some(label));
    label_widget.set_halign(gtk4::Align::Start);
    label_widget.set_hexpand(true);
    main_box.append(&label_widget);
    
    control.set_halign(gtk4::Align::End);
    main_box.append(control);
    
    row_box.append(&main_box);
    
    if let Some(desc) = description {
        let desc_label = gtk4::Label::new(Some(desc));
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.add_css_class("dim-label");
        desc_label.set_wrap(true);
        desc_label.set_margin_top(4);
        row_box.append(&desc_label);
    }
    
    row_box.set_margin_bottom(8);
    row_box
}

/// Apply settings-specific CSS
pub fn apply_settings_css() {
    let provider = gtk4::CssProvider::new();
    
    // Essential settings styling
    let css_content = "
        .settings-dialog .heading {
            font-size: 1.1em;
            font-weight: bold;
            margin-bottom: 4px;
        }
        
        .settings-dialog .dim-label {
            opacity: 0.7;
            font-size: 0.9em;
        }
        
        .settings-dialog .section-separator {
            margin-top: 12px;
            margin-bottom: 12px;
        }
        
        .settings-dialog notebook {
            border: 1px solid rgba(0,0,0,0.1);
            border-radius: 8px;
        }
        
        .settings-dialog notebook tab {
            padding: 8px 16px;
        }
        
        .settings-dialog .settings-page {
            padding: 16px;
        }
        
        .settings-dialog .destructive-action {
            background-color: #e74c3c;
            color: white;
        }
        
        .settings-dialog .destructive-action:hover {
            background-color: #c0392b;
        }
        
        .settings-dialog switch {
            margin-left: 12px;
        }
        
        .settings-dialog combobox {
            min-width: 150px;
        }
        
        .settings-dialog button {
            padding: 6px 12px;
            border-radius: 4px;
        }
        
        .settings-dialog scrolledwindow {
            background-color: transparent;
        }
        ";
    
    provider.load_from_data(&css_content);
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Show a notification toast (if available)
pub fn show_notification(_parent: &Window, message: &str) {
    // For now, just print to console to avoid modal dialog conflicts
    println!("Settings: {}", message);
    
    // TODO: In a full implementation, you might use libadwaita's toast or a custom notification
    // that doesn't interfere with dialog management
}

/// Get available CSS themes by scanning themes directory
pub fn get_available_css_themes() -> Vec<String> {
    let themes_dir = std::path::Path::new("themes");
    let mut themes = vec!["standard".to_string()]; // Default theme
    
    if themes_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".css") {
                        let theme_name = filename.strip_suffix(".css").unwrap_or(filename);
                        if theme_name != "standard" {
                            themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    themes.sort();
    themes
}

/// Get available UI languages
pub fn get_available_languages() -> Vec<(String, String)> {
    // Return (code, display_name) pairs
    vec![
        ("en".to_string(), "English".to_string()),
        ("de".to_string(), "Deutsch".to_string()),
        ("es".to_string(), "Español".to_string()),
        ("fr".to_string(), "Français".to_string()),
    ]
}

/// Check if a file exists and is readable
pub fn is_file_readable(path: &str) -> bool {
    std::path::Path::new(path).exists() && std::path::Path::new(path).is_file()
}
