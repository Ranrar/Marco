use core::logic::swanson::EditorSettings;
use gtk4::prelude::*;
use gtk4::Box;
use log::{debug, error};
use std::rc::Rc;

use super::helpers::{add_setting_row_i18n, create_section_header_i18n, SettingsI18nRegistry};
use crate::components::language::{SettingsIntelligenceTranslations, Translations};

pub fn build_intelligence_tab(
    settings_path: &str,
    translations: &SettingsIntelligenceTranslations,
    i18n: &SettingsI18nRegistry,
) -> Box {
    use gtk4::{Box as GtkBox, Orientation, Switch};

    let settings_manager_opt = match core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path),
    ) {
        Ok(settings_manager) => Some(std::sync::Arc::new(settings_manager)),
        Err(e) => {
            debug!(
                "Failed to initialize SettingsManager for intelligence settings: {}",
                e
            );
            None
        }
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    let intelligence_header = create_section_header_i18n(
        i18n,
        &translations.section_intelligence,
        Rc::new(|t: &Translations| t.settings.intelligence.section_intelligence.clone()),
    );
    intelligence_header.set_margin_start(10);
    intelligence_header.set_margin_end(10);
    container.append(&intelligence_header);

    let intro = gtk4::Label::new(Some(&translations.intro_description));
    intro.set_xalign(0.0);
    intro.set_wrap(true);
    intro.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
    intro.add_css_class("marco-settings-description");
    intro.set_margin_top(4);
    intro.set_margin_start(10);
    intro.set_margin_end(10);
    i18n.bind_label_text(
        &intro,
        Rc::new(|t: &Translations| t.settings.intelligence.intro_description.clone()),
    );
    container.append(&intro);

    let issues_header = create_section_header_i18n(
        i18n,
        &translations.section_issues,
        Rc::new(|t: &Translations| t.settings.intelligence.section_issues.clone()),
    );
    issues_header.set_margin_start(10);
    issues_header.set_margin_end(10);
    container.append(&issues_header);

    let diagnostics_underlines_switch = Switch::new();
    diagnostics_underlines_switch.add_css_class("marco-switch");
    let current_diagnostics_underlines = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings
            .editor
            .and_then(|e| e.diagnostics_underlines_enabled)
            .unwrap_or(true)
    } else {
        true
    };
    diagnostics_underlines_switch.set_active(current_diagnostics_underlines);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        diagnostics_underlines_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Diagnostics underlines changed to: {}", enabled);

            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = settings.editor {
                    editor.diagnostics_underlines_enabled = Some(enabled);
                }
            }) {
                error!("Failed to save diagnostics underlines setting: {}", e);
                return glib::Propagation::Proceed;
            }

            crate::components::editor::editor_manager::trigger_intelligence_refresh();

            glib::Propagation::Proceed
        });
    }

    let diagnostics_underlines_row = add_setting_row_i18n(
        i18n,
        &translations.diagnostics_underlines_label,
        &translations.diagnostics_underlines_description,
        Rc::new(|t: &Translations| t.settings.intelligence.diagnostics_underlines_label.clone()),
        Rc::new(|t: &Translations| {
            t.settings
                .intelligence
                .diagnostics_underlines_description
                .clone()
        }),
        &diagnostics_underlines_switch,
        false,
    );
    container.append(&diagnostics_underlines_row);

    let insights_header = create_section_header_i18n(
        i18n,
        &translations.section_insights,
        Rc::new(|t: &Translations| t.settings.intelligence.section_insights.clone()),
    );
    insights_header.set_margin_start(10);
    insights_header.set_margin_end(10);
    container.append(&insights_header);

    let markdown_insights_switch = Switch::new();
    markdown_insights_switch.add_css_class("marco-switch");
    let current_markdown_hover = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings
            .editor
            .and_then(|e| e.markdown_hover_enabled)
            .unwrap_or(true)
    } else {
        true
    };
    markdown_insights_switch.set_active(current_markdown_hover);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        markdown_insights_switch.connect_state_set(move |_switch, state| {
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }
                if let Some(editor) = settings.editor.as_mut() {
                    editor.markdown_hover_enabled = Some(state);
                }
            }) {
                error!("Failed to save markdown insights setting: {}", e);
            }

            crate::components::editor::editor_manager::trigger_intelligence_refresh();

            glib::Propagation::Proceed
        });
    }

    container.append(&add_setting_row_i18n(
        i18n,
        &translations.markdown_insights_label,
        &translations.markdown_insights_description,
        Rc::new(|t: &Translations| t.settings.intelligence.markdown_insights_label.clone()),
        Rc::new(|t: &Translations| {
            t.settings
                .intelligence
                .markdown_insights_description
                .clone()
        }),
        &markdown_insights_switch,
        false,
    ));

    let diagnostics_insights_switch = Switch::new();
    diagnostics_insights_switch.add_css_class("marco-switch");
    let current_diagnostics_hover = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings
            .editor
            .and_then(|e| e.diagnostics_hover_enabled)
            .unwrap_or(true)
    } else {
        true
    };
    diagnostics_insights_switch.set_active(current_diagnostics_hover);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        diagnostics_insights_switch.connect_state_set(move |_switch, state| {
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.editor.is_none() {
                    settings.editor = Some(EditorSettings::default());
                }
                if let Some(editor) = settings.editor.as_mut() {
                    editor.diagnostics_hover_enabled = Some(state);
                }
            }) {
                error!("Failed to save diagnostics insights setting: {}", e);
            }

            crate::components::editor::editor_manager::trigger_intelligence_refresh();

            glib::Propagation::Proceed
        });
    }

    container.append(&add_setting_row_i18n(
        i18n,
        &translations.issue_insights_label,
        &translations.issue_insights_description,
        Rc::new(|t: &Translations| t.settings.intelligence.issue_insights_label.clone()),
        Rc::new(|t: &Translations| t.settings.intelligence.issue_insights_description.clone()),
        &diagnostics_insights_switch,
        false,
    ));

    let highlighting_header = create_section_header_i18n(
        i18n,
        &translations.section_highlighting,
        Rc::new(|t: &Translations| t.settings.intelligence.section_highlighting.clone()),
    );
    highlighting_header.set_margin_start(10);
    highlighting_header.set_margin_end(10);
    container.append(&highlighting_header);

    let syntax_colors_switch = Switch::new();
    syntax_colors_switch.add_css_class("marco-switch");

    let current_syntax_colors = if let Some(ref settings_manager) = settings_manager_opt {
        let settings = settings_manager.get_settings();
        settings
            .editor
            .and_then(|e| e.syntax_colors)
            .unwrap_or(true)
    } else {
        true
    };
    syntax_colors_switch.set_active(current_syntax_colors);

    if let Some(settings_manager_clone) = settings_manager_opt.clone() {
        syntax_colors_switch.connect_state_set(move |_switch, state| {
            let enabled = state;
            debug!("Syntax colors changed to: {}", enabled);

            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.editor.is_none() {
                    settings.editor = Some(core::logic::swanson::EditorSettings::default());
                }
                if let Some(ref mut editor) = settings.editor {
                    editor.syntax_colors = Some(enabled);
                }
            }) {
                error!("Failed to save syntax colors setting: {}", e);
                return glib::Propagation::Proceed;
            }

            let settings = settings_manager_clone.get_settings();
            let editor = settings.editor.unwrap_or_default();
            let editor_settings =
                crate::components::editor::display_config::EditorDisplaySettings {
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
            }

            crate::components::editor::editor_manager::trigger_intelligence_refresh();

            glib::Propagation::Proceed
        });
    }

    container.append(&add_setting_row_i18n(
        i18n,
        &translations.syntax_highlighting_label,
        &translations.syntax_highlighting_description,
        Rc::new(|t: &Translations| t.settings.intelligence.syntax_highlighting_label.clone()),
        Rc::new(|t: &Translations| {
            t.settings
                .intelligence
                .syntax_highlighting_description
                .clone()
        }),
        &syntax_colors_switch,
        false,
    ));

    container
}
