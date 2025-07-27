
use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use std::rc::Rc;
use std::cell::RefCell;

// Import your theme manager
use crate::theme::{ThemeManager};
use crate::logic::theme_list::{list_app_themes, find_theme_by_label};

pub fn build_appearance_tab(
    theme_manager: Rc<RefCell<ThemeManager>>,
    settings_path: std::path::PathBuf,
    on_preview_theme_changed: Box<dyn Fn(String) + 'static>,
    theme_mode: Rc<RefCell<String>>,
    refresh_preview: Rc<RefCell<Box<dyn Fn()>>>,
) -> GtkBox {
    use gtk4::{Label, ComboBoxText, Scale, Adjustment, Button, Box as GtkBox, Orientation, Align};

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-appearance");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

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

    // Helper for a row: title, desc, control right-aligned, with extra vertical space
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
        vbox.append(&desc);
        vbox.set_spacing(4);
        vbox.set_margin_bottom(24);
        vbox
    };

    use std::rc::Rc;
    let on_preview_theme_changed = Rc::new(on_preview_theme_changed);
    // Application Theme (Dropdown, dynamic)
    let app_theme_combo = ComboBoxText::new();
    let theme_dir = std::path::Path::new("src/assets/themes/gtk4");
    let app_themes = list_app_themes(theme_dir);
    for entry in &app_themes {
        app_theme_combo.append_text(&entry.label);
    }
    // Set active based on current theme filename in settings
    let current_theme = theme_manager.borrow().settings.appearance.as_ref().and_then(|a| a.app_theme.clone());
    let current_theme_str = current_theme.as_deref().unwrap_or("standard-light.css");
    let active_idx = app_themes.iter().position(|t| t.filename == current_theme_str).unwrap_or(0);
    app_theme_combo.set_active(Some(active_idx as u32));
    let app_theme_row = add_row(
        "Application Theme",
        "Choose a theme for the user interface.",
        app_theme_combo.upcast_ref(),
    );
    container.append(&app_theme_row);

    // HTML Preview Theme (Dropdown)
    let preview_theme_combo = ComboBoxText::new();
    use crate::logic::theme_list::list_html_view_themes;
    let html_theme_dir = std::path::Path::new("src/assets/themes/html_viever");
    let html_themes = list_html_view_themes(html_theme_dir);
    for entry in &html_themes {
        preview_theme_combo.append_text(&entry.label);
    }
    // Set active based on current preview_theme filename in settings
    let current_preview = theme_manager.borrow().settings.appearance.as_ref().and_then(|a| a.preview_theme.clone());
    let current_preview_str = current_preview.as_deref().unwrap_or("standard.css");
    let preview_active_idx = html_themes.iter().position(|t| t.filename == current_preview_str).unwrap_or(0);
    preview_theme_combo.set_active(Some(preview_active_idx as u32));
    let preview_theme_row = add_row(
        "HTML Preview Theme",
        "Select a CSS theme for rendered Markdown preview.",
        preview_theme_combo.upcast_ref(),
    );
    container.append(&preview_theme_row);

    // --- SIGNALS ---
    // Track if the user has made a manual selection of the HTML preview theme
    use std::cell::Cell;
    let user_selected_preview_theme = Rc::new(Cell::new(false));

    // Application Theme change
    {
        let theme_manager = Rc::clone(&theme_manager);
        let settings_path = settings_path.clone();
        let app_themes = app_themes.clone();
        let html_themes = html_themes.clone();
        let preview_theme_combo = preview_theme_combo.clone();
        let on_preview_theme_changed = Rc::clone(&on_preview_theme_changed);
        let user_selected_preview_theme = Rc::clone(&user_selected_preview_theme);
        let theme_mode = Rc::clone(&theme_mode);
        let refresh_preview = refresh_preview.clone();
        app_theme_combo.connect_changed(move |combo| {
            let idx = combo.active().unwrap_or(0) as usize;
            if let Some(theme) = app_themes.get(idx) {
                // Save theme filename to settings, apply CSS, set color mode
                theme_manager.borrow_mut().set_app_theme(theme.filename.clone(), &settings_path);

                // Update theme_mode for preview
                *theme_mode.borrow_mut() = if theme.is_dark { "theme-dark".to_string() } else { "theme-light".to_string() };
                (refresh_preview.borrow())();

                // Only auto-sync HTML preview theme if user hasn't made a manual selection
                if !user_selected_preview_theme.get() {
                    if let Some(matching_html_theme_idx) = html_themes.iter().position(|t| t.is_dark == theme.is_dark) {
                        preview_theme_combo.set_active(Some(matching_html_theme_idx as u32));
                        let html_theme = &html_themes[matching_html_theme_idx];
                        theme_manager.borrow_mut().set_preview_theme(html_theme.filename.clone(), &settings_path);
                        (on_preview_theme_changed)(html_theme.filename.clone());
                    }
                }
            }
        });
    }
    // Preview Theme change
    {
        let theme_manager = Rc::clone(&theme_manager);
        let settings_path = settings_path.clone();
        let html_themes = html_themes.clone();
        let on_preview_theme_changed = Rc::clone(&on_preview_theme_changed);
        let user_selected_preview_theme = Rc::clone(&user_selected_preview_theme);
        preview_theme_combo.connect_changed(move |combo| {
            let idx = combo.active().unwrap_or(0) as usize;
            if let Some(theme_entry) = html_themes.get(idx) {
                // Mark that the user has made a manual selection
                user_selected_preview_theme.set(true);
                println!("Saving preview_theme: {} to {:?}", theme_entry.filename, settings_path);
                theme_manager.borrow_mut().set_preview_theme(theme_entry.filename.clone(), &settings_path);
                // Apply the new preview theme live
                (on_preview_theme_changed)(theme_entry.filename.clone());
            }
        });
    }

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
    let ui_font_combo = ComboBoxText::new();
    ui_font_combo.append_text("System Default");
    ui_font_combo.append_text("Sans");
    ui_font_combo.append_text("Serif");
    ui_font_combo.append_text("Monospace");
    ui_font_combo.set_active(Some(0));
    let ui_font_row = add_row(
        "UI Font",
        "Customize the font used in the application's user interface (menus, sidebars).",
        ui_font_combo.upcast_ref(),
    );
    container.append(&ui_font_row);

    // UI Font Size (Slider)
    let ui_font_size_adj = Adjustment::new(14.0, 10.0, 24.0, 1.0, 0.0, 0.0);
    let ui_font_size_slider = Scale::new(Orientation::Horizontal, Some(&ui_font_size_adj));
    ui_font_size_slider.set_draw_value(false);
    ui_font_size_slider.set_hexpand(true);
    ui_font_size_slider.set_round_digits(0);
    ui_font_size_slider.set_width_request(300);
    for size in 10..=24 {
        ui_font_size_slider.add_mark(size as f64, gtk4::PositionType::Bottom, Some(&size.to_string()));
    }
    let ui_font_size_vbox = GtkBox::new(Orientation::Vertical, 2);
    let ui_font_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let ui_font_size_title = bold_label("UI Font Size");
    let ui_font_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    ui_font_size_spacer.set_hexpand(true);
    ui_font_size_hbox.append(&ui_font_size_title);
    ui_font_size_hbox.append(&ui_font_size_spacer);
    // No spinbutton for now
    ui_font_size_hbox.set_hexpand(true);
    ui_font_size_vbox.append(&ui_font_size_hbox);
    ui_font_size_vbox.append(&desc_label("Customize the font size used in the application's user interface (menus, sidebars)."));
    ui_font_size_slider.set_halign(Align::Start);
    ui_font_size_vbox.append(&ui_font_size_slider);
    ui_font_size_vbox.set_spacing(2);
    ui_font_size_vbox.set_margin_bottom(24);
    container.append(&ui_font_size_vbox);

    container
}
