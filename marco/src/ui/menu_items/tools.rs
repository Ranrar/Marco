use crate::components::language::Translations;
use crate::components::viewer::preview_types::ViewMode;
use core::logic::swanson::{EditorSettings, LayoutSettings, SettingsManager};
use gtk4::gio;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn populate_tools_menu(
    tools_menu: &gio::Menu,
    translations: &Translations,
    state: &ToolsMenuState,
) {
    tools_menu.remove_all();

    let preview = gio::Menu::new();
    let preview_label = if state.show_raw_html {
        &translations.menu.show_raw_html
    } else {
        &translations.menu.show_rendered_markdown
    };
    preview.append(Some(preview_label), Some("app.tools_toggle_render_mode"));
    tools_menu.append_section(None, &preview);

    let editor_toggles = gio::Menu::new();
    let _ = state;
    editor_toggles.append(
        Some(&translations.menu.text_wrap),
        Some("app.tools_toggle_text_wrap"),
    );
    editor_toggles.append(
        Some(&translations.menu.line_numbers),
        Some("app.tools_toggle_line_numbers"),
    );
    editor_toggles.append(
        Some(&translations.settings.editor.auto_pairing_label),
        Some("app.tools_toggle_auto_pairing"),
    );
    editor_toggles.append(
        Some(&translations.settings.editor.tabs_to_spaces_label),
        Some("app.tools_toggle_tabs_to_spaces"),
    );
    editor_toggles.append(
        Some(&translations.settings.editor.syntax_colors_label),
        Some("app.tools_toggle_syntax_colors"),
    );
    editor_toggles.append(
        Some(&translations.settings.editor.linting_label),
        Some("app.tools_toggle_markdown_linting"),
    );
    tools_menu.append_section(None, &editor_toggles);

    // Settings-aligned layout function.
    let layout = gio::Menu::new();
    layout.append(
        Some(&translations.menu.sync_scrolling),
        Some("app.tools_toggle_sync_scrolling"),
    );
    tools_menu.append_section(None, &layout);

    let app_items = gio::Menu::new();
    app_items.append(Some(&translations.menu.settings), Some("app.settings"));
    tools_menu.append_section(None, &app_items);
}

pub fn setup_tools_actions(
    app: &gtk4::Application,
    tools_menu: &gio::Menu,
    translations_rc: Rc<RefCell<Translations>>,
    settings_manager: Arc<SettingsManager>,
    editor_view: &sourceview5::View,
    set_view_mode: Rc<Box<dyn Fn(ViewMode)>>,
) {
    // Ensure initial label reflects current mode.
    let initial = current_tools_state(&settings_manager, editor_view);

    ensure_bool_toggle_action(
        app,
        "tools_toggle_auto_pairing",
        initial.auto_pairing_enabled,
        false,
        || {},
    );
    ensure_bool_toggle_action(
        app,
        "tools_toggle_markdown_linting",
        initial.markdown_linting_enabled,
        false,
        || {},
    );

    sync_tools_toggle_action_states(app, &initial);
    populate_tools_menu(tools_menu, &translations_rc.borrow(), &initial);

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let set_view_mode = set_view_mode.clone();
        super::add_format_action(&app, "tools_toggle_render_mode", move || {
            let currently_raw = is_currently_raw_mode(&settings_manager);
            if currently_raw {
                (set_view_mode)(ViewMode::HtmlPreview);
            } else {
                (set_view_mode)(ViewMode::CodePreview);
            }

            let next_mode = if currently_raw {
                "HTML Preview"
            } else {
                "Source Code"
            };

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut l) = s.layout {
                    l.view_mode = Some(next_mode.to_string());
                }
            }) {
                log::warn!("Failed to persist tools render/raw mode: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_text_wrap",
            None,
            &gtk4::glib::Variant::from(initial.wrap_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let wrap_enabled = editor_view.wrap_mode() != gtk4::WrapMode::None;
            let next = !wrap_enabled;
            editor_view.set_wrap_mode(if next {
                gtk4::WrapMode::WordChar
            } else {
                gtk4::WrapMode::None
            });

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.editor.is_none() {
                    s.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = s.editor {
                    editor.line_wrapping = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools text wrapping: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_line_numbers",
            None,
            &gtk4::glib::Variant::from(initial.line_numbers_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.show_line_numbers)
                .unwrap_or(true);
            let next = !current;

            if let Err(e) =
                crate::components::editor::editor_manager::update_line_numbers_globally(next)
            {
                log::warn!("Failed to apply tools line numbers toggle: {}", e);
            }

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut layout) = s.layout {
                    layout.show_line_numbers = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools line numbers toggle: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_sync_scrolling",
            None,
            &gtk4::glib::Variant::from(initial.sync_scrolling_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.sync_scrolling)
                .unwrap_or(true);
            let next = !current;

            if let Err(e) =
                crate::components::editor::editor_manager::set_scroll_sync_enabled_globally(next)
            {
                log::warn!("Failed to apply tools sync scrolling toggle: {}", e);
            }

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut layout) = s.layout {
                    layout.sync_scrolling = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools sync scrolling toggle: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_tabs_to_spaces",
            None,
            &gtk4::glib::Variant::from(initial.tabs_to_spaces_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .editor
                .as_ref()
                .and_then(|e| e.tabs_to_spaces)
                .unwrap_or(true);
            let next = !current;

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.editor.is_none() {
                    s.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = s.editor {
                    editor.tabs_to_spaces = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools tabs-to-spaces toggle: {}", e);
            }

            if let Err(e) = apply_editor_display_settings_from_settings(&settings_manager) {
                log::warn!("Failed to apply tabs-to-spaces editor settings: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_syntax_colors",
            None,
            &gtk4::glib::Variant::from(initial.syntax_colors_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .editor
                .as_ref()
                .and_then(|e| e.syntax_colors)
                .unwrap_or(true);
            let next = !current;

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.editor.is_none() {
                    s.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = s.editor {
                    editor.syntax_colors = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools syntax colors toggle: {}", e);
            }

            if let Err(e) = apply_editor_display_settings_from_settings(&settings_manager) {
                log::warn!("Failed to apply syntax colors editor settings: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_text_direction",
            None,
            &gtk4::glib::Variant::from(initial.rtl_text_direction_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current_rtl = settings_manager
                .get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.text_direction.as_deref())
                .map(|dir| dir.eq_ignore_ascii_case("rtl"))
                .unwrap_or(false);
            let next_rtl = !current_rtl;

            editor_view.set_direction(if next_rtl {
                gtk4::TextDirection::Rtl
            } else {
                gtk4::TextDirection::Ltr
            });

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.layout.is_none() {
                    s.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut layout) = s.layout {
                    layout.text_direction = Some(if next_rtl {
                        "rtl".to_string()
                    } else {
                        "ltr".to_string()
                    });
                }
            }) {
                log::warn!("Failed to persist tools text direction toggle: {}", e);
            }

            refresh_tools_menu(
                &app_for_closure,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    }

    // Auto pairing + markdown linting are currently not fully wired in runtime behavior.
    // Keep menu text visible with state indicator, but disable interaction for now.
    set_bool_toggle_action_state(
        app,
        "tools_toggle_auto_pairing",
        initial.auto_pairing_enabled,
        false,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_markdown_linting",
        initial.markdown_linting_enabled,
        false,
    );
}

#[derive(Clone, Copy)]
pub struct ToolsMenuState {
    pub show_raw_html: bool,
    pub wrap_enabled: bool,
    pub line_numbers_enabled: bool,
    pub sync_scrolling_enabled: bool,
    pub auto_pairing_enabled: bool,
    pub tabs_to_spaces_enabled: bool,
    pub syntax_colors_enabled: bool,
    pub markdown_linting_enabled: bool,
    pub rtl_text_direction_enabled: bool,
}

fn current_tools_state(
    settings_manager: &Arc<SettingsManager>,
    editor_view: &sourceview5::View,
) -> ToolsMenuState {
    let settings = settings_manager.get_settings();
    let line_numbers_enabled = settings
        .layout
        .as_ref()
        .and_then(|l| l.show_line_numbers)
        .unwrap_or(true);
    let sync_scrolling_enabled = settings
        .layout
        .as_ref()
        .and_then(|l| l.sync_scrolling)
        .unwrap_or(true);
    let auto_pairing_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.auto_pairing)
        .unwrap_or(true);
    let tabs_to_spaces_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.tabs_to_spaces)
        .unwrap_or(true);
    let syntax_colors_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.syntax_colors)
        .unwrap_or(true);
    let markdown_linting_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.linting)
        .unwrap_or(true);
    let rtl_text_direction_enabled = settings
        .layout
        .as_ref()
        .and_then(|l| l.text_direction.as_deref())
        .map(|dir| dir.eq_ignore_ascii_case("rtl"))
        .unwrap_or(false);
    let show_raw_html = !is_currently_raw_mode(settings_manager);
    let wrap_enabled = editor_view.wrap_mode() != gtk4::WrapMode::None;

    ToolsMenuState {
        show_raw_html,
        wrap_enabled,
        line_numbers_enabled,
        sync_scrolling_enabled,
        auto_pairing_enabled,
        tabs_to_spaces_enabled,
        syntax_colors_enabled,
        markdown_linting_enabled,
        rtl_text_direction_enabled,
    }
}

fn refresh_tools_menu(
    app: &gtk4::Application,
    tools_menu: &gio::Menu,
    translations_rc: &Rc<RefCell<Translations>>,
    settings_manager: &Arc<SettingsManager>,
    editor_view: &sourceview5::View,
) {
    let state = current_tools_state(settings_manager, editor_view);
    sync_tools_toggle_action_states(app, &state);
    populate_tools_menu(tools_menu, &translations_rc.borrow(), &state);
}

fn sync_tools_toggle_action_states(app: &gtk4::Application, state: &ToolsMenuState) {
    set_bool_toggle_action_state(app, "tools_toggle_text_wrap", state.wrap_enabled, true);
    set_bool_toggle_action_state(
        app,
        "tools_toggle_line_numbers",
        state.line_numbers_enabled,
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_auto_pairing",
        state.auto_pairing_enabled,
        false,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_tabs_to_spaces",
        state.tabs_to_spaces_enabled,
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_syntax_colors",
        state.syntax_colors_enabled,
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_markdown_linting",
        state.markdown_linting_enabled,
        false,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_sync_scrolling",
        state.sync_scrolling_enabled,
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_text_direction",
        state.rtl_text_direction_enabled,
        false,
    );
}

fn ensure_bool_toggle_action<F>(
    app: &gtk4::Application,
    name: &str,
    initial: bool,
    enabled: bool,
    activate: F,
) where
    F: Fn() + 'static,
{
    if app.lookup_action(name).is_none() {
        let action =
            gio::SimpleAction::new_stateful(name, None, &gtk4::glib::Variant::from(initial));
        action.connect_activate(move |_, _| activate());
        app.add_action(&action);
    }

    set_bool_toggle_action_state(app, name, initial, enabled);
}

fn set_bool_toggle_action_state(app: &gtk4::Application, name: &str, state: bool, enabled: bool) {
    if let Some(action) = app
        .lookup_action(name)
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        action.set_state(&gtk4::glib::Variant::from(state));
        action.set_enabled(enabled);
    }
}

fn apply_editor_display_settings_from_settings(
    settings_manager: &Arc<SettingsManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = settings_manager.get_settings();
    let editor = settings.editor.unwrap_or_default();
    let editor_display_settings =
        crate::components::editor::display_config::EditorDisplaySettings {
            font_family: editor.font.unwrap_or_else(|| "Monospace".to_string()),
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

    crate::components::editor::editor_manager::update_editor_settings_globally(
        &editor_display_settings,
    )
}

fn is_currently_raw_mode(settings_manager: &Arc<SettingsManager>) -> bool {
    settings_manager
        .get_settings()
        .layout
        .as_ref()
        .and_then(|l| l.view_mode.as_ref())
        .map(|v| matches!(v.as_str(), "Source Code" | "Code Preview"))
        .unwrap_or(false)
}
