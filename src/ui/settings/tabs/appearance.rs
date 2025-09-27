use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Import your theme manager
use crate::logic::loaders::theme_loader::list_html_view_themes;
use crate::logic::signal_manager::SignalManager;
use crate::theme::ThemeManager;

pub fn build_appearance_tab(
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    on_preview_theme_changed: Box<dyn Fn(String) + 'static>,
    refresh_preview: Rc<RefCell<Box<dyn Fn()>>>,
    on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
) -> (gtk4::Box, Rc<RefCell<SignalManager>>) {
    use gtk4::{
        Adjustment, Align, Box as GtkBox, Button, DropDown, Label, Orientation, Scale,
        SpinButton, StringList, PropertyExpression, StringObject, Expression,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-appearance");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);
    
    // Create signal manager to track all signal handlers for proper cleanup
    let signal_manager = Rc::new(RefCell::new(SignalManager::new()));

    // Helper for bold label
    let bold_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(text)));
        l
    };

    // Helper for normal description
    let desc_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_wrap(true);
        l
    };

    // Helper for a row: title, desc, control right-aligned, with reduced spacing (new style)
    let add_row = |title: &str, desc: &str, control: &gtk4::Widget| {
        let vbox = GtkBox::new(Orientation::Vertical, 2);
        let hbox = GtkBox::new(Orientation::Horizontal, 0);
        let title_label = bold_label(title);
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        let control = control.clone();
        hbox.append(&title_label);
        hbox.append(&spacer); // Expanding spacer
        hbox.append(&control);
        control.set_halign(Align::End);
        hbox.set_hexpand(true);
        vbox.append(&hbox);
        let desc = desc_label(desc);
        desc.add_css_class("dim-label");
        vbox.append(&desc);
        vbox.set_spacing(4);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(12); // Reduced from 24px to 12px
        vbox
    };

    // --- Split Functions ---

    let build_html_preview_theme_row =
        |theme_manager: Rc<RefCell<ThemeManager>>,
         settings_path: std::path::PathBuf,
         on_preview_theme_changed: Rc<Box<dyn Fn(String)>>,
         user_selected_preview_theme: Rc<std::cell::Cell<bool>>,
         html_themes: Vec<crate::logic::loaders::theme_loader::ThemeEntry>|
         -> (gtk4::Box, DropDown) {
            // Create preview theme dropdown with automatic checkmarks
            // Extract theme labels for the dropdown
            let theme_labels: Vec<&str> = html_themes.iter().map(|entry| entry.label.as_str()).collect();
            
            // Create StringList from theme labels
            let theme_string_list = StringList::new(&theme_labels);
            
            // Create PropertyExpression for string matching (required for DropDown)
            let theme_expression = PropertyExpression::new(
                StringObject::static_type(),
                None::<Expression>,
                "string",
            );
            
            // Create DropDown with automatic checkmarks
            let preview_theme_combo = DropDown::new(Some(theme_string_list), Some(theme_expression));
            
            let current_preview = theme_manager
                .borrow()
                .settings
                .appearance
                .as_ref()
                .and_then(|a| a.preview_theme.clone());
            let current_preview_str = current_preview.as_deref().unwrap_or("standard.css");
            let preview_active_idx = html_themes
                .iter()
                .position(|t| t.filename == current_preview_str)
                .unwrap_or(0);
            preview_theme_combo.set_selected(preview_active_idx as u32);
            // Signal - properly managed for cleanup
            {
                let theme_manager = Rc::clone(&theme_manager);
                let settings_path = settings_path.clone();
                let html_themes = html_themes.clone();
                let on_preview_theme_changed = Rc::clone(&on_preview_theme_changed);
                let user_selected_preview_theme = Rc::clone(&user_selected_preview_theme);
                let signal_manager = signal_manager.clone();
                
                let handler_id = preview_theme_combo.connect_selected_notify(move |combo| {
                    let idx = combo.selected() as usize;
                    if let Some(theme_entry) = html_themes.get(idx) {
                        user_selected_preview_theme.set(true);
                        log::info!(
                            "Saving preview_theme: {} to {:?}",
                            theme_entry.filename,
                            settings_path
                        );
                        theme_manager
                            .borrow_mut()
                            .set_preview_theme(theme_entry.filename.clone(), &settings_path);
                        (on_preview_theme_changed)(theme_entry.filename.clone());
                    }
                });
                
                // Register handler for cleanup
                signal_manager.borrow_mut().register_handler(
                    "appearance_tab", 
                    &preview_theme_combo.clone().upcast(), 
                    handler_id
                );
            }
            let row = add_row(
                "HTML Preview Theme",
                "Select a CSS theme for rendered Markdown preview.",
                preview_theme_combo.upcast_ref(),
            );
            (row, preview_theme_combo)
        };

    // --- Compose Tab ---
    // Light/Dark Mode Dropdown
    use crate::logic::swanson::Settings as AppSettings;
    let app_settings =
        AppSettings::load_from_file(settings_path.to_str().unwrap()).unwrap_or_default();
    let current_mode = app_settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.clone())
        .unwrap_or("marco-light".to_string());
    // Create color mode dropdown with automatic checkmarks
    let color_mode_options = ["light", "dark"];
    
    // Create StringList from color mode options
    let color_mode_string_list = StringList::new(&color_mode_options);
    
    // Create PropertyExpression for string matching (required for DropDown)
    let color_mode_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<Expression>,
        "string",
    );
    
    // Create DropDown with automatic checkmarks
    let color_mode_combo = DropDown::new(Some(color_mode_string_list), Some(color_mode_expression));
    
    let active_idx = match current_mode.as_str() {
        "marco-dark" | "dark" => 1,
        _ => 0, // Default to light mode for "marco-light", "light", or any other value
    };
    color_mode_combo.set_selected(active_idx);
    let color_mode_row = add_row(
        "Light/Dark Mode",
        "Choose between light or dark user interface.",
        color_mode_combo.upcast_ref(),
    );
    container.append(&color_mode_row);
    // Wire dropdown to update theme state and persist user preference
    {
        let settings_path = settings_path.clone();
        let refresh_preview = Rc::clone(&refresh_preview);
        let on_editor_theme_changed = on_editor_theme_changed.map(Rc::new);
        let signal_manager = signal_manager.clone();
        
        let handler_id = color_mode_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected();
            let mode = if idx == 1 { "dark" } else { "light" };
            log::info!("Switching color mode to: {}", mode);
            // Update settings struct and persist
            let mut app_settings =
                AppSettings::load_from_file(settings_path.to_str().unwrap()).unwrap_or_default();
            if let Some(ref mut appearance) = app_settings.appearance {
                appearance.editor_mode = Some(mode.to_string());
            } else {
                app_settings.appearance = Some(crate::logic::swanson::AppearanceSettings {
                    editor_mode: Some(mode.to_string()),
                    ..Default::default()
                });
            }
            app_settings
                .save_to_file(settings_path.to_str().unwrap())
                .ok();
            // Call editor theme change callback if provided
            if let Some(ref callback) = on_editor_theme_changed {
                let scheme_id = if idx == 1 {
                    "marco-dark"
                } else {
                    "marco-light"
                };
                callback(scheme_id.to_string());
            }
            // Refresh preview and UI
            (refresh_preview.borrow())();
        });
        
        // Register handler for cleanup
        signal_manager.borrow_mut().register_handler(
            "appearance_tab", 
            &color_mode_combo.clone().upcast(), 
            handler_id
        );
    }
    use std::rc::Rc;
    let on_preview_theme_changed = Rc::new(on_preview_theme_changed);
    use std::cell::Cell;
    let user_selected_preview_theme = Rc::new(Cell::new(false));
    let html_theme_dir = asset_dir.join("themes/html_viever");
    let html_themes = list_html_view_themes(&html_theme_dir);

    // Build HTML Preview Theme row
    let (preview_theme_row, _preview_theme_combo) = build_html_preview_theme_row(
        Rc::clone(&theme_manager),
        settings_path.clone(),
        Rc::clone(&on_preview_theme_changed),
        Rc::clone(&user_selected_preview_theme),
        html_themes.clone(),
    );
    container.append(&preview_theme_row);
    // You can use refresh_preview here for future preview refresh logic if needed.

    // ...existing code for custom CSS, font, etc...
    // Custom CSS for Preview (Button)
    let custom_css_button = Button::with_label("Open CSS Themes Folder");
    let custom_css_row = add_row(
        "Custom CSS for Preview",
        "Add your own custom CSS to override the preview style.",
        custom_css_button.upcast_ref(),
    );
    container.append(&custom_css_row);

    // UI Font (Dropdown)
    // Create UI font dropdown with automatic checkmarks
    let ui_font_options = ["System Default", "Sans", "Serif", "Monospace"];
    
    // Create StringList from UI font options
    let ui_font_string_list = StringList::new(&ui_font_options);
    
    // Create PropertyExpression for string matching (required for DropDown)
    let ui_font_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<Expression>,
        "string",
    );
    
    // Create DropDown with automatic checkmarks
    let ui_font_combo = DropDown::new(Some(ui_font_string_list), Some(ui_font_expression));
    ui_font_combo.set_selected(0); // Default to "System Default"
    let ui_font_row = add_row(
        "UI Font",
        "Customize the font used in the application's user interface (menus, sidebars).",
        ui_font_combo.upcast_ref(),
    );
    container.append(&ui_font_row);

    // UI Font Size (Slider/SpinButton)
    let ui_font_size_adj = Adjustment::new(14.0, 10.0, 24.0, 1.0, 0.0, 0.0);

    // UI Font Size SpinButton with title
    let ui_font_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let ui_font_size_title = bold_label("UI Font Size");
    let ui_font_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    ui_font_size_spacer.set_hexpand(true);

    let ui_font_size_spin = SpinButton::new(Some(&ui_font_size_adj), 1.0, 0);
    ui_font_size_spin.set_halign(Align::End);

    ui_font_size_hbox.append(&ui_font_size_title);
    ui_font_size_hbox.append(&ui_font_size_spacer);
    ui_font_size_hbox.append(&ui_font_size_spin);
    ui_font_size_hbox.set_margin_top(8);
    ui_font_size_hbox.set_margin_bottom(4);
    container.append(&ui_font_size_hbox);

    // Description under header
    let ui_font_size_desc = desc_label(
        "Customize the font size used in the application's user interface (menus, sidebars).",
    );
    ui_font_size_desc.add_css_class("dim-label");
    ui_font_size_desc.set_margin_bottom(12);
    container.append(&ui_font_size_desc);

    // Slider below
    let ui_font_size_slider = Scale::new(Orientation::Horizontal, Some(&ui_font_size_adj));
    ui_font_size_slider.set_draw_value(false);
    ui_font_size_slider.set_hexpand(true);
    ui_font_size_slider.set_round_digits(0);
    ui_font_size_slider.set_width_request(300);
    for size in 10..=24 {
        ui_font_size_slider.add_mark(
            size as f64,
            gtk4::PositionType::Bottom,
            Some(&size.to_string()),
        );
    }
    ui_font_size_slider.set_halign(Align::Start);
    ui_font_size_slider.set_margin_bottom(12);
    container.append(&ui_font_size_slider);

    (container, signal_manager)
}
