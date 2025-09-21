// IMPORTANT: When updating settings.ron, always preserve all existing settings and only modify the relevant fields.
// This ensures user preferences are not lost when changing schema or other options.
// Settings structure
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Dialog, Label, Notebook, Orientation, Window};

use crate::ui::settings::tabs;
use log::trace;

pub struct Settings {
    pub theme: String,
    pub font_size: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            theme: "light".to_string(),
            font_size: 12,
        }
    }
}

use crate::theme::ThemeManager;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

// Type alias for complex preview refresh callback
type RefreshPreviewCallback = Rc<RefCell<Box<dyn Fn()>>>;

/// Container for optional callbacks passed into the Settings dialog. Using a
/// single struct keeps the function signature compact and satisfies clippy's
/// `too_many_arguments` lint.
///
/// Construct this struct at the call-site and pass it into `show_settings_dialog`.
pub struct SettingsDialogCallbacks {
    pub on_preview_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
    pub refresh_preview: Option<RefreshPreviewCallback>,
    pub on_editor_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
    pub on_schema_changed: Option<Box<dyn Fn(Option<String>) + 'static>>,
    pub on_view_mode_changed: Option<std::boxed::Box<dyn Fn(String) + 'static>>,
    pub on_split_ratio_changed: Option<std::boxed::Box<dyn Fn(i32) + 'static>>,
    pub on_sync_scrolling_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
    pub on_line_numbers_changed: Option<std::boxed::Box<dyn Fn(bool) + 'static>>,
}

pub fn show_settings_dialog(
    parent: &Window,
    theme_manager: Rc<RefCell<ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    callbacks: SettingsDialogCallbacks,
) {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .build();

    let notebook = Notebook::new();
    notebook.set_tab_pos(gtk4::PositionType::Top);

    // Add each tab
    notebook.append_page(
        &tabs::editor::build_editor_tab(settings_path.to_str().unwrap()),
        Some(&Label::new(Some("Editor"))),
    );
    // Build layout tab and provide a callback that will persist the setting and
    // forward the value to any external on_view_mode_changed handler supplied by
    // the caller via the `callbacks` struct.
    let settings_path_clone = settings_path.clone();
    // Read saved view mode so the layout tab can initialize its dropdown.
    use crate::logic::swanson::Settings as AppSettings;
    let saved_view_mode: Option<String> =
        AppSettings::load_from_file(settings_path_clone.to_str().unwrap())
            .unwrap_or_default()
            .layout
            .and_then(|l| l.view_mode);
    // Move the optional outer callback into the closure so we don't require Clone
    let outer_on_view = callbacks.on_view_mode_changed;
    let layout_cb = std::boxed::Box::new(move |selected: String| {
        use crate::logic::swanson::{LayoutSettings, Settings as AppSettings};
        let mut app_settings =
            AppSettings::load_from_file(settings_path_clone.to_str().unwrap()).unwrap_or_default();
        if app_settings.layout.is_none() {
            app_settings.layout = Some(LayoutSettings::default());
        }
        if let Some(ref mut layout) = app_settings.layout {
            layout.view_mode = Some(selected.clone());
        }
        app_settings
            .save_to_file(settings_path_clone.to_str().unwrap())
            .ok();
        // If the caller wanted a direct String callback, call it with the
        // selected value.
        if let Some(ref cb) = outer_on_view {
            cb(selected.clone());
        }
    }) as std::boxed::Box<dyn Fn(String) + 'static>;

    notebook.append_page(
        &tabs::layout::build_layout_tab(
            saved_view_mode,
            Some(layout_cb),
            settings_path.to_str(),
            callbacks.on_split_ratio_changed,
            callbacks.on_sync_scrolling_changed,
            callbacks.on_line_numbers_changed,
        ),
        Some(&Label::new(Some("Layout"))),
    );

    // Appearance tab wiring uses callbacks from the callbacks struct.
    if let (Some(cb), Some(refresh_preview_cb)) = (
        callbacks.on_preview_theme_changed,
        callbacks.refresh_preview.clone(),
    ) {
        notebook.append_page(
            &tabs::appearance::build_appearance_tab(
                theme_manager.clone(),
                settings_path.clone(),
                asset_dir,
                cb,
                refresh_preview_cb,
                callbacks.on_editor_theme_changed,
            ),
            Some(&Label::new(Some("Appearance"))),
        );
    } else {
        notebook.append_page(
            &tabs::appearance::build_appearance_tab(
                theme_manager.clone(),
                settings_path.clone(),
                asset_dir,
                Box::new(|_| {}),
                Rc::new(RefCell::new(Box::new(|| {}) as Box<dyn Fn()>)),
                callbacks.on_editor_theme_changed,
            ),
            Some(&Label::new(Some("Appearance"))),
        );
    }
    notebook.append_page(
        &tabs::language::build_language_tab(),
        Some(&Label::new(Some("Language"))),
    );

    // Add Markdown tab for markdown-specific settings
    notebook.append_page(
        &tabs::markdown::build_markdown_tab(settings_path.to_str().unwrap()),
        Some(&Label::new(Some("Markdown"))),
    );

    // Optionally show Debug tab when `debug` is enabled in settings.ron
    {
        use crate::logic::swanson::Settings as AppSettings;
        let app_settings =
            AppSettings::load_from_file(settings_path.to_str().unwrap()).unwrap_or_default();
        if app_settings.debug.unwrap_or(false) {
            // Pass settings path as string to debug tab builder so it can save changes
            let settings_path_str = settings_path.to_string_lossy().to_string();
            notebook.append_page(
                &tabs::debug::build_debug_tab(&settings_path_str),
                Some(&Label::new(Some("Debug"))),
            );
        }
    }

    // Layout: notebook + close button at bottom right
    let content_box = GtkBox::new(Orientation::Vertical, 0);
    content_box.append(&notebook);

    let button_box = GtkBox::new(Orientation::Horizontal, 0);
    button_box.set_halign(Align::End);
    let close_button = Button::with_label("Close");
    let dialog_clone = dialog.clone();
    close_button.connect_clicked(move |_| {
        trace!("audit: settings dialog closed");
        dialog_clone.close();
    });
    close_button.set_margin_start(0);
    close_button.set_margin_end(8);
    close_button.set_margin_bottom(8);
    close_button.set_margin_top(8);
    button_box.append(&close_button);
    content_box.append(&button_box);

    dialog.set_default_size(700, 600);
    dialog.set_child(Some(&content_box));
    dialog.present();
}
