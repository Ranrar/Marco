use crate::components::language::Translations;
use crate::components::viewer::preview_types::ViewMode;
use gtk4::gio;
use gtk4::prelude::*;
use marco_shared::logic::swanson::{EditorSettings, LayoutSettings, SettingsManager};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn populate_tools_menu(
    tools_menu: &gio::Menu,
    translations: &Translations,
    state: &ToolsMenuState,
) {
    tools_menu.remove_all();

    // ── View-mode section (radio-style, three explicit items) ──────────────
    let view_section = gio::Menu::new();
    view_section.append(Some("Live preview"), Some("app.tools_view_live"));
    view_section.append(Some("Print preview"), Some("app.tools_view_print"));
    view_section.append(Some("Code"), Some("app.tools_view_code"));
    tools_menu.append_section(None, &view_section);

    let editor_toggles = gio::Menu::new();
    let _ = state;
    editor_toggles.append(
        Some(&translations.settings.editor.line_wrapping_label),
        Some("app.tools_toggle_text_wrap"),
    );
    editor_toggles.append(
        Some(&translations.settings.layout.line_numbers_label),
        Some("app.tools_toggle_line_numbers"),
    );
    editor_toggles.append(
        Some(&translations.settings.editor.show_invisibles_label),
        Some("app.tools_toggle_show_invisibles"),
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
        Some(&translations.settings.editor.table_auto_align_label),
        Some("app.tools_toggle_table_auto_align"),
    );
    tools_menu.append_section(None, &editor_toggles);

    // Settings-aligned layout function.
    let layout = gio::Menu::new();
    layout.append(
        Some(&translations.settings.layout.sync_scrolling_label),
        Some("app.tools_toggle_sync_scrolling"),
    );
    {
        let dir_item = gio::MenuItem::new(
            Some(&translations.settings.layout.text_direction_label),
            Some("app.tools_toggle_text_direction"),
        );
        let badge = if state.rtl_text_direction_enabled {
            "RTL"
        } else {
            "LTR"
        };
        dir_item.set_attribute_value("badge", Some(&gtk4::glib::Variant::from(badge)));
        layout.append_item(&dir_item);
    }
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
    apply_rtl_fn: Rc<dyn Fn(bool)>,
) {
    // Ensure initial label reflects current mode.
    let initial = current_tools_state(&settings_manager, editor_view);

    sync_tools_toggle_action_states(app, &initial);
    populate_tools_menu(tools_menu, &translations_rc.borrow(), &initial);

    // ── View-mode actions (radio group: live, print, code) ─────────────────
    // Helper macro to reduce repetition for the three actions.
    // Each action: (a) switches ViewMode / page_view state, (b) persists, (c) refreshes menu.
    let register_view_action = |action_name: &'static str,
                                html_view_mode: &'static str,
                                page_view: bool| {
        let tools_menu = tools_menu.clone();
        let app_inner = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let set_view_mode = set_view_mode.clone();
        let action =
            gio::SimpleAction::new_stateful(action_name, None, &gtk4::glib::Variant::from(false));
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            // Apply the view mode switch immediately.
            if html_view_mode == "Source Code" {
                (set_view_mode)(ViewMode::CodePreview);
            } else {
                (set_view_mode)(ViewMode::HtmlPreview);
            }

            // Toggle page-view on/off via the global helper (triggers re-render).
            crate::components::editor::editor_manager::set_page_view_enabled(page_view);

            // Persist.
            if let Err(e) = settings_manager.update_settings(|s| {
                let layout = s.layout.get_or_insert_with(LayoutSettings::default);
                layout.view_mode = Some(html_view_mode.to_string());
                layout.page_view_enabled = Some(page_view);
            }) {
                log::warn!("Failed to persist view mode '{}': {}", action_name, e);
            }

            refresh_tools_menu(
                &app_inner,
                &tools_menu,
                &translations_rc,
                &settings_manager,
                &editor_view,
            );
        });
    };

    register_view_action("tools_view_live", "HTML Preview", false);
    register_view_action("tools_view_print", "HTML Preview", true);
    register_view_action("tools_view_code", "Source Code", false);

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
        let apply_rtl_fn = apply_rtl_fn.clone();
        action.connect_activate(move |_, _| {
            let current_rtl = settings_manager
                .get_settings()
                .layout
                .as_ref()
                .and_then(|l| l.text_direction.as_deref())
                .map(|dir| dir.eq_ignore_ascii_case("rtl"))
                .unwrap_or(false);
            let next_rtl = !current_rtl;

            // Apply direction to the entire application (window + all widgets).
            (apply_rtl_fn)(next_rtl);

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

    {
        let tools_menu = tools_menu.clone();
        let app = app.clone();
        let app_for_closure = app.clone();
        let translations_rc = translations_rc.clone();
        let settings_manager = settings_manager.clone();
        let editor_view = editor_view.clone();
        let action = gio::SimpleAction::new_stateful(
            "tools_toggle_show_invisibles",
            None,
            &gtk4::glib::Variant::from(initial.show_invisibles_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .editor
                .as_ref()
                .and_then(|e| e.show_invisibles)
                .unwrap_or(false);
            let next = !current;

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.editor.is_none() {
                    s.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = s.editor {
                    editor.show_invisibles = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools show-invisibles toggle: {}", e);
            }

            if let Err(e) = apply_editor_display_settings_from_settings(&settings_manager) {
                log::warn!("Failed to apply show-invisibles editor settings: {}", e);
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
            "tools_toggle_table_auto_align",
            None,
            &gtk4::glib::Variant::from(initial.table_auto_align_enabled),
        );
        app.add_action(&action);
        action.connect_activate(move |_, _| {
            let current = settings_manager
                .get_settings()
                .editor
                .as_ref()
                .and_then(|e| e.table_auto_align)
                .unwrap_or(true);
            let next = !current;

            crate::logic::tables::set_table_auto_align(next);

            if let Err(e) = settings_manager.update_settings(|s| {
                if s.editor.is_none() {
                    s.editor = Some(EditorSettings::default());
                }
                if let Some(ref mut editor) = s.editor {
                    editor.table_auto_align = Some(next);
                }
            }) {
                log::warn!("Failed to persist tools table-auto-align toggle: {}", e);
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
}

#[derive(Clone, Copy)]
pub struct ToolsMenuState {
    /// Which view mode is currently active: "live", "print", or "code".
    pub current_view_mode: &'static str,
    pub wrap_enabled: bool,
    pub line_numbers_enabled: bool,
    pub sync_scrolling_enabled: bool,
    pub tabs_to_spaces_enabled: bool,
    pub syntax_colors_enabled: bool,
    pub rtl_text_direction_enabled: bool,
    pub show_invisibles_enabled: bool,
    pub table_auto_align_enabled: bool,
}

fn current_tools_state(
    settings_manager: &Arc<SettingsManager>,
    _editor_view: &sourceview5::View,
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
    let rtl_text_direction_enabled = settings
        .layout
        .as_ref()
        .and_then(|l| l.text_direction.as_deref())
        .map(|dir| dir.eq_ignore_ascii_case("rtl"))
        .unwrap_or(false);
    let current_view_mode = current_view_mode_key(settings_manager);
    let wrap_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.line_wrapping)
        .unwrap_or(false);
    let show_invisibles_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.show_invisibles)
        .unwrap_or(false);
    let table_auto_align_enabled = settings
        .editor
        .as_ref()
        .and_then(|e| e.table_auto_align)
        .unwrap_or(true);

    ToolsMenuState {
        current_view_mode,
        wrap_enabled,
        line_numbers_enabled,
        sync_scrolling_enabled,
        tabs_to_spaces_enabled,
        syntax_colors_enabled,
        rtl_text_direction_enabled,
        show_invisibles_enabled,
        table_auto_align_enabled,
    }
}

pub fn refresh_tools_menu(
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
    set_bool_toggle_action_state(
        app,
        "tools_view_live",
        state.current_view_mode == "live",
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_view_print",
        state.current_view_mode == "print",
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_view_code",
        state.current_view_mode == "code",
        true,
    );
    set_bool_toggle_action_state(app, "tools_toggle_text_wrap", state.wrap_enabled, true);
    set_bool_toggle_action_state(
        app,
        "tools_toggle_line_numbers",
        state.line_numbers_enabled,
        true,
    );
    set_bool_toggle_action_state(
        app,
        "tools_toggle_show_invisibles",
        state.show_invisibles_enabled,
        true,
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
        "tools_toggle_table_auto_align",
        state.table_auto_align_enabled,
        true,
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
        true,
    );
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

/// Return "live", "print", or "code" for the current saved view mode.
fn current_view_mode_key(settings_manager: &Arc<SettingsManager>) -> &'static str {
    let settings = settings_manager.get_settings();
    let is_page_view = settings
        .layout
        .as_ref()
        .and_then(|l| l.page_view_enabled)
        .unwrap_or(false);
    let is_code = settings
        .layout
        .as_ref()
        .and_then(|l| l.view_mode.as_ref())
        .map(|v| matches!(v.as_str(), "Source Code" | "Code Preview"))
        .unwrap_or(false);
    if is_code {
        "code"
    } else if is_page_view {
        "print"
    } else {
        "live"
    }
}
