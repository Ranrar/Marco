use gtk4::prelude::*;
use crate::settings::core::get_app_preferences;
use crate::editor::MarkdownEditor;
use crate::theme::ThemeManager;
use crate::language;

/// Apply settings to the application
pub fn apply_settings_to_app(
    editor: &MarkdownEditor,
    theme_manager: &ThemeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = get_app_preferences();
    
    // Apply UI theme
    let ui_theme = prefs.get_ui_theme();
    let theme = match ui_theme.as_str() {
        "light" => crate::theme::Theme::Light,
        "dark" => crate::theme::Theme::Dark,
        _ => crate::theme::Theme::System,
    };
    theme_manager.set_theme(theme);
    
    // Update editor theme to match UI theme
    editor.update_editor_theme();
    
    // Apply CSS theme
    let css_theme = prefs.get_css_theme();
    editor.set_css_theme(&css_theme);
    
    // Apply view mode
    let view_mode = prefs.get_view_mode();
    editor.set_view_mode(&view_mode);
    
    // Apply language
    let language_code = prefs.get_language();
    language::set_locale(&language_code);
    
    // Apply layout mode
    let layout_mode = prefs.get_layout_mode();
    if layout_mode == "editor-right" {
        editor.set_layout_reversed(true);   // Preview left, editor right
    } else {
        editor.set_layout_reversed(false);  // Editor left, preview right (default)
    }
    
    // Apply function highlighting
    let function_highlighting = prefs.get_function_highlighting();
    editor.set_function_colloring(function_highlighting);
    
    // Apply editor color syntax highlighting
    let editor_color_syntax = prefs.get_editor_color_syntax();
    eprintln!("============ DEBUG: Loading editor_color_syntax setting: {} ============", editor_color_syntax);
    editor.set_editor_color_syntax(editor_color_syntax);
    
    // Apply markdown warnings
    let markdown_warnings = prefs.get_markdown_warnings();
    editor.set_markdown_warnings(markdown_warnings);
    
    Ok(())
}

/// Save current application state to settings
pub fn save_app_state_to_settings(
    editor: &MarkdownEditor,
    theme_manager: &ThemeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = get_app_preferences();
    
    // Save current view mode
    let view_mode = editor.get_view_mode();
    prefs.set_view_mode(&view_mode);
    
    // Save current CSS theme
    let css_theme = editor.get_current_css_theme();
    prefs.set_css_theme(&css_theme);
    
    // Save current UI theme
    let ui_theme = match theme_manager.get_current_theme() {
        crate::theme::Theme::Light => "light",
        crate::theme::Theme::Dark => "dark",
        crate::theme::Theme::System => "system",
    };
    prefs.set_ui_theme(ui_theme);
    
    // Save current language
    let language = language::get_current_locale();
    prefs.set_language(&language);
    
    // Note: Layout mode, function highlighting, and markdown warnings are 
    // already stored in settings when they're changed, so we don't need to 
    // save them again here. The current values can be retrieved from settings.
    
    Ok(())
}

/// Load application state from settings
pub fn load_app_state_from_settings(
    editor: &MarkdownEditor,
    theme_manager: &ThemeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    apply_settings_to_app(editor, theme_manager)
}

/// Save window geometry to settings
pub fn save_window_geometry(
    window: &gtk4::ApplicationWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = get_app_preferences();
    
    // Get the actual window size (not the default size)
    let (width, height) = window.default_size();
    
    // For maximized windows, we still want to save the size they would have when unmaximized
    if !window.is_maximized() {
        prefs.set_window_size(width, height);
    }
    
    // Always save the maximized state
    let maximized = window.is_maximized();
    prefs.set_window_maximized(maximized);
    
    // Note: Getting window position in GTK4 is more complex
    // and may require platform-specific code
    // For now, we'll just save the size and maximized state
    
    Ok(())
}

/// Restore window geometry from settings
pub fn restore_window_geometry(
    window: &gtk4::ApplicationWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = get_app_preferences();
    
    let (width, height) = prefs.get_window_size();
    if width > 0 && height > 0 {
        window.set_default_size(width, height);
    }
    
    // Restore maximized state
    let maximized = prefs.get_window_maximized();
    if maximized {
        window.maximize();
    }
    
    // Note: Setting window position in GTK4 is more complex
    // and may require platform-specific code
    // For now, we'll just restore the size and maximized state
    
    Ok(())
}

/// Connect to settings changes and update UI accordingly
pub fn connect_settings_changes(
    _editor: &MarkdownEditor,
    _theme_manager: &ThemeManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let prefs = get_app_preferences();
    
    // Connect to settings changes
    let _handler_id = prefs.connect_changed(None, move |_settings, key| {
        println!("Settings changed: {}", key);
        // In a full implementation, we would emit a signal or use a different mechanism
        // to notify the UI of changes
    });
    
    Ok(())
}
