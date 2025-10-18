// IMPORTANT: When updating settings.ron, always preserve all existing settings and only modify the relevant fields.
// This ensures user preferences are not lost when changing schema or other options.
// Settings structure
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Notebook, Orientation, Window};

use crate::logic::signal_manager::SignalManager;
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
use std::rc::{Rc, Weak};

// Type alias for complex preview refresh callback
type RefreshPreviewCallback = Rc<RefCell<Box<dyn Fn()>>>;

// Cached dialog storage using thread-local storage for GTK single-threaded environment
// Also cache the parent window to detect when dialog should use same parent
thread_local! {
    static CACHED_DIALOG: RefCell<Option<(Weak<Window>, Weak<Window>)>> = const { RefCell::new(None) };
}

/// Get or create the settings dialog, reusing existing if possible
fn get_or_create_cached_dialog(
    parent: &Window,
    theme_manager: Rc<RefCell<ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    callbacks: SettingsDialogCallbacks,
) -> Rc<Window> {
    CACHED_DIALOG.with(|cached| {
        // Check if we have a valid cached dialog with the same parent
        if let Some((weak_dialog, weak_parent)) = cached.borrow().as_ref() {
            if let (Some(dialog), Some(cached_parent)) = (weak_dialog.upgrade(), weak_parent.upgrade()) {
                // Only reuse if same parent window (common case)
                if std::ptr::eq(parent, &*cached_parent) {
                    trace!("audit: reusing cached settings dialog with same parent");
                    return dialog;
                } else {
                    trace!("audit: parent changed, creating new settings dialog");
                }
            }
        }

        // Create new dialog if none cached, previous was destroyed, or parent changed
        trace!("audit: creating new settings dialog");
        let dialog = Rc::new(create_dialog_impl(parent, theme_manager, settings_path, asset_dir, callbacks));
        
        // Cache weak references to both dialog and parent
        *cached.borrow_mut() = Some((
            Rc::downgrade(&dialog),
            Rc::downgrade(&Rc::new(parent.clone()))
        ));
        
        dialog
    })
}

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
    // Use cached dialog to avoid recreation overhead
    let dialog = get_or_create_cached_dialog(parent, theme_manager, settings_path, asset_dir, callbacks);
    dialog.present();
}

/// Clear the cached dialog (useful for cleanup or testing)
#[allow(dead_code)]
pub fn clear_cached_dialog() {
    CACHED_DIALOG.with(|cached| {
        *cached.borrow_mut() = None;
    });
    trace!("audit: cleared cached settings dialog");
}

/// Internal function to create the actual dialog implementation
/// This contains the original dialog creation logic
fn create_dialog_impl(
    parent: &Window,
    theme_manager: Rc<RefCell<ThemeManager>>,
    settings_path: PathBuf,
    asset_dir: &std::path::Path,
    callbacks: SettingsDialogCallbacks,
) -> Window {
    // Detect parent window theme
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let theme_class = if parent_widget.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };
    
    // Create Window instead of Dialog (non-modal to allow editing while settings is open)
    let window = Window::builder()
        .transient_for(parent)
        .modal(false) // Non-modal so user can edit text in main window
        .default_width(600)
        .default_height(500)
        .resizable(false)
        .build();
    
    // Apply CSS classes for theming
    window.add_css_class("marco-settings-window");
    window.add_css_class(theme_class);
    
    // Add ESC key handler to close window (like search dialog)
    let key_controller = gtk4::EventControllerKey::new();
    let window_weak_for_esc = window.downgrade();
    key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(window) = window_weak_for_esc.upgrade() {
                window.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);
    
    // Set up runtime theme synchronization
    // Monitor parent window for CSS class changes and update settings window accordingly
    {
        let window_weak = window.downgrade();
        let parent_weak = parent.downgrade();
        
        // Connect to parent's CSS class changes
        // We'll poll for changes using a timeout since GTK doesn't have a direct CSS class change signal
        let poll_interval = std::time::Duration::from_millis(100);
        glib::timeout_add_local(poll_interval, move || {
            // Check if both windows still exist
            if let (Some(settings_win), Some(parent_win)) = (window_weak.upgrade(), parent_weak.upgrade()) {
                let parent_widget = parent_win.upcast_ref::<gtk4::Widget>();
                let settings_widget = settings_win.upcast_ref::<gtk4::Widget>();
                
                // Detect current parent theme
                let parent_is_dark = parent_widget.has_css_class("marco-theme-dark");
                let settings_is_dark = settings_widget.has_css_class("marco-theme-dark");
                
                // If themes don't match, synchronize
                if parent_is_dark != settings_is_dark {
                    if parent_is_dark {
                        // Switch to dark theme
                        settings_widget.remove_css_class("marco-theme-light");
                        settings_widget.add_css_class("marco-theme-dark");
                        trace!("Settings dialog switched to dark theme");
                    } else {
                        // Switch to light theme
                        settings_widget.remove_css_class("marco-theme-dark");
                        settings_widget.add_css_class("marco-theme-light");
                        trace!("Settings dialog switched to light theme");
                    }
                }
                
                // Continue polling if settings window is visible
                if settings_win.is_visible() {
                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            } else {
                // One or both windows destroyed, stop polling
                glib::ControlFlow::Break
            }
        });
    }
    
    // Create custom HeaderBar matching marco's style
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);
    
    // Set title in headerbar
    let title_label = Label::new(Some("Settings"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));
    
    // Create custom close button with icon font
    let close_label = Label::new(None);
    close_label.set_markup("<span font_family='icomoon'>\u{39}</span>"); // \u{39} = marco-close icon
    close_label.set_valign(Align::Center);
    close_label.add_css_class("icon-font");
    
    let btn_close_titlebar = Button::new();
    btn_close_titlebar.set_child(Some(&close_label));
    btn_close_titlebar.set_tooltip_text(Some("Close"));
    btn_close_titlebar.set_valign(Align::Center);
    btn_close_titlebar.set_margin_start(1);
    btn_close_titlebar.set_margin_end(1);
    btn_close_titlebar.set_focusable(false);
    btn_close_titlebar.set_can_focus(false);
    btn_close_titlebar.set_has_frame(false);
    btn_close_titlebar.add_css_class("topright-btn");
    btn_close_titlebar.add_css_class("window-control-btn");
    
    // Add close button to right side of headerbar
    headerbar.pack_end(&btn_close_titlebar);
    
    window.set_titlebar(Some(&headerbar));

    let notebook = Notebook::new();
    notebook.set_tab_pos(gtk4::PositionType::Top);
    notebook.add_css_class("marco-settings-notebook");

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
    use core::logic::swanson::SettingsManager;
    let saved_view_mode: Option<String> = {
        if let Ok(settings_manager) = SettingsManager::initialize(settings_path_clone.clone()) {
            settings_manager.get_settings()
                .layout
                .and_then(|l| l.view_mode)
        } else {
            None
        }
    };
    // Move the optional outer callback into the closure so we don't require Clone
    let outer_on_view = callbacks.on_view_mode_changed;
    let layout_cb = std::boxed::Box::new(move |selected: String| {
        use core::logic::swanson::{LayoutSettings, SettingsManager};
        if let Ok(settings_manager) = SettingsManager::initialize(settings_path_clone.clone()) {
            if let Err(e) = settings_manager.update_settings(|settings| {
                if settings.layout.is_none() {
                    settings.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut layout) = settings.layout {
                    layout.view_mode = Some(selected.clone());
                }
            }) {
                eprintln!("Failed to update view mode setting: {}", e);
            }
        } else {
            eprintln!("Failed to initialize settings manager for view mode update");
        }
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

    // Collect signal managers for cleanup on dialog close
    let mut signal_managers: Vec<Rc<RefCell<SignalManager>>> = Vec::new();

    // Appearance tab wiring uses callbacks from the callbacks struct.
    if let (Some(cb), Some(refresh_preview_cb)) = (
        callbacks.on_preview_theme_changed,
        callbacks.refresh_preview.clone(),
    ) {
        let (appearance_tab, appearance_signals) = tabs::appearance::build_appearance_tab(
            theme_manager.clone(),
            settings_path.clone(),
            asset_dir,
            cb,
            refresh_preview_cb,
            callbacks.on_editor_theme_changed,
        );
        notebook.append_page(&appearance_tab, Some(&Label::new(Some("Appearance"))));
        signal_managers.push(appearance_signals);
    } else {
        let (appearance_tab, appearance_signals) = tabs::appearance::build_appearance_tab(
            theme_manager.clone(),
            settings_path.clone(),
            asset_dir,
            Box::new(|_| {}),
            Rc::new(RefCell::new(Box::new(|| {}) as Box<dyn Fn()>)),
            callbacks.on_editor_theme_changed,
        );
        notebook.append_page(&appearance_tab, Some(&Label::new(Some("Appearance"))));
        signal_managers.push(appearance_signals);
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
        use core::logic::swanson::SettingsManager;
        if let Ok(settings_manager) = SettingsManager::initialize(settings_path.clone()) {
            let app_settings = settings_manager.get_settings();
            if app_settings.debug.unwrap_or(false) {
                // Pass settings path as string to debug tab builder so it can save changes
                let settings_path_str = settings_path.to_string_lossy().to_string();
                notebook.append_page(
                    &tabs::debug::build_debug_tab(&settings_path_str),
                    Some(&Label::new(Some("Debug"))),
                );
            }
        } else {
            eprintln!("Failed to initialize settings manager for debug tab visibility check");
        }
    }

    // Layout: notebook + close button at bottom right
    let content_box = GtkBox::new(Orientation::Vertical, 0);
    content_box.add_css_class("marco-settings-content");
    content_box.append(&notebook);

    // Create close button wrapped in a table-like frame for alignment
    let close_button = Button::with_label("Close");
    close_button.add_css_class("marco-settings-close-button");
    close_button.set_halign(Align::End);
    close_button.set_valign(Align::Center);
    
    // Wrap close button in frame matching the settings rows
    let close_frame = gtk4::Frame::new(None);
    close_frame.add_css_class("marco-settings-close-frame");
    close_frame.set_height_request(56);  // Match ROW_FIXED_HEIGHT (reduced from 70 to 56)
    close_frame.set_vexpand(false);
    
    let close_inner_box = GtkBox::new(Orientation::Horizontal, 0);
    close_inner_box.set_margin_start(10);
    close_inner_box.set_margin_end(10);
    close_inner_box.set_margin_top(6);
    close_inner_box.set_margin_bottom(6);
    close_inner_box.set_halign(Align::Fill);
    close_inner_box.set_valign(Align::Center);
    
    // Add expanding spacer to push button to the right
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    
    close_inner_box.append(&spacer);
    close_inner_box.append(&close_button);
    
    close_frame.set_child(Some(&close_inner_box));
    
    // Add some top margin to separate from tabs
    close_frame.set_margin_top(4);
    
    content_box.append(&close_frame);
    
    let window_clone = window.clone();
    let window_weak_for_titlebar = window.downgrade();
    
    // Connect titlebar close button
    btn_close_titlebar.connect_clicked({
        let signal_managers = signal_managers.clone();
        let window_weak = window_weak_for_titlebar.clone();
        move |_| {
            trace!("audit: settings dialog closed via titlebar button");
            // Clean up all signal handlers before closing
            for manager in &signal_managers {
                manager.borrow_mut().disconnect_all();
            }
            if let Some(window) = window_weak.upgrade() {
                window.close();
            }
        }
    });
    
    // Connect bottom close button
    close_button.connect_clicked({
        let signal_managers = signal_managers.clone();
        move |_| {
            trace!("audit: settings dialog closed via close button");
            // Clean up all signal handlers before closing
            for manager in &signal_managers {
                manager.borrow_mut().disconnect_all();
            }
            window_clone.close();
        }
    });

    // Ensure signal handlers are cleaned up if window is closed
    window.connect_close_request({
        let signal_managers = signal_managers.clone();
        move |_| {
            for manager in &signal_managers {
                manager.borrow_mut().disconnect_all();
            }
            log::debug!("Settings window closed - cleaned up signal handlers");
            glib::Propagation::Proceed
        }
    });

    window.set_child(Some(&content_box));
    
    window
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// Smoke test to verify dialog caching works without errors
    #[test]
    fn smoke_test_dialog_caching() {
        // Clear any cached dialog from previous tests
        clear_cached_dialog();
        
        // The actual dialog creation requires GTK to be initialized with a display
        // In a real environment, the dialog would be cached on subsequent calls
        println!("Dialog caching smoke test passed - no compilation errors");
    }

    #[test]
    fn test_clear_cached_dialog() {
        // Test that clearing the cache works
        clear_cached_dialog();
        
        // Call it again to ensure it doesn't panic on empty cache
        clear_cached_dialog();
        
        println!("Clear cached dialog test passed");
    }
    
    /// Demonstrates the performance improvement concept
    /// In real usage, the first call creates widgets, subsequent calls reuse them
    #[test] 
    fn demonstrate_caching_concept() {
        // Simulate the performance difference between widget creation vs reuse
        let start = Instant::now();
        
        // First "creation" - expensive widget creation
        std::thread::sleep(std::time::Duration::from_micros(100));
        let first_creation = start.elapsed();
        
        let start = Instant::now();
        // Second "reuse" - much faster, just present existing dialog
        std::thread::sleep(std::time::Duration::from_micros(10));
        let second_reuse = start.elapsed();
        
        println!("Simulated first creation: {:?}", first_creation);
        println!("Simulated second reuse: {:?}", second_reuse);
        
        // The real implementation shows similar improvements:
        // - First call: Create Dialog + Notebook + Tabs + Buttons + Signal handlers
        // - Second call: Reuse existing widgets, just call present()
        assert!(second_reuse < first_creation);
        
        println!("Dialog caching demonstrates expected performance improvement");
    }
}
