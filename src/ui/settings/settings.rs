// IMPORTANT: When updating settings.ron, always preserve all existing settings and only modify the relevant fields.
// This ensures user preferences are not lost when changing schema or other options.
// Settings structure
use gtk4::prelude::*;
use gtk4::{Dialog, Window, Notebook, Button, Box as GtkBox, Orientation, Align, Label};

use crate::ui::settings::tabs;

pub struct Settings {
    pub theme: String,
    pub font_size: i32,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            theme: "light".to_string(),
            font_size: 12,
        }
    }
}

use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::theme::ThemeManager;

pub fn show_settings_dialog(
    parent: &Window,
    theme_manager: Rc<RefCell<ThemeManager>>,
    settings_path: PathBuf,
    on_preview_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
    refresh_preview: Option<Rc<RefCell<Box<dyn Fn()>>>>,
    on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
    on_schema_changed_callback: Option<Box<dyn Fn(Option<String>) + 'static>>,
) {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .build();

    let notebook = Notebook::new();
    notebook.set_tab_pos(gtk4::PositionType::Top);

    // Add each tab
    notebook.append_page(&tabs::editor::build_editor_tab(),     Some(&Label::new(Some("Editor"))));
    notebook.append_page(&tabs::layout::build_layout_tab(),     Some(&Label::new(Some("Layout"))));
    if let (Some(cb), Some(refresh_preview)) = (on_preview_theme_changed, refresh_preview.clone()) {
        notebook.append_page(&tabs::appearance::build_appearance_tab(
            theme_manager.clone(),
            settings_path.clone(),
            cb,
            refresh_preview,
            on_editor_theme_changed,
        ), Some(&Label::new(Some("Appearance"))));
    } else {
        notebook.append_page(&tabs::appearance::build_appearance_tab(
            theme_manager.clone(),
            settings_path.clone(),
            Box::new(|_| {}),
            Rc::new(RefCell::new(Box::new(|| {}) as Box<dyn Fn()>)),
            on_editor_theme_changed,
        ), Some(&Label::new(Some("Appearance"))));
    }
    // --- Markdown Schema Tab ---
    use crate::ui::settings::tabs::schema::build_schema_tab;
    let schema_root = "src/assets/markdown_schema";
        // Load settings using Settings struct
        use crate::logic::swanson::Settings as AppSettings;
        let app_settings = AppSettings::load_from_file(settings_path.to_str().unwrap()).unwrap_or_default();
        let active_schema = app_settings.active_schema.clone();
        let schema_disabled = app_settings.schema_disabled.unwrap_or(false);
        // Handler for schema change
        let settings_path_clone = settings_path.clone();
        let on_schema_changed = Rc::new(move |selected: Option<String>| {
            // Update settings using Settings struct
            use crate::logic::swanson::Settings as AppSettings;
            let mut app_settings = AppSettings::load_from_file(settings_path_clone.to_str().unwrap()).unwrap_or_default();
            app_settings.active_schema = selected.clone();
            app_settings.schema_disabled = Some(selected.is_none());
            app_settings.save_to_file(settings_path_clone.to_str().unwrap()).ok();
            // Reload parser and update UI (handled by on_schema_changed_callback in caller)
            // Invoke optional external callback so callers can update shared state
            if let Some(ref cb) = on_schema_changed_callback {
                cb(selected.clone());
            }
        });
        notebook.append_page(&build_schema_tab(
            schema_root,
            active_schema,
            schema_disabled,
            on_schema_changed,
        ), Some(&Label::new(Some("Markdown Schema"))));
    notebook.append_page(&tabs::language::build_language_tab(), Some(&Label::new(Some("Language"))));

    // Layout: notebook + close button at bottom right
    let content_box = GtkBox::new(Orientation::Vertical, 0);
    content_box.append(&notebook);

    let button_box = GtkBox::new(Orientation::Horizontal, 0);
    button_box.set_halign(Align::End);
    let close_button = Button::with_label("Close");
    let dialog_clone = dialog.clone();
    close_button.connect_clicked(move |_| dialog_clone.close());
    button_box.append(&close_button);
    content_box.append(&button_box);

    dialog.set_default_size(700, 600); // Make dialog wider
    dialog.set_child(Some(&content_box));
    dialog.present();
}