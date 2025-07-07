use gtk4::prelude::*;
use gtk4::{
    Box, Button, ComboBoxText, Dialog, 
    Label, Orientation, ResponseType, Switch, Window, Notebook, CheckButton,
};
use std::rc::Rc;
use std::cell::RefCell;
use crate::settings::core::get_app_preferences;
use crate::settings::ui::{create_settings_section_header, create_settings_row, get_available_css_themes, get_available_languages};

/// Tracks current values and changes in the settings dialog
#[derive(Debug, Clone)]
struct SettingsChangeTracker {
    // Current values in the dialog (not yet saved)
    function_highlighting: bool,
    editor_color_syntax: bool,
    markdown_warnings: bool,
    ui_theme: String,
    css_theme: String,
    layout_mode: String,
    view_mode: String,
    language: String,
    custom_css_file: String,
}

impl SettingsChangeTracker {
    fn load_current() -> Self {
        let prefs = get_app_preferences();
        let layout_mode = prefs.get_layout_mode();
        Self {
            function_highlighting: prefs.get_function_highlighting(),
            editor_color_syntax: prefs.get_editor_color_syntax(),
            markdown_warnings: prefs.get_markdown_warnings(),
            ui_theme: prefs.get_ui_theme(),
            css_theme: prefs.get_css_theme(),
            layout_mode,
            view_mode: prefs.get_view_mode(),
            language: prefs.get_language(),
            custom_css_file: prefs.get_custom_css_file(),
        }
    }
    
    fn has_changes(&self, original: &OriginalSettings) -> bool {
        self.function_highlighting != original.function_highlighting ||
        self.editor_color_syntax != original.editor_color_syntax ||
        self.markdown_warnings != original.markdown_warnings ||
        self.ui_theme != original.ui_theme ||
        self.css_theme != original.css_theme ||
        self.layout_mode != original.layout_mode ||
        self.view_mode != original.view_mode ||
        self.language != original.language ||
        self.custom_css_file != original.custom_css_file
    }
    
    fn apply_changes(&self, editor: &crate::editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
        let prefs = get_app_preferences();
        
        println!("DEBUG: apply_changes called");
        
        // Store current values for comparison
        let old_ui_theme = prefs.get_ui_theme();
        let old_css_theme = prefs.get_css_theme();
        let old_language = prefs.get_language();
        let old_view_mode = prefs.get_view_mode();
        let old_layout_mode = prefs.get_layout_mode();
        let old_function_highlighting = prefs.get_function_highlighting();
        let old_editor_color_syntax = prefs.get_editor_color_syntax();
        let old_markdown_warnings = prefs.get_markdown_warnings();
        
        println!("DEBUG: Old values - UI theme: {}, CSS theme: {}", old_ui_theme, old_css_theme);
        println!("DEBUG: New values - UI theme: {}, CSS theme: {}", self.ui_theme, self.css_theme);
        
        // Apply all changes to settings
        prefs.set_function_highlighting(self.function_highlighting);
        prefs.set_editor_color_syntax(self.editor_color_syntax);
        prefs.set_markdown_warnings(self.markdown_warnings);
        prefs.set_ui_theme(&self.ui_theme);
        prefs.set_css_theme(&self.css_theme);
        prefs.set_layout_mode(&self.layout_mode);
        prefs.set_view_mode(&self.view_mode);
        prefs.set_language(&self.language);
        prefs.set_custom_css_file(&self.custom_css_file);
        
        println!("DEBUG: Settings saved to GSettings");
        
        // Apply theme changes immediately if they changed
        if old_ui_theme != self.ui_theme {
            println!("DEBUG: UI theme changed, applying change");
            apply_ui_theme_change(&self.ui_theme, theme_manager);
        }
        if old_css_theme != self.css_theme {
            println!("DEBUG: CSS theme changed, applying change");
            apply_css_theme_change(&self.css_theme, editor);
        }
        if old_language != self.language {
            println!("DEBUG: Language changed, applying change");
            apply_language_change(&self.language);
        }
        if old_view_mode != self.view_mode {
            println!("DEBUG: View mode changed, applying change");
            editor.set_view_mode(&self.view_mode);
        }
        if old_layout_mode != self.layout_mode {
            println!("DEBUG: Layout mode changed, applying change");
            if self.layout_mode == "editor-right" {
                editor.set_layout_reversed(true);   // Preview left, editor right
            } else {
                editor.set_layout_reversed(false);  // Editor left, preview right (default)
            }
        }
        if old_function_highlighting != self.function_highlighting {
            println!("DEBUG: Function highlighting changed, applying change");
            editor.set_function_highlighting(self.function_highlighting);
        }
        if old_editor_color_syntax != self.editor_color_syntax {
            println!("DEBUG: Editor color syntax changed, applying change");
            editor.set_editor_color_syntax(self.editor_color_syntax);
        }
        if old_markdown_warnings != self.markdown_warnings {
            println!("DEBUG: Markdown warnings changed, applying change");
            editor.set_markdown_warnings(self.markdown_warnings);
        }
        
        println!("DEBUG: All changes applied successfully");
    }
}

/// Stores original values to allow reverting changes
#[derive(Debug, Clone)]
struct OriginalSettings {
    function_highlighting: bool,
    editor_color_syntax: bool,
    markdown_warnings: bool,
    ui_theme: String,
    css_theme: String,
    layout_mode: String,
    view_mode: String,
    language: String,
    custom_css_file: String,
}

impl OriginalSettings {
    fn load_current() -> Self {
        let prefs = get_app_preferences();
        Self {
            function_highlighting: prefs.get_function_highlighting(),
            editor_color_syntax: prefs.get_editor_color_syntax(),
            markdown_warnings: prefs.get_markdown_warnings(),
            ui_theme: prefs.get_ui_theme(),
            css_theme: prefs.get_css_theme(),
            layout_mode: prefs.get_layout_mode(),
            view_mode: prefs.get_view_mode(),
            language: prefs.get_language(),
            custom_css_file: prefs.get_custom_css_file(),
        }
    }
    
    fn restore(&self) {
        let prefs = get_app_preferences();
        prefs.set_function_highlighting(self.function_highlighting);
        prefs.set_editor_color_syntax(self.editor_color_syntax);
        prefs.set_markdown_warnings(self.markdown_warnings);
        prefs.set_ui_theme(&self.ui_theme);
        prefs.set_css_theme(&self.css_theme);
        prefs.set_layout_mode(&self.layout_mode);
        prefs.set_view_mode(&self.view_mode);
        prefs.set_language(&self.language);
        prefs.set_custom_css_file(&self.custom_css_file);
    }
}

/// Create and show the settings dialog
pub fn show_settings_dialog(parent: &Window, editor: &crate::editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
    println!("DEBUG: Creating settings dialog");
    let dialog = Dialog::builder()
        .title("Settings")
        .transient_for(parent)
        .modal(true)
        .resizable(true)
        .default_width(650)
        .default_height(550)
        .build();

    println!("DEBUG: Settings dialog created, loading original settings");

    // Store original settings for cancel functionality
    let original_settings = OriginalSettings::load_current();
    
    // Create change tracker - start with current values
    let change_tracker = Rc::new(RefCell::new(SettingsChangeTracker::load_current()));
    
    // Flag to track if settings have been saved (to avoid unsaved changes dialog after save)
    let settings_saved = Rc::new(RefCell::new(false));

    // Create main content area
    let content_area = dialog.content_area();
    content_area.set_spacing(0);
    
    // Create notebook for different settings categories
    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_vexpand(true);
    notebook.set_hexpand(true);
    
    // Create button box for Save/Cancel/Reset
    let button_box = Box::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(12);
    button_box.set_margin_bottom(12);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);
    
    // Reset button
    let reset_button = Button::with_label("Reset to Defaults");
    
    // Cancel button (red)
    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("destructive-action");
    
    // Save button (green) - initially disabled
    let save_button = Button::with_label("Save");
    save_button.add_css_class("suggested-action");
    save_button.set_sensitive(false);
    
    // Create settings pages with change tracking
    create_editor_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_appearance_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_layout_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    create_advanced_settings_page(&notebook, &change_tracker, &save_button, &original_settings);
    
    // Add notebook to dialog
    content_area.append(&notebook);
    
    // Add buttons to button box
    button_box.append(&reset_button);
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_area.append(&button_box);
    
    // Connect reset button
    reset_button.connect_clicked({
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        move |_| {
            show_reset_confirmation_dialog(&dialog, &change_tracker);
        }
    });
    
    // Connect cancel button
    cancel_button.connect_clicked({
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let original_settings = original_settings.clone();
        move |_| {
            println!("DEBUG: Cancel button clicked in main settings dialog");
            handle_cancel_request(&dialog, &change_tracker, &original_settings);
        }
    });
    
    // Connect save button
    save_button.connect_clicked({
        let dialog = dialog.clone();
        let change_tracker = change_tracker.clone();
        let settings_saved = settings_saved.clone();
        let editor = editor.clone();
        let theme_manager = theme_manager.clone();
        move |_| {
            println!("DEBUG: Save button clicked in main settings dialog");
            handle_save_request(&dialog, &change_tracker, &settings_saved, &editor, &theme_manager);
        }
    });
    
    // Handle close request (X button)
    dialog.connect_close_request({
        let change_tracker = change_tracker.clone();
        let original_settings = original_settings.clone();
        let settings_saved = settings_saved.clone();
        move |dialog| {
            println!("DEBUG: Close request (X button) received");
            // If settings were just saved, allow closing without checking for changes
            if *settings_saved.borrow() {
                println!("DEBUG: Settings were saved, allowing close");
                glib::Propagation::Proceed
            } else if change_tracker.borrow().has_changes(&original_settings) {
                println!("DEBUG: Changes detected, preventing close and showing dialog");
                handle_close_request(dialog, &change_tracker, &original_settings);
                glib::Propagation::Stop
            } else {
                println!("DEBUG: No changes, allowing close");
                glib::Propagation::Proceed
            }
        }
    });
    
    // Apply custom CSS
    crate::settings::ui::apply_settings_css();
    dialog.add_css_class("settings-dialog");
    
    println!("DEBUG: Showing settings dialog");
    dialog.show();
}

/// Handle reset to defaults request
fn show_reset_confirmation_dialog(parent: &Dialog, change_tracker: &Rc<RefCell<SettingsChangeTracker>>) {
    let confirm_dialog = Dialog::builder()
        .title("Reset Settings")
        .transient_for(parent)
        .modal(true)
        .build();
    
    let content = confirm_dialog.content_area();
    let message = Label::new(Some("Are you sure you want to reset all settings to their default values? This action cannot be undone."));
    message.set_wrap(true);
    message.set_margin_top(12);
    message.set_margin_bottom(12);
    message.set_margin_start(12);
    message.set_margin_end(12);
    content.append(&message);
    
    confirm_dialog.add_button("Cancel", ResponseType::Cancel);
    confirm_dialog.add_button("Reset", ResponseType::Ok);
    
    confirm_dialog.connect_response({
        let parent = parent.clone();
        let change_tracker = change_tracker.clone();
        move |confirm_dialog, response| {
            confirm_dialog.close();
            if response == ResponseType::Ok {
                let prefs = get_app_preferences();
                prefs.reset_to_defaults();
                
                println!("Settings have been reset to defaults");
                
                // Update change tracker with new defaults
                *change_tracker.borrow_mut() = SettingsChangeTracker::load_current();
                parent.close();
            }
        }
    });
    
    confirm_dialog.show();
}

/// Handle cancel request
fn handle_cancel_request(dialog: &Dialog, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, original_settings: &OriginalSettings) {
    println!("DEBUG: Handle cancel request called");
    if change_tracker.borrow().has_changes(original_settings) {
        println!("DEBUG: Changes detected, showing unsaved changes dialog");
        show_unsaved_changes_dialog(dialog, change_tracker, original_settings);
    } else {
        println!("DEBUG: No changes, closing settings dialog");
        dialog.close();
    }
}

/// Handle save request
fn handle_save_request(
    dialog: &Dialog, 
    change_tracker: &Rc<RefCell<SettingsChangeTracker>>, 
    settings_saved: &Rc<RefCell<bool>>,
    editor: &crate::editor::MarkdownEditor,
    theme_manager: &crate::theme::ThemeManager,
) {
    println!("DEBUG: Handle save request called");
    // Apply the changes to the settings and immediately to the application
    change_tracker.borrow().apply_changes(editor, theme_manager);
    
    // Reset the change tracker to the new current values to avoid "unsaved changes" dialog
    *change_tracker.borrow_mut() = SettingsChangeTracker::load_current();
    
    // Mark that settings have been saved
    *settings_saved.borrow_mut() = true;
    
    println!("Settings saved successfully!");
    println!("DEBUG: Closing settings dialog after save");
    dialog.close();
}

/// Handle close request (X button)
fn handle_close_request(dialog: &Dialog, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, original_settings: &OriginalSettings) {
    println!("DEBUG: Handle close request (X button) called");
    if change_tracker.borrow().has_changes(original_settings) {
        println!("DEBUG: Changes detected on close, showing unsaved changes dialog");
        show_unsaved_changes_dialog(dialog, change_tracker, original_settings);
    } else {
        println!("DEBUG: No changes on close, allowing dialog to close");
        // This shouldn't be reached as the close_request handler only calls this if there are changes
        dialog.close();
    }
}

/// Show dialog when there are unsaved changes - simplified version
fn show_unsaved_changes_dialog(parent: &Dialog, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, original_settings: &OriginalSettings) {
    println!("DEBUG: Creating unsaved changes dialog");
    let confirm_dialog = Dialog::builder()
        .title("Unsaved Changes")
        .transient_for(parent)
        .modal(true)
        .resizable(false)
        .default_width(400)
        .default_height(150)
        .build();
    
    println!("DEBUG: Unsaved changes dialog created");
    
    // Apply the same CSS class as the settings dialog
    confirm_dialog.add_css_class("settings-dialog");
    
    // Create content
    let content = confirm_dialog.content_area();
    content.set_spacing(16);
    
    let message = Label::new(Some("You have unsaved changes. What would you like to do?"));
    message.set_wrap(true);
    message.set_halign(gtk4::Align::Center);
    message.set_margin_top(16);
    message.set_margin_bottom(16);
    message.set_margin_start(16);
    message.set_margin_end(16);
    content.append(&message);
    
    // Create button box with same styling as settings dialog
    let button_box = Box::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(8);
    button_box.set_margin_bottom(16);
    button_box.set_margin_start(16);
    button_box.set_margin_end(16);
    
    // Don't Save button (red, destructive)
    let dont_save_button = Button::with_label("Don't Save");
    dont_save_button.add_css_class("destructive-action");
    
    // Cancel button (neutral)
    let cancel_button = Button::with_label("Cancel");
    
    // Add buttons to button box
    button_box.append(&dont_save_button);
    button_box.append(&cancel_button);
    content.append(&button_box);
    
    // Connect button signals directly
    dont_save_button.connect_clicked({
        let confirm_dialog = confirm_dialog.clone();
        let parent = parent.clone();
        let original_settings = original_settings.clone();
        let change_tracker = change_tracker.clone();
        move |_| {
            println!("DEBUG: Don't Save button clicked");
            // Don't Save: Restore original settings and reset change tracker
            original_settings.restore();
            println!("DEBUG: Original settings restored");
            
            // Reset the change tracker to match the restored settings
            *change_tracker.borrow_mut() = SettingsChangeTracker::load_current();
            println!("DEBUG: Change tracker reset to current settings");
            
            confirm_dialog.close();
            println!("DEBUG: Closing confirmation dialog");
            parent.close();
            println!("DEBUG: Closing parent settings dialog");
            println!("DEBUG: Don't Save operation completed");
        }
    });
    
    cancel_button.connect_clicked({
        let confirm_dialog = confirm_dialog.clone();
        move |_| {
            println!("DEBUG: Cancel button clicked in unsaved changes dialog");
            // Cancel: Just close the confirmation dialog, keep the settings dialog open
            confirm_dialog.close();
            println!("DEBUG: Confirmation dialog closed, settings dialog remains open");
        }
    });
    
    println!("DEBUG: Showing unsaved changes dialog");
    confirm_dialog.show();
}

/// Create the editor settings page
fn create_editor_settings_page(notebook: &Notebook, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, save_button: &Button, original_settings: &OriginalSettings) {
    let page_box = Box::new(Orientation::Vertical, 16);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(24);
    page_box.set_margin_end(24);
    page_box.add_css_class("settings-page");
    
    // Function highlighting section
    let function_section = create_settings_section_header(
        "Function Highlighting",
        Some("Visual emphasis for function-related areas when hovering")
    );
    page_box.append(&function_section);
    
    let function_switch = Switch::new();
    
    // Set initial value from change tracker
    function_switch.set_active(change_tracker.borrow().function_highlighting);
    
    // Connect change tracking
    function_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            // Update the change tracker with new value
            change_tracker.borrow_mut().function_highlighting = switch.is_active();
            
            // Check if there are changes and enable/disable save button
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    
    let function_row = create_settings_row(
        "Enable function highlighting",
        &function_switch,
        Some("Highlights function-related text when hovering over functions")
    );
    page_box.append(&function_row);
    
    // Syntax color section
    let syntax_color_section = create_settings_section_header(
        "Syntax Color",
        Some("Enable syntax highlighting in the markdown editor")
    );
    page_box.append(&syntax_color_section);
    
    let syntax_color_switch = Switch::new();
    
    // Set initial value from change tracker
    syntax_color_switch.set_active(change_tracker.borrow().editor_color_syntax);
    
    // Connect change tracking
    syntax_color_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            // Update the change tracker with new value
            change_tracker.borrow_mut().editor_color_syntax = switch.is_active();
            
            // Check if there are changes and enable/disable save button
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    
    let syntax_color_row = create_settings_row(
        "Enable syntax color",
        &syntax_color_switch,
        Some("Apply syntax highlighting colors to markdown text in the editor")
    );
    page_box.append(&syntax_color_row);
    
    // Markdown warnings section
    let markdown_section = create_settings_section_header(
        "Markdown Warnings",
        Some("Show warnings for malformed Markdown syntax")
    );
    page_box.append(&markdown_section);
    
    let markdown_switch = Switch::new();
    
    // Set initial value from change tracker
    markdown_switch.set_active(change_tracker.borrow().markdown_warnings);
    
    // Connect change tracking
    markdown_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            // Update the change tracker with new value
            change_tracker.borrow_mut().markdown_warnings = switch.is_active();
            
            // Check if there are changes and enable/disable save button
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    
    let markdown_row = create_settings_row(
        "Show Markdown warnings",
        &markdown_switch,
        Some("Display warnings for syntax errors and formatting issues")
    );
    page_box.append(&markdown_row);
    
    // Add expandable explanation of warning types
    let expander = gtk4::Expander::new(Some("What do the different warning types mean?"));
    expander.set_margin_top(8);
    expander.set_margin_bottom(8);
    expander.set_margin_start(20);
    expander.set_margin_end(20);
    
    let explanation_box = Box::new(Orientation::Vertical, 12);
    explanation_box.set_margin_top(12);
    explanation_box.set_margin_bottom(12);
    explanation_box.set_margin_start(8);
    explanation_box.set_margin_end(8);
    
    // Create explanation text with different warning types
    let warning_types = vec![
        ("🔴", "Syntax Errors", "Broken links, unclosed code blocks, malformed tables"),
        ("🟠", "Formatting Issues", "Missing alt text, empty links, improper headings"),
        ("🟡", "Style Warnings", "Raw HTML usage, inconsistent list markers"),
        ("🔵", "Structure Issues", "Invalid references, unclosed emphasis markers"),
    ];
    
    for (icon, category, description) in warning_types {
        let warning_box = Box::new(Orientation::Horizontal, 12);
        warning_box.set_margin_bottom(6);
        
        let icon_label = Label::new(Some(icon));
        icon_label.set_halign(gtk4::Align::Center);
        icon_label.set_valign(gtk4::Align::Start);
        icon_label.set_size_request(24, -1);
        
        let content_box = Box::new(Orientation::Vertical, 2);
        
        let category_label = Label::new(Some(category));
        category_label.set_markup(&format!("<b>{}</b>", category));
        category_label.set_halign(gtk4::Align::Start);
        
        let description_label = Label::new(Some(description));
        description_label.set_halign(gtk4::Align::Start);
        description_label.set_wrap(true);
        description_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
        description_label.add_css_class("dim-label");
        
        content_box.append(&category_label);
        content_box.append(&description_label);
        
        warning_box.append(&icon_label);
        warning_box.append(&content_box);
        explanation_box.append(&warning_box);
    }
    
    // Add examples section
    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    separator.set_margin_top(8);
    separator.set_margin_bottom(8);
    explanation_box.append(&separator);
    
    let examples_header = Label::new(Some("Examples of warnings:"));
    examples_header.set_markup("<b>Examples of warnings:</b>");
    examples_header.set_halign(gtk4::Align::Start);
    examples_header.set_margin_bottom(6);
    explanation_box.append(&examples_header);
    
    let examples = vec![
        "##NoSpace",
        "[broken link](",
        "![](image.png)",
        "```rust",
        "**bold text"
    ];
    
    let examples_descriptions = vec![
        "missing space after heading hash",
        "incomplete link syntax",
        "missing alt text",
        "unclosed code block",
        "unclosed emphasis"
    ];
    
    for (example, desc) in examples.iter().zip(examples_descriptions.iter()) {
        let example_box = Box::new(Orientation::Horizontal, 8);
        example_box.set_margin_bottom(2);
        example_box.set_margin_start(8);
        
        let bullet_label = Label::new(Some("•"));
        bullet_label.set_halign(gtk4::Align::Start);
        bullet_label.set_size_request(12, -1);
        
        let example_label = Label::new(Some(example));
        example_label.set_halign(gtk4::Align::Start);
        example_label.add_css_class("monospace");
        example_label.set_size_request(120, -1);
        
        let desc_label = Label::new(Some(desc));
        desc_label.set_halign(gtk4::Align::Start);
        desc_label.add_css_class("dim-label");
        
        example_box.append(&bullet_label);
        example_box.append(&example_label);
        example_box.append(&desc_label);
        explanation_box.append(&example_box);
    }
    
    expander.set_child(Some(&explanation_box));
    page_box.append(&expander);
    
    // Add page to notebook
    let label = Label::new(Some("Editor"));
    notebook.append_page(&page_box, Some(&label));
}

/// Create the appearance settings page
fn create_appearance_settings_page(notebook: &Notebook, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, save_button: &Button, original_settings: &OriginalSettings) {
    let page_box = Box::new(Orientation::Vertical, 16);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(24);
    page_box.set_margin_end(24);
    page_box.add_css_class("settings-page");
    
    // UI Theme section
    let ui_theme_section = create_settings_section_header(
        "UI Theme",
        Some("Choose the application's color scheme")
    );
    page_box.append(&ui_theme_section);
    
    let ui_theme_combo = ComboBoxText::new();
    ui_theme_combo.append(Some("system"), "Follow System");
    ui_theme_combo.append(Some("light"), "Light");
    ui_theme_combo.append(Some("dark"), "Dark");
    
    let current_ui_theme = &change_tracker.borrow().ui_theme;
    ui_theme_combo.set_active_id(Some(current_ui_theme));
    
    ui_theme_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                // Update the change tracker with new value
                change_tracker.borrow_mut().ui_theme = selected.to_string();
                
                // Check if there are changes and enable/disable save button
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    
    let ui_theme_row = create_settings_row(
        "Application theme",
        &ui_theme_combo,
        Some("System will follow your desktop environment's theme")
    );
    page_box.append(&ui_theme_row);
    
    // CSS Theme section
    let css_theme_section = create_settings_section_header(
        "Preview Theme",
        Some("Choose the CSS theme for Markdown preview")
    );
    page_box.append(&css_theme_section);
    
    let css_theme_combo = ComboBoxText::new();
    let available_themes = get_available_css_themes();
    for theme in &available_themes {
        css_theme_combo.append(Some(theme), theme);
    }
    
    let current_css_theme = &change_tracker.borrow().css_theme;
    css_theme_combo.set_active_id(Some(current_css_theme));
    
    css_theme_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                // Update the change tracker with new value
                change_tracker.borrow_mut().css_theme = selected.to_string();
                
                // Check if there are changes and enable/disable save button
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    
    let css_theme_row = create_settings_row(
        "Preview theme",
        &css_theme_combo,
        Some("CSS theme used for rendering the Markdown preview")
    );
    page_box.append(&css_theme_row);
    
    // Language section
    let language_section = create_settings_section_header(
        "Language",
        Some("Choose the interface language")
    );
    page_box.append(&language_section);
    
    let language_combo = ComboBoxText::new();
    let available_languages = get_available_languages();
    for (code, name) in &available_languages {
        language_combo.append(Some(code), name);
    }
    
    let current_language = &change_tracker.borrow().language;
    language_combo.set_active_id(Some(current_language));
    
    language_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                // Update the change tracker with new value
                change_tracker.borrow_mut().language = selected.to_string();
                
                // Check if there are changes and enable/disable save button
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    
    let language_row = create_settings_row(
        "Interface language",
        &language_combo,
        Some("Language for the application interface")
    );
    page_box.append(&language_row);
    
    // Add page to notebook
    let label = Label::new(Some("Appearance"));
    notebook.append_page(&page_box, Some(&label));
}

/// Create the layout settings page
fn create_layout_settings_page(notebook: &Notebook, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, save_button: &Button, original_settings: &OriginalSettings) {
    let page_box = Box::new(Orientation::Vertical, 16);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(24);
    page_box.set_margin_end(24);
    page_box.add_css_class("settings-page");
    
    // Layout mode section
    let layout_section = create_settings_section_header(
        "Layout Mode",
        Some("Choose the editor and preview layout")
    );
    page_box.append(&layout_section);
    
    // Create radio buttons for layout options using CheckButton
    let layout_box = Box::new(Orientation::Vertical, 8);
    
    // First radio button (Editor Left, Preview Right)
    let editor_left_radio = CheckButton::with_label("Editor Left, Preview Right");
    
    // Second radio button (Editor Right, Preview Left) - group it with the first
    let editor_right_radio = CheckButton::with_label("Editor Right, Preview Left");
    editor_right_radio.set_group(Some(&editor_left_radio));
    
    // Set initial state based on current setting
    let current_layout = &change_tracker.borrow().layout_mode;
    
    if current_layout == "editor-left" {
        editor_left_radio.set_active(true);
    } else {
        editor_right_radio.set_active(true);
    }
    
    // Connect signal handlers for change tracking
    editor_left_radio.connect_toggled({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            if button.is_active() {
                // Update the change tracker
                change_tracker.borrow_mut().layout_mode = "editor-left".to_string();
                
                // Check if there are changes and enable/disable save button
                let has_changes = change_tracker.borrow().has_changes(&original_settings);
                save_button.set_sensitive(has_changes);
            }
        }
    });
    
    editor_right_radio.connect_toggled({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            if button.is_active() {
                // Update the change tracker
                change_tracker.borrow_mut().layout_mode = "editor-right".to_string();
                
                // Check if there are changes and enable/disable save button
                let has_changes = change_tracker.borrow().has_changes(&original_settings);
                save_button.set_sensitive(has_changes);
            }
        }
    });
    
    // Add radio buttons to container
    layout_box.append(&editor_left_radio);
    layout_box.append(&editor_right_radio);
    
    let layout_row = create_settings_row(
        "Layout mode",
        &layout_box,
        Some("Choose whether the editor or preview appears on the left side")
    );
    page_box.append(&layout_row);
    
    // View mode section
    let view_section = create_settings_section_header(
        "View Mode",
        Some("Choose the default view mode")
    );
    page_box.append(&view_section);
    
    let view_combo = ComboBoxText::new();
    view_combo.append(Some("html"), "HTML Preview");
    view_combo.append(Some("code"), "Source Code");
    
    let current_view = &change_tracker.borrow().view_mode;
    view_combo.set_active_id(Some(current_view));
    
    view_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                // Update the change tracker with new value
                change_tracker.borrow_mut().view_mode = selected.to_string();
                
                // Check if there are changes and enable/disable save button
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    
    let view_row = create_settings_row(
        "Default view mode",
        &view_combo,
        Some("Default view mode when opening files")
    );
    page_box.append(&view_row);
    
    // Add page to notebook
    let label = Label::new(Some("Layout"));
    notebook.append_page(&page_box, Some(&label));
}

/// Create the advanced settings page
fn create_advanced_settings_page(notebook: &Notebook, change_tracker: &Rc<RefCell<SettingsChangeTracker>>, save_button: &Button, original_settings: &OriginalSettings) {
    let page_box = Box::new(Orientation::Vertical, 16);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(24);
    page_box.set_margin_end(24);
    page_box.add_css_class("settings-page");
    
    // Settings info section
    let info_section = create_settings_section_header(
        "Settings Information",
        Some("Information about the settings system")
    );
    page_box.append(&info_section);
    
    let settings_info = "Settings are stored using GSettings.\nSettings will be preserved across application updates and are managed by the desktop environment.";
    
    let info_label = Label::new(Some(settings_info));
    info_label.set_halign(gtk4::Align::Start);
    info_label.set_wrap(true);
    info_label.add_css_class("dim-label");
    info_label.set_margin_bottom(16);
    page_box.append(&info_label);
    
    // Custom CSS file section
    let css_section = create_settings_section_header(
        "Custom CSS",
        Some("Override preview styling with a custom CSS file")
    );
    page_box.append(&css_section);
    
    let css_button = Button::with_label("Select CSS File");
    let current_css = &change_tracker.borrow().custom_css_file;
    
    if !current_css.is_empty() {
        css_button.set_label(&format!("CSS File: {}", current_css));
    }
    
    css_button.connect_clicked({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            let file_chooser = gtk4::FileChooserDialog::new(
                Some("Select CSS File"),
                Some(button.root().unwrap().downcast_ref::<Window>().unwrap()),
                gtk4::FileChooserAction::Open,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Select", ResponseType::Accept),
                ]
            );
            
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("CSS Files"));
            filter.add_pattern("*.css");
            file_chooser.add_filter(&filter);
            
            let button_clone = button.clone();
            let change_tracker = change_tracker.clone();
            let save_button = save_button.clone();
            let original_settings = original_settings.clone();
            file_chooser.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_str().unwrap_or("");
                            
                            // Update the change tracker with new value
                            change_tracker.borrow_mut().custom_css_file = path_str.to_string();
                            button_clone.set_label(&format!("CSS File: {}", path.display()));
                            
                            // Check if there are changes and enable/disable save button
                            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
                        }
                    }
                }
                dialog.close();
            });
            
            file_chooser.show();
        }
    });
    
    let css_row = create_settings_row(
        "Custom CSS file",
        &css_button,
        Some("Path to a custom CSS file to override preview styling")
    );
    page_box.append(&css_row);
    
    // Add page to notebook
    let label = Label::new(Some("Advanced"));
    notebook.append_page(&page_box, Some(&label));
}

/// Apply UI theme change immediately
fn apply_ui_theme_change(theme: &str, theme_manager: &crate::theme::ThemeManager) {
    use crate::theme::Theme;
    
    println!("DEBUG: Applying UI theme change to: {}", theme);
    
    // Apply theme change through theme manager
    let theme_enum = match theme {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        "system" => Theme::System,
        _ => Theme::System,
    };
    
    theme_manager.set_theme(theme_enum);
    println!("DEBUG: UI theme change applied to: {}", theme);
}

/// Apply CSS theme change immediately 
fn apply_css_theme_change(theme: &str, editor: &crate::editor::MarkdownEditor) {
    println!("DEBUG: Applying CSS theme change to: {}", theme);
    editor.set_css_theme(theme);
    println!("DEBUG: CSS theme change applied to: {}", theme);
}

/// Apply language change immediately
fn apply_language_change(language: &str) {
    crate::language::set_locale(language);
    println!("Applied language change to: {}", language);
}
