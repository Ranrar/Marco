use marco_core::logic::swanson::EditorSettings;
use gtk4::prelude::*;
use gtk4::Box;
use log::{debug, error};

// Import unified helper
use super::helpers::add_setting_row;

pub fn build_editor_tab(settings_path: &str) -> Box {
    use gtk4::{
        Adjustment, Align, Box as GtkBox, DropDown, Orientation, SpinButton,
        Switch, StringList, PropertyExpression, StringObject, Expression,
    };

    // Initialize SettingsManager for this editor tab
    let settings_manager_opt = match marco_core::logic::swanson::SettingsManager::initialize(std::path::PathBuf::from(settings_path)) {
        Ok(settings_manager) => Some(std::sync::Arc::new(settings_manager)),
        Err(e) => {
            debug!("Failed to initialize SettingsManager for editor settings: {}", e);
            None
        }
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Editor Font (Dropdown with actual system fonts)
    // Load current font setting from SettingsManager
    let current_font = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        let editor = settings.editor.unwrap_or_default();
        editor.font.unwrap_or_else(|| "Monospace".to_string())
    } else {
        "Monospace".to_string()
    };
    debug!("Current font setting: {}", current_font);

    // Get monospace fonts (preferred for code editing) - using cached fonts
    let monospace_fonts =
        marco_core::logic::loaders::font_loader::FontLoader::get_cached_monospace_fonts();
    let monospace_names: Vec<String> = monospace_fonts.into_iter().map(|f| f.name).collect();

    // Create searchable DropDown with font list
    // Step 1: Create StringList from font names
    let font_string_refs: Vec<&str> = monospace_names.iter().map(|s| s.as_str()).collect();
    let font_string_list = StringList::new(&font_string_refs);

    // Step 2: Create PropertyExpression for search functionality
    let font_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<Expression>,
        "string",
    );

    // Step 3: Create DropDown with search enabled
    let font_combo = DropDown::new(Some(font_string_list), Some(font_expression));
    font_combo.set_enable_search(true);
    font_combo.add_css_class("marco-dropdown");
    font_combo.set_halign(Align::End);
    // Note: set_search_match_mode may require newer GTK4 version, using default for now

    // Set initial selection based on current font
    let initial_selection = monospace_names
        .iter()
        .position(|font_name| font_name == &current_font)
        .unwrap_or(0); // Default to first font if current not found
    font_combo.set_selected(initial_selection as u32);

    // Connect font selection change to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        let monospace_names_clone = monospace_names.clone();
        
        font_combo.connect_selected_notify(move |combo| {
            let selected_index = combo.selected() as usize;
            if let Some(selected_font) = monospace_names_clone.get(selected_index) {
                let selected_font = selected_font.clone();
                debug!("Font changed to: {}", selected_font);

                // Update font setting using SettingsManager
                if let Err(e) = settings_manager_clone.update_settings(|settings| {
                    // Ensure editor settings exist
                    if settings.editor.is_none() {
                        settings.editor = Some(EditorSettings::default());
                    }

                    // Update font setting
                    if let Some(ref mut editor) = settings.editor {
                        editor.font = Some(selected_font.clone());
                    }
                }) {
                    error!("Failed to save font setting: {}", e);
                    return;
                }

                // Get updated settings for runtime update
                let settings = settings_manager_clone.get_settings();
                let editor = settings.editor.unwrap_or_default();
                let editor_settings =
                    crate::components::editor::font_config::EditorDisplaySettings {
                        font_family: selected_font,
                        font_size: editor.font_size.unwrap_or(14),
                        line_height: editor.line_height.unwrap_or(1.4),
                        line_wrapping: editor.line_wrapping.unwrap_or(false),
                        show_invisibles: editor.show_invisibles.unwrap_or(false),
                        tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                        syntax_colors: editor.syntax_colors.unwrap_or(true),
                        show_line_numbers: settings
                            .layout
                            .as_ref()
                            .and_then(|l| l.show_line_numbers)
                            .unwrap_or(true),
                    };

                if let Err(e) =
                    crate::components::editor::editor_manager::update_editor_settings_globally(
                        &editor_settings,
                    )
                {
                    error!("Failed to update font settings globally: {}", e);
                } else {
                    debug!("Successfully updated font settings globally");
                }
            }
        });
    }

    // Create font row using unified helper (first row)
    let font_row = add_setting_row(
        "Font",
        "Select the font used in the editor.",
        &font_combo,
        true  // First row - no top margin
    );
    container.append(&font_row);

    // Font Size (SpinButton) with actual settings
    // Load current font size from SettingsManager
    let current_font_size = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        let editor = settings.editor.unwrap_or_default();
        editor.font_size.unwrap_or(14) as f64
    } else {
        14.0
    };

    let font_size_adj = Adjustment::new(current_font_size, 10.0, 24.0, 1.0, 0.0, 0.0);
    let font_size_spin = SpinButton::new(Some(&font_size_adj), 1.0, 0);
    font_size_spin.add_css_class("marco-spinbutton");

    // Connect font size changes to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        font_size_adj.connect_value_changed(move |adj| {
            let new_size = adj.value() as u8;
            debug!("Font size changed to: {}px", new_size);

            // Update font size setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update font size setting
                if let Some(ref mut editor) = settings.editor {
                    editor.font_size = Some(new_size);
                }
            }) {
                error!("Failed to save font size setting: {}", e);
                return;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: new_size,
                line_height: editor.line_height.unwrap_or(1.4),
                line_wrapping: editor.line_wrapping.unwrap_or(false),
                show_invisibles: editor.show_invisibles.unwrap_or(false),
                tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                syntax_colors: editor.syntax_colors.unwrap_or(true),
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update font size settings globally: {}", e);
            } else {
                debug!("Successfully updated font size settings globally");
            }
        });
    }

    // Create font size row using unified helper
    let font_size_row = add_setting_row(
        "Font Size",
        "Set the font size for the editor text (10-24 px).",
        &font_size_spin,
        false  // Not first row
    );
    container.append(&font_size_row);

    // Line Height (SpinButton)
    // Load current line height from SettingsManager
    let current_line_height = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings.editor
            .and_then(|e| e.line_height)
            .unwrap_or(1.4) as f64
    } else {
        1.4
    };

    let line_height_adj = Adjustment::new(current_line_height, 1.0, 2.0, 0.05, 0.0, 0.0);
    let line_height_spin = SpinButton::new(Some(&line_height_adj), 0.05, 2);
    line_height_spin.add_css_class("marco-spinbutton");

    // Connect line height changes to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        line_height_adj.connect_value_changed(move |adj| {
            let new_line_height = adj.value() as f32;
            debug!("Line height changed to: {}", new_line_height);

            // Update line height setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(marco_core::logic::swanson::EditorSettings::default());
                }

                // Update line height setting
                if let Some(ref mut editor) = settings.editor {
                    editor.line_height = Some(new_line_height);
                }
            }) {
                error!("Failed to save line height setting: {}", e);
                return;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: editor.font_size.unwrap_or(14),
                line_height: new_line_height,
                line_wrapping: editor.line_wrapping.unwrap_or(false),
                show_invisibles: editor.show_invisibles.unwrap_or(false),
                tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                syntax_colors: editor.syntax_colors.unwrap_or(true),
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update line height settings globally: {}", e);
            } else {
                debug!("Successfully updated line height settings globally");
            }
        });
    }

    // Create line height row using unified helper
    let line_height_row = add_setting_row(
        "Line Height",
        "Adjust the vertical spacing between lines.",
        &line_height_spin,
        false  // Not first row
    );
    container.append(&line_height_row);

    // Line Wrapping (Toggle)
    let line_wrap_switch = Switch::new();
    line_wrap_switch.add_css_class("marco-switch");

    // Load current line wrapping setting from SettingsManager
    let current_line_wrapping = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings.editor
            .and_then(|e| e.line_wrapping)
            .unwrap_or(true) // Default to true for better UX
    } else {
        true
    };
    line_wrap_switch.set_active(current_line_wrapping);

    // Connect line wrapping changes to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        line_wrap_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Line wrapping changed to: {}", enabled);

            // Update line wrapping setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update line wrapping setting
                if let Some(ref mut editor) = settings.editor {
                    editor.line_wrapping = Some(enabled);
                }
            }) {
                error!("Failed to save line wrapping setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: editor.font_size.unwrap_or(14),
                line_height: editor.line_height.unwrap_or(1.4),
                line_wrapping: enabled,
                show_invisibles: editor.show_invisibles.unwrap_or(false),
                tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                syntax_colors: editor.syntax_colors.unwrap_or(true),
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update line wrapping settings globally: {}", e);
            } else {
                debug!("Successfully updated line wrapping settings globally");
            }

            glib::Propagation::Proceed
        });
    }

    // Create line wrapping row using unified helper
    let line_wrap_row = add_setting_row(
        "Line Wrapping",
        "Wrap long lines to fit within the editor window.",
        &line_wrap_switch,
        false  // Not first row
    );
    container.append(&line_wrap_row);

    // Auto Pairing (Toggle)
    let auto_pair_switch = Switch::new();
    auto_pair_switch.add_css_class("marco-switch");

    // Create auto pairing row using unified helper
    let auto_pair_row = add_setting_row(
        "Auto Pairing",
        "Automatically insert closing characters for **, [], (), and backticks.",
        &auto_pair_switch,
        false  // Not first row
    );
    container.append(&auto_pair_row);

    // Show Invisible Characters (Toggle)
    let show_invis_switch = Switch::new();
    show_invis_switch.add_css_class("marco-switch");

    // Load current show invisible characters setting from SettingsManager
    let current_show_invisibles = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings.editor
            .and_then(|e| e.show_invisibles)
            .unwrap_or(false)
    } else {
        false
    };
    show_invis_switch.set_active(current_show_invisibles);

    // Connect show invisibles changes to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        show_invis_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Show invisibles changed to: {}", enabled);

            // Update show invisibles setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update show invisibles setting
                if let Some(ref mut editor) = settings.editor {
                    editor.show_invisibles = Some(enabled);
                }
            }) {
                error!("Failed to save show invisibles setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: editor.font_size.unwrap_or(14),
                line_height: editor.line_height.unwrap_or(1.4),
                line_wrapping: editor.line_wrapping.unwrap_or(false),
                show_invisibles: enabled,
                tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                syntax_colors: editor.syntax_colors.unwrap_or(true),
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update show invisibles settings globally: {}", e);
            } else {
                debug!("Successfully updated show invisibles settings globally");
            }

            glib::Propagation::Proceed
        });
    }

    // Create show invisibles row using unified helper
    let show_invis_row = add_setting_row(
        "Show Invisible Characters",
        "Display tabs, spaces, and newlines visually in the editor.",
        &show_invis_switch,
        false  // Not first row
    );
    container.append(&show_invis_row);

    // Convert Tabs to Spaces (Toggle)
    let tabs_to_spaces_switch = Switch::new();
    tabs_to_spaces_switch.add_css_class("marco-switch");

    // Load current tabs to spaces setting from SettingsManager
    let current_tabs_to_spaces = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings.editor
            .and_then(|e| e.tabs_to_spaces)
            .unwrap_or(false)
    } else {
        false
    };
    tabs_to_spaces_switch.set_active(current_tabs_to_spaces);

    // Connect tabs to spaces changes to save settings and trigger runtime updates
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        tabs_to_spaces_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Tabs to spaces changed to: {}", enabled);

            // Update tabs to spaces setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update tabs to spaces setting
                if let Some(ref mut editor) = settings.editor {
                    editor.tabs_to_spaces = Some(enabled);
                }
            }) {
                error!("Failed to save tabs to spaces setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: editor.font_size.unwrap_or(14),
                line_height: editor.line_height.unwrap_or(1.4),
                line_wrapping: editor.line_wrapping.unwrap_or(false),
                show_invisibles: editor.show_invisibles.unwrap_or(false),
                tabs_to_spaces: enabled,
                syntax_colors: editor.syntax_colors.unwrap_or(true),
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update tabs to spaces settings globally: {}", e);
            } else {
                debug!("Successfully updated tabs to spaces settings globally");
            }

            glib::Propagation::Proceed
        });
    }

    // Create tabs to spaces row using unified helper
    let tabs_to_spaces_row = add_setting_row(
        "Convert Tabs to Spaces",
        "Replace tab characters with spaces.",
        &tabs_to_spaces_switch,
        false  // Not first row
    );
    container.append(&tabs_to_spaces_row);

    // Syntax Colors (Toggle)
    let syntax_colors_switch = Switch::new();
    syntax_colors_switch.add_css_class("marco-switch");

    // Load current syntax colors setting from SettingsManager
    let current_syntax_colors = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings.editor
            .and_then(|e| e.syntax_colors)
            .unwrap_or(true)
    } else {
        true
    };
    syntax_colors_switch.set_active(current_syntax_colors);

    // Wire to save setting when toggled
    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        syntax_colors_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Syntax colors changed to: {}", enabled);

            // Update syntax colors setting using SettingsManager
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(marco_core::logic::swanson::EditorSettings::default());
                }

                // Update syntax colors setting
                if let Some(ref mut editor) = settings.editor {
                    editor.syntax_colors = Some(enabled);
                }
            }) {
                error!("Failed to save syntax colors setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Get updated settings for runtime update
            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings = crate::components::editor::font_config::EditorDisplaySettings {
                font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
                font_size: editor.font_size.unwrap_or(14),
                line_height: editor.line_height.unwrap_or(1.4),
                line_wrapping: editor.line_wrapping.unwrap_or(false),
                show_invisibles: editor.show_invisibles.unwrap_or(false),
                tabs_to_spaces: editor.tabs_to_spaces.unwrap_or(false),
                syntax_colors: enabled,
                show_line_numbers: settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true),
            };

            if let Err(e) =
                crate::components::editor::editor_manager::update_editor_settings_globally(
                    &editor_settings,
                )
            {
                error!("Failed to update syntax colors settings globally: {}", e);
            } else {
                debug!("Successfully updated syntax colors settings globally");
            }

            glib::Propagation::Proceed
        });
    }

    // Create syntax colors row using unified helper
    let syntax_colors_row = add_setting_row(
        "Syntax Colors",
        "Enable or disable syntax-based color highlighting for Markdown.",
        &syntax_colors_switch,
        false  // Not first row
    );
    container.append(&syntax_colors_row);

    // Enable Markdown Linting (Toggle)
    let linting_switch = Switch::new();
    linting_switch.add_css_class("marco-switch");

    // Create linting row using unified helper
    let linting_row = add_setting_row(
        "Enable Markdown Linting",
        "Check for Markdown syntax issues and style problems.",
        &linting_switch,
        false  // Not first row
    );
    container.append(&linting_row);

    container
}
