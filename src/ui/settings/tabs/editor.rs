use crate::logic::swanson::{EditorSettings, Settings as AppSettings};
use gtk4::prelude::*;
use gtk4::Box;
use log::{debug, error};

pub fn build_editor_tab(settings_path: &str) -> Box {
    use gtk4::{
        Adjustment, Align, Box as GtkBox, DropDown, Label, Orientation, Scale, SpinButton,
        Switch, StringList, PropertyExpression, StringObject, Expression,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-editor");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Editor Font (Dropdown with actual system fonts)
    let font_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let font_header = Label::new(Some("Font"));
    font_header.set_markup("<b>Font</b>");
    font_header.set_halign(Align::Start);
    font_header.set_xalign(0.0);

    let font_spacer = GtkBox::new(Orientation::Horizontal, 0);
    font_spacer.set_hexpand(true);

    // Load system fonts and current font setting
    let current_font = {
        let settings = AppSettings::load_from_file(settings_path).unwrap_or_default();
        let editor = settings.editor.unwrap_or_default();
        editor.font.unwrap_or_else(|| "Monospace".to_string())
    };
    debug!("Current font setting: {}", current_font);

    // Get monospace fonts (preferred for code editing) - using cached fonts
    let monospace_fonts =
        crate::logic::loaders::font_loader::FontLoader::get_cached_monospace_fonts();
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
    // Note: set_search_match_mode may require newer GTK4 version, using default for now

    // Set initial selection based on current font
    let initial_selection = monospace_names
        .iter()
        .position(|font_name| font_name == &current_font)
        .unwrap_or(0); // Default to first font if current not found
    font_combo.set_selected(initial_selection as u32);

    // Connect font selection change to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        let monospace_names_clone = monospace_names.clone();
        
        font_combo.connect_selected_notify(move |combo| {
            let selected_index = combo.selected() as usize;
            if let Some(selected_font) = monospace_names_clone.get(selected_index) {
                let selected_font = selected_font.clone();
                debug!("Font changed to: {}", selected_font);

                // Load current settings and update font family
                let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

                // Ensure editor settings exist
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }

                // Update font setting
                if let Some(ref mut editor) = settings.editor {
                    editor.font = Some(selected_font.clone());
                }

                // Save settings
                if let Err(e) = settings.save_to_file(&settings_path) {
                    error!("Failed to save font setting: {}", e);
                    return;
                }

                // Create complete editor settings and trigger runtime update
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

    font_combo.set_halign(Align::End);
    font_hbox.append(&font_header);
    font_hbox.append(&font_spacer);
    font_hbox.append(&font_combo);
    font_hbox.set_margin_bottom(4);
    container.append(&font_hbox);

    // Description text under header
    let font_description = Label::new(Some("Select the font used in the editor."));
    font_description.set_halign(Align::Start);
    font_description.set_xalign(0.0);
    font_description.set_wrap(true);
    font_description.add_css_class("dim-label");
    font_description.set_margin_bottom(12);
    container.append(&font_description);

    // Font Size (Slider/SpinButton) with actual settings
    let font_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let font_size_header = Label::new(Some("Font Size"));
    font_size_header.set_markup("<b>Font Size</b>");
    font_size_header.set_halign(Align::Start);
    font_size_header.set_xalign(0.0);

    let font_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    font_size_spacer.set_hexpand(true);

    // Load current font size from settings
    let current_font_size = {
        let settings = AppSettings::load_from_file(settings_path).unwrap_or_default();
        let editor = settings.editor.unwrap_or_default();
        editor.font_size.unwrap_or(14) as f64
    };

    let font_size_adj = Adjustment::new(current_font_size, 10.0, 24.0, 1.0, 0.0, 0.0);
    let font_size_spin = SpinButton::new(Some(&font_size_adj), 1.0, 0);
    font_size_spin.set_halign(Align::End);

    // Connect font size changes to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        font_size_adj.connect_value_changed(move |adj| {
            let new_size = adj.value() as u8;
            debug!("Font size changed to: {}px", new_size);

            // Load current settings and update font size
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(EditorSettings::default());
            }

            // Update font size setting
            if let Some(ref mut editor) = settings.editor {
                editor.font_size = Some(new_size);
            }

            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save font size setting: {}", e);
                return;
            }

            // Create complete editor settings and trigger runtime update
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

    font_size_hbox.append(&font_size_header);
    font_size_hbox.append(&font_size_spacer);
    font_size_hbox.append(&font_size_spin);
    font_size_hbox.set_margin_top(8);
    font_size_hbox.set_margin_bottom(4);
    container.append(&font_size_hbox);

    // Description text under header
    let font_size_description =
        Label::new(Some("Set the font size for the editor text (10-24 px)."));
    font_size_description.set_halign(Align::Start);
    font_size_description.set_xalign(0.0);
    font_size_description.set_wrap(true);
    font_size_description.add_css_class("dim-label");
    font_size_description.set_margin_bottom(12);
    container.append(&font_size_description);

    let font_size_slider = Scale::new(Orientation::Horizontal, Some(&font_size_adj));
    font_size_slider.set_draw_value(false);
    font_size_slider.set_hexpand(true);
    font_size_slider.set_round_digits(0); // Discrete steps
    font_size_slider.set_value_pos(gtk4::PositionType::Right);
    font_size_slider.set_increments(1.0, 1.0);
    for size in 10..=24 {
        font_size_slider.add_mark(
            size as f64,
            gtk4::PositionType::Bottom,
            Some(&size.to_string()),
        );
    }

    // Slider left-aligned under spinbutton
    font_size_slider.set_halign(Align::Start);
    font_size_slider.set_width_request(300);
    font_size_slider.set_margin_bottom(12);
    container.append(&font_size_slider);

    // Line Height (Slider/SpinButton)
    let line_height_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let line_height_header = Label::new(Some("Line Height"));
    line_height_header.set_markup("<b>Line Height</b>");
    line_height_header.set_halign(Align::Start);
    line_height_header.set_xalign(0.0);

    let line_height_spacer = GtkBox::new(Orientation::Horizontal, 0);
    line_height_spacer.set_hexpand(true);

    // Load current line height from settings
    let current_line_height = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .editor
        .and_then(|e| e.line_height)
        .unwrap_or(1.4) as f64;

    let line_height_adj = Adjustment::new(current_line_height, 1.0, 2.0, 0.05, 0.0, 0.0);
    let line_height_spin = SpinButton::new(Some(&line_height_adj), 0.05, 2);
    line_height_spin.set_halign(Align::End);

    // Connect line height changes to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        line_height_adj.connect_value_changed(move |adj| {
            let new_line_height = adj.value() as f32;
            debug!("Line height changed to: {}", new_line_height);

            // Load current settings and update line height
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(crate::logic::swanson::EditorSettings::default());
            }

            // Update line height setting
            if let Some(ref mut editor) = settings.editor {
                editor.line_height = Some(new_line_height);
            }

            // Save settings
            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save line height setting: {}", e);
                return;
            }

            // Create complete editor settings and trigger runtime update
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

    line_height_hbox.append(&line_height_header);
    line_height_hbox.append(&line_height_spacer);
    line_height_hbox.append(&line_height_spin);
    line_height_hbox.set_margin_top(8);
    line_height_hbox.set_margin_bottom(4);
    container.append(&line_height_hbox);

    // Description text under header
    let line_height_description = Label::new(Some("Adjust the vertical spacing between lines."));
    line_height_description.set_halign(Align::Start);
    line_height_description.set_xalign(0.0);
    line_height_description.set_wrap(true);
    line_height_description.add_css_class("dim-label");
    line_height_description.set_margin_bottom(12);
    container.append(&line_height_description);

    let line_height_slider = Scale::new(Orientation::Horizontal, Some(&line_height_adj));
    line_height_slider.set_draw_value(false);
    line_height_slider.set_hexpand(true);
    for mark in [1.0, 1.2, 1.4, 1.6, 1.8, 2.0].iter() {
        line_height_slider.add_mark(
            *mark,
            gtk4::PositionType::Bottom,
            Some(&format!("{:.1}", mark)),
        );
    }

    // Slider left-aligned under spinbutton
    line_height_slider.set_halign(Align::Start);
    line_height_slider.set_width_request(300);
    line_height_slider.set_margin_bottom(12);
    container.append(&line_height_slider);

    // Line Wrapping (Toggle)
    let line_wrap_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let line_wrap_header = Label::new(Some("Line Wrapping"));
    line_wrap_header.set_markup("<b>Line Wrapping</b>");
    line_wrap_header.set_halign(Align::Start);
    line_wrap_header.set_xalign(0.0);

    let line_wrap_spacer = GtkBox::new(Orientation::Horizontal, 0);
    line_wrap_spacer.set_hexpand(true);

    let line_wrap_switch = Switch::new();
    line_wrap_switch.set_halign(Align::End);

    // Load current line wrapping setting
    let current_line_wrapping = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .editor
        .and_then(|e| e.line_wrapping)
        .unwrap_or(true); // Default to true for better UX
    line_wrap_switch.set_active(current_line_wrapping);

    // Connect line wrapping changes to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        line_wrap_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Line wrapping changed to: {}", enabled);

            // Load current settings and update line wrapping
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(EditorSettings::default());
            }

            // Update line wrapping setting
            if let Some(ref mut editor) = settings.editor {
                editor.line_wrapping = Some(enabled);
            }

            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save line wrapping setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Create complete editor settings and trigger runtime update
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

    line_wrap_hbox.append(&line_wrap_header);
    line_wrap_hbox.append(&line_wrap_spacer);
    line_wrap_hbox.append(&line_wrap_switch);
    line_wrap_hbox.set_margin_top(8);
    line_wrap_hbox.set_margin_bottom(4);
    container.append(&line_wrap_hbox);

    // Description text under header
    let line_wrap_description =
        Label::new(Some("Wrap long lines to fit within the editor window."));
    line_wrap_description.set_halign(Align::Start);
    line_wrap_description.set_xalign(0.0);
    line_wrap_description.set_wrap(true);
    line_wrap_description.add_css_class("dim-label");
    line_wrap_description.set_margin_bottom(12);
    container.append(&line_wrap_description);

    // Auto Pairing (Toggle)
    let auto_pair_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let auto_pair_header = Label::new(Some("Auto Pairing"));
    auto_pair_header.set_markup("<b>Auto Pairing</b>");
    auto_pair_header.set_halign(Align::Start);
    auto_pair_header.set_xalign(0.0);

    let auto_pair_spacer = GtkBox::new(Orientation::Horizontal, 0);
    auto_pair_spacer.set_hexpand(true);

    let auto_pair_switch = Switch::new();
    auto_pair_switch.set_halign(Align::End);

    auto_pair_hbox.append(&auto_pair_header);
    auto_pair_hbox.append(&auto_pair_spacer);
    auto_pair_hbox.append(&auto_pair_switch);
    auto_pair_hbox.set_margin_top(8);
    auto_pair_hbox.set_margin_bottom(4);
    container.append(&auto_pair_hbox);

    // Description text under header
    let auto_pair_description = Label::new(Some(
        "Automatically insert closing characters for **, [], (), and backticks.",
    ));
    auto_pair_description.set_halign(Align::Start);
    auto_pair_description.set_xalign(0.0);
    auto_pair_description.set_wrap(true);
    auto_pair_description.add_css_class("dim-label");
    auto_pair_description.set_margin_bottom(12);
    container.append(&auto_pair_description);

    // Show Invisible Characters (Toggle)
    let show_invis_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let show_invis_header = Label::new(Some("Show Invisible Characters"));
    show_invis_header.set_markup("<b>Show Invisible Characters</b>");
    show_invis_header.set_halign(Align::Start);
    show_invis_header.set_xalign(0.0);

    let show_invis_spacer = GtkBox::new(Orientation::Horizontal, 0);
    show_invis_spacer.set_hexpand(true);

    let show_invis_switch = Switch::new();
    show_invis_switch.set_halign(Align::End);

    // Load current show invisible characters setting
    let current_show_invisibles = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .editor
        .and_then(|e| e.show_invisibles)
        .unwrap_or(false);
    show_invis_switch.set_active(current_show_invisibles);

    // Connect show invisibles changes to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        show_invis_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Show invisibles changed to: {}", enabled);

            // Load current settings and update show invisibles
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(EditorSettings::default());
            }

            // Update show invisibles setting
            if let Some(ref mut editor) = settings.editor {
                editor.show_invisibles = Some(enabled);
            }

            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save show invisibles setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Create complete editor settings and trigger runtime update
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

    show_invis_hbox.append(&show_invis_header);
    show_invis_hbox.append(&show_invis_spacer);
    show_invis_hbox.append(&show_invis_switch);
    show_invis_hbox.set_margin_top(8);
    show_invis_hbox.set_margin_bottom(4);
    container.append(&show_invis_hbox);

    // Description text under header
    let show_invis_description = Label::new(Some(
        "Display tabs, spaces, and newlines visually in the editor.",
    ));
    show_invis_description.set_halign(Align::Start);
    show_invis_description.set_xalign(0.0);
    show_invis_description.set_wrap(true);
    show_invis_description.add_css_class("dim-label");
    show_invis_description.set_margin_bottom(12);
    container.append(&show_invis_description);

    // Convert Tabs to Spaces (Toggle)
    let tabs_to_spaces_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let tabs_to_spaces_header = Label::new(Some("Convert Tabs to Spaces"));
    tabs_to_spaces_header.set_markup("<b>Convert Tabs to Spaces</b>");
    tabs_to_spaces_header.set_halign(Align::Start);
    tabs_to_spaces_header.set_xalign(0.0);

    let tabs_to_spaces_spacer = GtkBox::new(Orientation::Horizontal, 0);
    tabs_to_spaces_spacer.set_hexpand(true);

    let tabs_to_spaces_switch = Switch::new();
    tabs_to_spaces_switch.set_halign(Align::End);

    // Load current tabs to spaces setting
    let current_tabs_to_spaces = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .editor
        .and_then(|e| e.tabs_to_spaces)
        .unwrap_or(false);
    tabs_to_spaces_switch.set_active(current_tabs_to_spaces);

    // Connect tabs to spaces changes to save settings and trigger runtime updates
    {
        let settings_path = settings_path.to_string();
        tabs_to_spaces_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Tabs to spaces changed to: {}", enabled);

            // Load current settings and update tabs to spaces
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(EditorSettings::default());
            }

            // Update tabs to spaces setting
            if let Some(ref mut editor) = settings.editor {
                editor.tabs_to_spaces = Some(enabled);
            }

            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save tabs to spaces setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Create complete editor settings and trigger runtime update
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

    tabs_to_spaces_hbox.append(&tabs_to_spaces_header);
    tabs_to_spaces_hbox.append(&tabs_to_spaces_spacer);
    tabs_to_spaces_hbox.append(&tabs_to_spaces_switch);
    tabs_to_spaces_hbox.set_margin_top(8);
    tabs_to_spaces_hbox.set_margin_bottom(4);
    container.append(&tabs_to_spaces_hbox);

    // Description text under header
    let tabs_to_spaces_description = Label::new(Some("Replace tab characters with spaces."));
    tabs_to_spaces_description.set_halign(Align::Start);
    tabs_to_spaces_description.set_xalign(0.0);
    tabs_to_spaces_description.set_wrap(true);
    tabs_to_spaces_description.add_css_class("dim-label");
    tabs_to_spaces_description.set_margin_bottom(12);
    container.append(&tabs_to_spaces_description);

    // Syntax Colors (Toggle)
    let syntax_colors_switch = Switch::new();

    // Load current setting
    let current_syntax_colors = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .editor
        .and_then(|e| e.syntax_colors)
        .unwrap_or(true);
    syntax_colors_switch.set_active(current_syntax_colors);

    // Wire to save setting when toggled
    {
        let settings_path = settings_path.to_string();
        syntax_colors_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Syntax colors changed to: {}", enabled);

            // Load current settings and update syntax colors
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure editor settings exist
            if settings.editor.is_none() {
                settings.editor = Some(crate::logic::swanson::EditorSettings::default());
            }

            // Update syntax colors setting
            if let Some(ref mut editor) = settings.editor {
                editor.syntax_colors = Some(enabled);
            }

            // Save settings
            if let Err(e) = settings.save_to_file(&settings_path) {
                error!("Failed to save syntax colors setting: {}", e);
                return glib::Propagation::Proceed;
            }

            // Create complete editor settings and trigger runtime update
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

    // Syntax Colors (Toggle)
    let syntax_colors_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let syntax_colors_header = Label::new(Some("Syntax Colors"));
    syntax_colors_header.set_markup("<b>Syntax Colors</b>");
    syntax_colors_header.set_halign(Align::Start);
    syntax_colors_header.set_xalign(0.0);

    let syntax_colors_spacer = GtkBox::new(Orientation::Horizontal, 0);
    syntax_colors_spacer.set_hexpand(true);

    syntax_colors_switch.set_halign(Align::End);

    syntax_colors_hbox.append(&syntax_colors_header);
    syntax_colors_hbox.append(&syntax_colors_spacer);
    syntax_colors_hbox.append(&syntax_colors_switch);
    syntax_colors_hbox.set_margin_top(8);
    syntax_colors_hbox.set_margin_bottom(4);
    container.append(&syntax_colors_hbox);

    // Description text under header
    let syntax_colors_description = Label::new(Some(
        "Enable or disable syntax-based color highlighting for Markdown.",
    ));
    syntax_colors_description.set_halign(Align::Start);
    syntax_colors_description.set_xalign(0.0);
    syntax_colors_description.set_wrap(true);
    syntax_colors_description.add_css_class("dim-label");
    syntax_colors_description.set_margin_bottom(12);
    container.append(&syntax_colors_description);

    // Enable Markdown Linting (Toggle)
    let linting_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let linting_header = Label::new(Some("Enable Markdown Linting"));
    linting_header.set_markup("<b>Enable Markdown Linting</b>");
    linting_header.set_halign(Align::Start);
    linting_header.set_xalign(0.0);

    let linting_spacer = GtkBox::new(Orientation::Horizontal, 0);
    linting_spacer.set_hexpand(true);

    let linting_switch = Switch::new();
    linting_switch.set_halign(Align::End);

    linting_hbox.append(&linting_header);
    linting_hbox.append(&linting_spacer);
    linting_hbox.append(&linting_switch);
    linting_hbox.set_margin_top(8);
    linting_hbox.set_margin_bottom(4);
    container.append(&linting_hbox);

    // Description text under header
    let linting_description =
        Label::new(Some("Check for Markdown syntax issues and style problems."));
    linting_description.set_halign(Align::Start);
    linting_description.set_xalign(0.0);
    linting_description.set_wrap(true);
    linting_description.add_css_class("dim-label");
    linting_description.set_margin_bottom(12);
    container.append(&linting_description);

    container
}
