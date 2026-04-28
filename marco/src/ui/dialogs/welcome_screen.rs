//! Welcome Screen for First-Run Experience
//!
//! Shows a non-blocking welcome assistant with:
//! - Marco introduction and key features
//! - Language selection (saved to settings)
//! - Telemetry placeholder (disabled, not implemented yet)
//!
//! This window is non-blocking and shows while the main app continues running.

use gtk4::prelude::*;
use gtk4::{
    Align, Assistant, AssistantPageType, Box as GtkBox, Button, CenterBox, CheckButton, DropDown,
    Expression, Label, Orientation, PolicyType, PropertyExpression, ScrolledWindow, StringList,
    StringObject, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use crate::components::language::{LocalizationProvider, SimpleLocalizationManager, Translations};

fn effective_locale_code(selected_code: Option<&str>) -> String {
    selected_code
        .map(|s| s.to_string())
        .or_else(marco_shared::paths::detect_system_locale_iso639_1)
        .unwrap_or_else(|| "en".to_string())
}

fn escape_markup(s: &str) -> String {
    gtk4::glib::markup_escape_text(s).as_str().to_string()
}

fn find_first_stack_sidebar(root: &gtk4::Widget) -> Option<gtk4::StackSidebar> {
    if let Ok(sidebar) = root.clone().downcast::<gtk4::StackSidebar>() {
        return Some(sidebar);
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_first_stack_sidebar(&widget) {
            return Some(found);
        }
        child = widget.next_sibling();
    }

    None
}

fn tag_native_assistant_sidebars(root: &gtk4::Widget) {
    if root.has_css_class("sidebar") {
        root.add_css_class("marco-settings-sidebar");
        root.add_css_class("marco-welcome-sidebar");
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        tag_native_assistant_sidebars(&widget);
        child = widget.next_sibling();
    }
}

fn apply_welcome_sidebar_classes(assistant: &gtk4::Assistant) {
    tag_native_assistant_sidebars(assistant.upcast_ref::<gtk4::Widget>());

    if let Some(stack_sidebar) = find_first_stack_sidebar(assistant.upcast_ref::<gtk4::Widget>()) {
        stack_sidebar.add_css_class("marco-settings-sidebar");
        stack_sidebar.add_css_class("marco-welcome-sidebar");
    }
}

fn infer_theme_class_from_settings(
    settings_manager: &Arc<marco_shared::logic::swanson::SettingsManager>,
) -> &'static str {
    let settings = settings_manager.get_settings();
    let scheme = settings
        .appearance
        .as_ref()
        .and_then(|a| a.editor_mode.as_deref())
        .unwrap_or("marco-light");

    if scheme.to_ascii_lowercase().contains("dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    }
}

/// Show the welcome screen for first-time users.
///
/// This is non-blocking - it will show the window and immediately return.
///
/// # Arguments
/// * `settings_manager` - Settings manager for saving preferences
/// * `parent` - Optional parent window to stay on top of
/// * `on_language_changed` - Optional callback when user changes language
/// * `on_theme_changed` - Optional callback when user changes theme (receives editor mode string)
pub fn show_welcome_screen(
    settings_manager: &Arc<marco_shared::logic::swanson::SettingsManager>,
    parent: Option<&Window>,
    on_language_changed: Option<Box<dyn Fn(Option<String>) + 'static>>,
    on_theme_changed: Option<Box<dyn Fn(String) + 'static>>,
) -> bool {
    log::info!("show_welcome_screen: Creating welcome assistant");

    // Determine initial theme from parent (preferred), fallback to settings.
    let initial_theme_class = parent
        .map(|p| {
            if p.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            }
        })
        .unwrap_or_else(|| infer_theme_class_from_settings(settings_manager));

    // Read initial settings.
    let (initial_language_setting, initial_telemetry_enabled, initial_editor_mode) = {
        let settings = settings_manager.get_settings();

        let initial_language_setting = settings.language.as_ref().and_then(|l| l.language.clone());

        let initial_telemetry_enabled = settings
            .telemetry
            .as_ref()
            .and_then(|t| t.enabled)
            .unwrap_or(false);

        let initial_editor_mode = settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.clone())
            .unwrap_or_else(|| "marco-light".to_string());

        (
            initial_language_setting,
            initial_telemetry_enabled,
            initial_editor_mode,
        )
    };

    // Load translations for the welcome screen.
    // We keep a local manager so we can apply language changes live in this assistant.
    let localization_manager = match SimpleLocalizationManager::new() {
        Ok(m) => m,
        Err(e) => {
            // If assets are missing, showing the welcome screen isn't critical.
            log::warn!(
                "Welcome assistant: failed to initialize localization manager: {}",
                e
            );
            return initial_telemetry_enabled;
        }
    };

    let localization_manager = Rc::new(localization_manager);

    let initial_locale_code = effective_locale_code(initial_language_setting.as_deref());
    if let Err(e) = localization_manager.load_locale(&initial_locale_code) {
        log::warn!(
            "Welcome assistant: failed to load locale '{}': {}. Falling back to English.",
            initial_locale_code,
            e
        );
        if initial_locale_code != "en" {
            if let Err(e) = localization_manager.load_locale("en") {
                log::error!(
                    "Welcome assistant: failed to load fallback locale 'en': {}",
                    e
                );
            }
        }
    }

    let translations = localization_manager.translations();

    // Ensure action widgets render in the assistant action area.
    // When GTK routes actions into a header bar, our custom titlebar can hide
    // assistant navigation controls on some Linux setups.
    #[allow(deprecated)]
    let assistant = Assistant::builder().use_header_bar(0).build();
    // Fixed size: keep the welcome flow visually stable and avoid scrollbars.
    assistant.set_default_size(860, 720);
    assistant.set_resizable(false);
    assistant.set_modal(false); // Keep non-blocking behavior
    assistant.set_hide_on_close(true);

    assistant.set_title(Some(&translations.welcome.window_title));

    // Custom Marco titlebar (consistent window controls and styling)
    let (titlebar, titlebar_close_button) = crate::ui::titlebar::create_custom_titlebar(
        assistant.upcast_ref(),
        &translations.welcome.window_title,
    );
    assistant.set_titlebar(Some(&titlebar));

    // Apply dialog and theme CSS classes (reuses dialog.rs palette + typography)
    assistant.add_css_class("marco-dialog");
    assistant.add_css_class("marco-welcome-assistant");
    assistant.add_css_class(initial_theme_class);

    // If parent window is provided, set as transient to stay on top
    if let Some(parent_window) = parent {
        assistant.set_transient_for(Some(parent_window));
        assistant.set_destroy_with_parent(false);
    }

    // ---------------------------------------------------------------------
    // Page 1: Info (features)
    // ---------------------------------------------------------------------
    let intro_scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    intro_scrolled.add_css_class("editor-scrolled");

    let intro_box = GtkBox::new(Orientation::Vertical, 24);
    intro_box.add_css_class("marco-dialog-content");
    intro_box.set_margin_start(24);
    intro_box.set_margin_end(24);
    intro_box.set_margin_top(24);
    intro_box.set_margin_bottom(24);
    intro_scrolled.set_child(Some(&intro_box));

    let title_label = Label::builder().use_markup(true).xalign(0.0).build();
    title_label.add_css_class("marco-dialog-title");
    intro_box.append(&title_label);

    let subtitle_label = Label::builder().use_markup(true).xalign(0.0).build();
    subtitle_label.add_css_class("marco-dialog-message");
    intro_box.append(&subtitle_label);

    let features_header_label = Label::builder().use_markup(true).xalign(0.0).build();
    features_header_label.add_css_class("marco-dialog-section-label");
    features_header_label.add_css_class("marco-dialog-section-label-strong");
    intro_box.append(&features_header_label);

    let feature_strings = [
        (
            "📝",
            translations.welcome.feature_live_preview_title.clone(),
            translations
                .welcome
                .feature_live_preview_description
                .clone(),
        ),
        (
            "🎨",
            translations.welcome.feature_themes_title.clone(),
            translations.welcome.feature_themes_description.clone(),
        ),
        (
            "⚡",
            translations.welcome.feature_fast_title.clone(),
            translations.welcome.feature_fast_description.clone(),
        ),
        (
            "🔒",
            translations.welcome.feature_privacy_title.clone(),
            translations.welcome.feature_privacy_description.clone(),
        ),
        (
            "📊",
            translations.welcome.feature_markdown_title.clone(),
            translations.welcome.feature_markdown_description.clone(),
        ),
    ];

    let mut feature_title_labels: Vec<Label> = Vec::with_capacity(feature_strings.len());
    let mut feature_desc_labels: Vec<Label> = Vec::with_capacity(feature_strings.len());

    for (icon, title_text, description_text) in feature_strings {
        let row = GtkBox::new(Orientation::Horizontal, 12);

        let icon_label = Label::builder()
            .label(format!("<span size='x-large'>{}</span>", icon))
            .use_markup(true)
            .valign(Align::Start)
            .build();
        row.append(&icon_label);

        let text_box = GtkBox::new(Orientation::Vertical, 4);

        let title_label = Label::builder().use_markup(true).xalign(0.0).build();
        title_label.add_css_class("marco-dialog-option-title");
        text_box.append(&title_label);

        let desc_label = Label::builder().xalign(0.0).wrap(true).build();
        desc_label.add_css_class("marco-dialog-option-desc");
        text_box.append(&desc_label);

        feature_title_labels.push(title_label.clone());
        feature_desc_labels.push(desc_label.clone());

        // Initial population
        title_label.set_markup(&format!("<b>{}</b>", escape_markup(&title_text)));
        desc_label.set_text(&description_text);

        row.append(&text_box);
        intro_box.append(&row);
    }

    assistant.append_page(&intro_scrolled);
    assistant.set_page_title(&intro_scrolled, &translations.welcome.page_info);
    assistant.set_page_type(&intro_scrolled, AssistantPageType::Custom);
    assistant.set_page_complete(&intro_scrolled, true);

    // ---------------------------------------------------------------------
    // Page 2: Language selection
    // ---------------------------------------------------------------------
    let language_scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    language_scrolled.add_css_class("editor-scrolled");

    let language_box = GtkBox::new(Orientation::Vertical, 16);
    language_box.add_css_class("marco-dialog-content");
    language_box.set_margin_start(24);
    language_box.set_margin_end(24);
    language_box.set_margin_top(24);
    language_box.set_margin_bottom(24);
    language_scrolled.set_child(Some(&language_box));

    let language_header_label = Label::builder().use_markup(true).xalign(0.0).build();
    language_header_label.add_css_class("marco-dialog-section-label");
    language_header_label.add_css_class("marco-dialog-section-label-strong");
    language_box.append(&language_header_label);

    let language_description_label = Label::builder().wrap(true).xalign(0.0).build();
    language_description_label.add_css_class("marco-dialog-message");
    language_box.append(&language_description_label);

    // Build dropdown: "System Default" + discovered locales.
    let available_locales = localization_manager.available_locale_infos();
    let mut language_labels: Vec<String> = Vec::with_capacity(1 + available_locales.len());
    let mut language_codes: Vec<Option<String>> = Vec::with_capacity(1 + available_locales.len());

    language_labels.push(translations.settings.language.system_default.clone());
    language_codes.push(None);

    for locale in &available_locales {
        language_labels.push(format!("{} ({})", locale.native_name, locale.code));
        language_codes.push(Some(locale.code.clone()));
    }

    let language_options: Vec<&str> = language_labels.iter().map(|s| s.as_str()).collect();
    let language_string_list = StringList::new(&language_options);

    let language_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");

    let lang_dropdown = DropDown::new(
        Some(language_string_list.clone()),
        Some(language_expression),
    );
    lang_dropdown.add_css_class("marco-dropdown");
    lang_dropdown.add_css_class(initial_theme_class);

    let language_codes = Rc::new(language_codes);

    let initial_index = initial_language_setting
        .as_deref()
        .and_then(|code| {
            language_codes
                .iter()
                .position(|entry| entry.as_deref() == Some(code))
        })
        .unwrap_or(0);
    lang_dropdown.set_selected(initial_index as u32);

    language_box.append(&lang_dropdown);

    assistant.append_page(&language_scrolled);
    assistant.set_page_title(&language_scrolled, &translations.welcome.page_language);
    assistant.set_page_type(&language_scrolled, AssistantPageType::Custom);
    assistant.set_page_complete(&language_scrolled, true);

    // ---------------------------------------------------------------------
    // Page 3: Appearance (light / dark mode)
    // ---------------------------------------------------------------------
    let appearance_scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    appearance_scrolled.add_css_class("editor-scrolled");

    let appearance_box = GtkBox::new(Orientation::Vertical, 16);
    appearance_box.add_css_class("marco-dialog-content");
    appearance_box.set_margin_start(24);
    appearance_box.set_margin_end(24);
    appearance_box.set_margin_top(24);
    appearance_box.set_margin_bottom(24);
    appearance_scrolled.set_child(Some(&appearance_box));

    let appearance_header_label = Label::builder().use_markup(true).xalign(0.0).build();
    appearance_header_label.add_css_class("marco-dialog-section-label");
    appearance_header_label.add_css_class("marco-dialog-section-label-strong");
    appearance_box.append(&appearance_header_label);

    let appearance_description_label = Label::builder().wrap(true).xalign(0.0).build();
    appearance_description_label.add_css_class("marco-dialog-message");
    appearance_box.append(&appearance_description_label);

    let theme_radio_box = GtkBox::new(Orientation::Vertical, 8);
    theme_radio_box.set_margin_top(8);
    appearance_box.append(&theme_radio_box);

    let light_radio = CheckButton::with_label(&translations.welcome.appearance_light);
    light_radio.add_css_class("marco-radio");
    light_radio.add_css_class(initial_theme_class);

    let dark_radio = CheckButton::with_label(&translations.welcome.appearance_dark);
    dark_radio.add_css_class("marco-radio");
    dark_radio.add_css_class(initial_theme_class);
    dark_radio.set_group(Some(&light_radio));

    let is_dark = initial_editor_mode.contains("dark");
    light_radio.set_active(!is_dark);
    dark_radio.set_active(is_dark);

    theme_radio_box.append(&light_radio);
    theme_radio_box.append(&dark_radio);

    assistant.append_page(&appearance_scrolled);
    assistant.set_page_title(&appearance_scrolled, &translations.welcome.page_appearance);
    assistant.set_page_type(&appearance_scrolled, AssistantPageType::Custom);
    assistant.set_page_complete(&appearance_scrolled, true);

    // ---------------------------------------------------------------------
    // Page 4: Telemetry placeholder (disabled)
    // ---------------------------------------------------------------------
    let telemetry_scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();
    telemetry_scrolled.add_css_class("editor-scrolled");

    let telemetry_box = GtkBox::new(Orientation::Vertical, 16);
    telemetry_box.add_css_class("marco-dialog-content");
    telemetry_box.set_margin_start(24);
    telemetry_box.set_margin_end(24);
    telemetry_box.set_margin_top(24);
    telemetry_box.set_margin_bottom(24);
    telemetry_scrolled.set_child(Some(&telemetry_box));

    let telemetry_header_label = Label::builder().use_markup(true).xalign(0.0).build();
    telemetry_header_label.add_css_class("marco-dialog-section-label");
    telemetry_header_label.add_css_class("marco-dialog-section-label-strong");
    telemetry_box.append(&telemetry_header_label);

    let telemetry_not_implemented_label = Label::new(None);
    telemetry_not_implemented_label.set_wrap(true);
    telemetry_not_implemented_label.set_xalign(0.0);
    telemetry_not_implemented_label.add_css_class("settings-note");
    telemetry_box.append(&telemetry_not_implemented_label);

    // This container holds the (future) telemetry controls, but is currently disabled.
    let telemetry_disabled_box = GtkBox::new(Orientation::Vertical, 12);
    telemetry_disabled_box.set_sensitive(false);

    let telemetry_intro_label = Label::new(None);
    telemetry_intro_label.set_wrap(true);
    telemetry_intro_label.set_xalign(0.0);
    telemetry_intro_label.add_css_class("marco-dialog-message");
    telemetry_disabled_box.append(&telemetry_intro_label);

    let telemetry_checkbox = CheckButton::new();
    telemetry_checkbox.add_css_class("marco-checkbutton");
    telemetry_checkbox.add_css_class(initial_theme_class);
    telemetry_checkbox.set_active(initial_telemetry_enabled);
    telemetry_disabled_box.append(&telemetry_checkbox);

    let privacy_details_label = Label::new(None);
    privacy_details_label.set_use_markup(true);
    privacy_details_label.set_wrap(true);
    privacy_details_label.set_xalign(0.0);
    privacy_details_label.set_margin_start(12);
    privacy_details_label.add_css_class("marco-dialog-option-desc");
    telemetry_disabled_box.append(&privacy_details_label);

    telemetry_box.append(&telemetry_disabled_box);

    assistant.append_page(&telemetry_scrolled);
    assistant.set_page_title(&telemetry_scrolled, &translations.welcome.page_telemetry);
    assistant.set_page_type(&telemetry_scrolled, AssistantPageType::Custom);
    assistant.set_page_complete(&telemetry_scrolled, true);

    // ---------------------------------------------------------------------
    // Action buttons (custom, no Cancel)
    // ---------------------------------------------------------------------
    // Theme the assistant's built-in left sidebar like Settings.
    // Internal assistant children can be created lazily, so we apply now and again after present.
    apply_welcome_sidebar_classes(&assistant);

    let back_button = Button::with_label(&translations.welcome.back_button);
    back_button.add_css_class("marco-btn");
    back_button.add_css_class("marco-btn-yellow");

    let next_button = Button::with_label(&translations.welcome.next_button);
    next_button.add_css_class("marco-btn");
    next_button.add_css_class("marco-btn-blue");

    let finish_button = Button::with_label(&translations.welcome.finish_button);
    finish_button.add_css_class("marco-btn");
    finish_button.add_css_class("marco-btn-blue");

    let end_buttons = GtkBox::new(Orientation::Horizontal, 12);
    end_buttons.append(&next_button);
    end_buttons.append(&finish_button);

    let action_row = CenterBox::new();
    action_row.set_hexpand(true);
    action_row.set_start_widget(Some(&back_button));
    action_row.set_end_widget(Some(&end_buttons));

    #[allow(deprecated)]
    assistant.add_action_widget(&action_row);

    // Force deterministic initial state: start on first page and sync nav buttons.
    assistant.set_current_page(0);

    let sync_nav_buttons = {
        let back_button = back_button.clone();
        let next_button = next_button.clone();
        let finish_button = finish_button.clone();
        move |assistant: &gtk4::Assistant| {
            let current_page = assistant.current_page();
            let n_pages = assistant.n_pages();

            back_button.set_visible(current_page > 0);

            if current_page >= 0 && current_page + 1 >= n_pages {
                next_button.set_visible(false);
                finish_button.set_visible(true);
            } else {
                next_button.set_visible(true);
                finish_button.set_visible(false);
            }
        }
    };

    sync_nav_buttons(&assistant);

    // ---------------------------------------------------------------------
    // Persistence helpers
    // ---------------------------------------------------------------------
    let current_language_setting_rc: Rc<RefCell<Option<String>>> =
        Rc::new(RefCell::new(initial_language_setting.clone()));

    let current_editor_mode_rc: Rc<RefCell<String>> =
        Rc::new(RefCell::new(initial_editor_mode.clone()));

    let queue_save_preferences = {
        let settings_manager = settings_manager.clone();
        let current_language_setting_rc = current_language_setting_rc.clone();
        let current_editor_mode_rc = current_editor_mode_rc.clone();
        move || {
            let selected_language = current_language_setting_rc.borrow().clone();
            let selected_editor_mode = current_editor_mode_rc.borrow().clone();

            log::info!(
                "Welcome assistant: queue saving preferences (language={:?}, theme={})",
                selected_language,
                selected_editor_mode
            );

            let settings_manager = settings_manager.clone();
            gtk4::glib::idle_add_local_once(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    if s.telemetry.is_none() {
                        s.telemetry =
                            Some(marco_shared::logic::swanson::TelemetrySettings::default());
                    }
                    if let Some(ref mut telemetry) = s.telemetry {
                        // Showing the assistant counts as completing first-run.
                        telemetry.first_run_dialog_shown = Some(true);
                        // Telemetry is currently disabled in the welcome assistant.
                        // We intentionally do not change telemetry.enabled here.
                    }

                    if s.language.is_none() {
                        s.language =
                            Some(marco_shared::logic::swanson::LanguageSettings::default());
                    }
                    if let Some(ref mut language) = s.language {
                        language.language = selected_language.clone();
                    }

                    if s.appearance.is_none() {
                        s.appearance =
                            Some(marco_shared::logic::swanson::AppearanceSettings::default());
                    }
                    if let Some(ref mut appearance) = s.appearance {
                        appearance.editor_mode = Some(selected_editor_mode.clone());
                    }
                }) {
                    log::error!("Failed to save welcome screen preferences: {}", e);
                }
            });
        }
    };

    // ---------------------------------------------------------------------
    // Live UI translation updates
    // ---------------------------------------------------------------------
    let apply_translations = {
        let assistant = assistant.clone();
        let titlebar = titlebar.clone();

        let intro_scrolled = intro_scrolled.clone();
        let language_scrolled = language_scrolled.clone();
        let appearance_scrolled = appearance_scrolled.clone();
        let telemetry_scrolled = telemetry_scrolled.clone();

        let title_label = title_label.clone();
        let subtitle_label = subtitle_label.clone();
        let features_header_label = features_header_label.clone();

        let language_header_label = language_header_label.clone();
        let language_description_label = language_description_label.clone();

        let appearance_header_label = appearance_header_label.clone();
        let appearance_description_label = appearance_description_label.clone();
        let light_radio = light_radio.clone();
        let dark_radio = dark_radio.clone();

        let telemetry_header_label = telemetry_header_label.clone();
        let telemetry_not_implemented_label = telemetry_not_implemented_label.clone();
        let telemetry_intro_label = telemetry_intro_label.clone();
        let telemetry_checkbox = telemetry_checkbox.clone();
        let privacy_details_label = privacy_details_label.clone();

        let back_button = back_button.clone();
        let next_button = next_button.clone();
        let finish_button = finish_button.clone();

        let feature_title_labels = feature_title_labels;
        let feature_desc_labels = feature_desc_labels;

        let language_string_list = language_string_list.clone();

        move |t: &Translations| {
            assistant.set_title(Some(&t.welcome.window_title));

            // Keep the custom titlebar label in sync with the window title.
            if let Some(title_widget) = titlebar.title_widget() {
                if let Ok(label) = title_widget.downcast::<Label>() {
                    label.set_text(&t.welcome.window_title);
                }
            }

            assistant.set_page_title(&intro_scrolled, &t.welcome.page_info);
            assistant.set_page_title(&language_scrolled, &t.welcome.page_language);
            assistant.set_page_title(&appearance_scrolled, &t.welcome.page_appearance);
            assistant.set_page_title(&telemetry_scrolled, &t.welcome.page_telemetry);

            title_label.set_markup(&format!(
                "<span size='xx-large' weight='bold'>{}</span>",
                escape_markup(&t.welcome.window_title)
            ));
            subtitle_label.set_markup(&format!(
                "<span size='large'>{}</span>",
                escape_markup(&t.welcome.subtitle)
            ));
            features_header_label.set_markup(&format!(
                "<span size='large' weight='bold'>{}</span>",
                escape_markup(&t.welcome.key_features_title)
            ));

            // Features (fixed order)
            let feature_titles = [
                &t.welcome.feature_live_preview_title,
                &t.welcome.feature_themes_title,
                &t.welcome.feature_fast_title,
                &t.welcome.feature_privacy_title,
                &t.welcome.feature_markdown_title,
            ];
            let feature_descs = [
                &t.welcome.feature_live_preview_description,
                &t.welcome.feature_themes_description,
                &t.welcome.feature_fast_description,
                &t.welcome.feature_privacy_description,
                &t.welcome.feature_markdown_description,
            ];

            for (i, label) in feature_title_labels.iter().enumerate() {
                if let Some(title) = feature_titles.get(i) {
                    label.set_markup(&format!("<b>{}</b>", escape_markup(title)));
                }
            }
            for (i, label) in feature_desc_labels.iter().enumerate() {
                if let Some(desc) = feature_descs.get(i) {
                    label.set_text(desc);
                }
            }

            language_header_label.set_markup(&format!(
                "<span size='large' weight='bold'>{}</span>",
                escape_markup(&t.welcome.language_header)
            ));
            language_description_label.set_text(&t.welcome.language_description);

            appearance_header_label.set_markup(&format!(
                "<span size='large' weight='bold'>{}</span>",
                escape_markup(&t.welcome.appearance_header)
            ));
            appearance_description_label.set_text(&t.welcome.appearance_description);
            light_radio.set_label(Some(&t.welcome.appearance_light));
            dark_radio.set_label(Some(&t.welcome.appearance_dark));

            // Update "System Default" dropdown label (index 0) when language changes.
            let additions = [t.settings.language.system_default.as_str()];
            language_string_list.splice(0, 1, &additions);

            telemetry_header_label.set_markup(&format!(
                "<span size='large' weight='bold'>{}</span>",
                escape_markup(&t.welcome.telemetry_header)
            ));
            telemetry_not_implemented_label.set_text(&t.welcome.telemetry_not_implemented);
            telemetry_intro_label.set_text(&t.welcome.telemetry_intro);
            telemetry_checkbox.set_label(Some(&t.welcome.telemetry_checkbox_label));
            privacy_details_label.set_markup(&t.welcome.telemetry_privacy_details);

            back_button.set_label(&t.welcome.back_button);
            next_button.set_label(&t.welcome.next_button);
            finish_button.set_label(&t.welcome.finish_button);
        }
    };

    // Populate all translated labels once (now that widgets exist).
    {
        title_label.set_markup(&format!(
            "<span size='xx-large' weight='bold'>{}</span>",
            escape_markup(&translations.welcome.window_title)
        ));
        subtitle_label.set_markup(&format!(
            "<span size='large'>{}</span>",
            escape_markup(&translations.welcome.subtitle)
        ));
        features_header_label.set_markup(&format!(
            "<span size='large' weight='bold'>{}</span>",
            escape_markup(&translations.welcome.key_features_title)
        ));

        language_header_label.set_markup(&format!(
            "<span size='large' weight='bold'>{}</span>",
            escape_markup(&translations.welcome.language_header)
        ));
        language_description_label.set_text(&translations.welcome.language_description);

        appearance_header_label.set_markup(&format!(
            "<span size='large' weight='bold'>{}</span>",
            escape_markup(&translations.welcome.appearance_header)
        ));
        appearance_description_label.set_text(&translations.welcome.appearance_description);

        telemetry_header_label.set_markup(&format!(
            "<span size='large' weight='bold'>{}</span>",
            escape_markup(&translations.welcome.telemetry_header)
        ));
        telemetry_not_implemented_label.set_text(&translations.welcome.telemetry_not_implemented);
        telemetry_intro_label.set_text(&translations.welcome.telemetry_intro);
        telemetry_checkbox.set_label(Some(&translations.welcome.telemetry_checkbox_label));
        privacy_details_label.set_markup(&translations.welcome.telemetry_privacy_details);
    }

    // ---------------------------------------------------------------------
    // Signal handlers (navigation / escape / close-request / prepare)
    // ---------------------------------------------------------------------
    // Keep welcome assistant theme in sync with parent while open.
    if let Some(parent_window) = parent {
        let parent_widget = parent_window.upcast_ref::<gtk4::Widget>().clone();
        let assistant_for_theme = assistant.clone();
        let lang_dropdown_for_theme = lang_dropdown.clone();
        let telemetry_checkbox_for_theme = telemetry_checkbox.clone();
        let light_radio_for_theme = light_radio.clone();
        let dark_radio_for_theme = dark_radio.clone();
        let theme_class_state = Rc::new(RefCell::new(initial_theme_class.to_string()));

        parent_widget.connect_notify_local(Some("css-classes"), move |widget, _| {
            let next_theme = if widget.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            };

            {
                let mut state = theme_class_state.borrow_mut();
                if state.as_str() == next_theme {
                    return;
                }
                *state = next_theme.to_string();
            }

            assistant_for_theme.remove_css_class("marco-theme-dark");
            assistant_for_theme.remove_css_class("marco-theme-light");
            assistant_for_theme.add_css_class(next_theme);

            lang_dropdown_for_theme.remove_css_class("marco-theme-dark");
            lang_dropdown_for_theme.remove_css_class("marco-theme-light");
            lang_dropdown_for_theme.add_css_class(next_theme);

            telemetry_checkbox_for_theme.remove_css_class("marco-theme-dark");
            telemetry_checkbox_for_theme.remove_css_class("marco-theme-light");
            telemetry_checkbox_for_theme.add_css_class(next_theme);

            light_radio_for_theme.remove_css_class("marco-theme-dark");
            light_radio_for_theme.remove_css_class("marco-theme-light");
            light_radio_for_theme.add_css_class(next_theme);

            dark_radio_for_theme.remove_css_class("marco-theme-dark");
            dark_radio_for_theme.remove_css_class("marco-theme-light");
            dark_radio_for_theme.add_css_class(next_theme);
        });
    }

    // Titlebar close button should behave like window-manager close (X)
    {
        let assistant = assistant.clone();
        let queue_save_preferences = queue_save_preferences.clone();
        titlebar_close_button.connect_clicked(move |_| {
            log::info!("Welcome assistant: titlebar close clicked");
            queue_save_preferences();
            assistant.hide();
        });
    }

    // Navigation handlers
    {
        let assistant = assistant.clone();
        back_button.connect_clicked(move |_| {
            let current = assistant.current_page();
            if current > 0 {
                assistant.set_current_page(current - 1);
            }
        });
    }

    {
        let assistant = assistant.clone();
        next_button.connect_clicked(move |_| {
            let current = assistant.current_page();
            let n_pages = assistant.n_pages();
            if current >= 0 && current + 1 < n_pages {
                assistant.set_current_page(current + 1);
            }
        });
    }

    {
        let assistant = assistant.clone();
        let queue_save_preferences = queue_save_preferences.clone();
        finish_button.connect_clicked(move |_| {
            log::info!("Welcome assistant: finish");
            // Close immediately, then persist asynchronously.
            queue_save_preferences();
            assistant.hide();
        });
    }

    // Persist preferences even if the window is closed via window manager / Escape
    {
        let assistant_for_close = assistant.clone();
        let queue_save_preferences = queue_save_preferences.clone();
        assistant.connect_close_request(move |_| {
            log::info!("Welcome assistant: close-request");
            queue_save_preferences();
            assistant_for_close.hide();
            gtk4::glib::Propagation::Stop
        });
    }

    {
        let assistant_for_escape = assistant.clone();
        let queue_save_preferences = queue_save_preferences.clone();
        assistant.connect_escape(move |_| {
            log::info!("Welcome assistant: escape");
            queue_save_preferences();
            assistant_for_escape.hide();
        });
    }

    assistant.connect_prepare({
        let sync_nav_buttons = sync_nav_buttons.clone();
        move |assistant, _page| {
            apply_welcome_sidebar_classes(assistant);
            sync_nav_buttons(assistant);
        }
    });

    // ---------------------------------------------------------------------
    // Language dropdown behavior
    // ---------------------------------------------------------------------
    {
        let localization_manager = localization_manager.clone();
        let language_codes = language_codes.clone();
        let current_language_setting_rc = current_language_setting_rc.clone();
        let queue_save_preferences = queue_save_preferences.clone();

        // Keep the callback alive and shareable across closures.
        let on_language_changed = Rc::new(on_language_changed);

        // Prevent re-entrant update storms (GTK can emit notify again while we update models).
        let is_applying_translations = Rc::new(Cell::new(false));
        let update_scheduled = Rc::new(Cell::new(false));

        // Widgets updated by translations
        let apply_translations = apply_translations.clone();

        lang_dropdown.connect_selected_notify(move |combo| {
            // If this notify was triggered by our own translation updates, ignore it.
            if is_applying_translations.get() {
                return;
            }

            let selected_index = combo.selected() as usize;
            let selected_code = language_codes
                .get(selected_index)
                .and_then(|entry| entry.clone());

            // Ignore spurious repeated notifications that don't actually change the value.
            if *current_language_setting_rc.borrow() == selected_code {
                return;
            }

            *current_language_setting_rc.borrow_mut() = selected_code;

            // Coalesce rapid changes into a single idle update.
            if update_scheduled.get() {
                return;
            }
            update_scheduled.set(true);

            let localization_manager = localization_manager.clone();
            let current_language_setting_rc = current_language_setting_rc.clone();
            let apply_translations = apply_translations.clone();
            let on_language_changed = on_language_changed.clone();
            let is_applying_translations = is_applying_translations.clone();
            let update_scheduled = update_scheduled.clone();
            let queue_save_preferences = queue_save_preferences.clone();

            gtk4::glib::idle_add_local_once(move || {
                update_scheduled.set(false);

                // Persist preferences once per coalesced selection change.
                queue_save_preferences();

                let selected_code = current_language_setting_rc.borrow().clone();

                let locale_code = effective_locale_code(selected_code.as_deref());
                if let Err(e) = localization_manager.load_locale(&locale_code) {
                    log::warn!(
                        "Welcome assistant: failed to load locale '{}': {}. Falling back to English.",
                        locale_code,
                        e
                    );
                    if locale_code != "en" {
                        if let Err(e) = localization_manager.load_locale("en") {
                            log::error!(
                                "Welcome assistant: failed to load fallback locale 'en': {}",
                                e
                            );
                        }
                    }
                }

                let new_translations = localization_manager.translations();
                is_applying_translations.set(true);
                apply_translations(&new_translations);
                is_applying_translations.set(false);

                if let Some(ref callback) = on_language_changed.as_ref() {
                    callback(selected_code);
                }
            });
        });
    }

    // ---------------------------------------------------------------------
    // Appearance radio button behavior
    // ---------------------------------------------------------------------
    {
        let on_theme_changed = Rc::new(on_theme_changed);
        let current_editor_mode_rc = current_editor_mode_rc.clone();
        let queue_save_preferences = queue_save_preferences.clone();

        // Widgets that carry the theme CSS class
        let assistant_for_radio = assistant.clone();
        let lang_dropdown_for_radio = lang_dropdown.clone();
        let telemetry_checkbox_for_radio = telemetry_checkbox.clone();
        let light_radio_for_radio = light_radio.clone();
        let dark_radio_for_radio = dark_radio.clone();

        dark_radio.connect_toggled(move |btn| {
            let is_dark = btn.is_active();
            let editor_mode = if is_dark { "marco-dark" } else { "marco-light" }.to_string();
            let theme_class = if is_dark {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            };
            let old_class = if is_dark {
                "marco-theme-light"
            } else {
                "marco-theme-dark"
            };

            *current_editor_mode_rc.borrow_mut() = editor_mode.clone();

            // Apply CSS class to all themed widgets immediately
            for widget in [
                assistant_for_radio.upcast_ref::<gtk4::Widget>(),
                lang_dropdown_for_radio.upcast_ref::<gtk4::Widget>(),
                telemetry_checkbox_for_radio.upcast_ref::<gtk4::Widget>(),
                light_radio_for_radio.upcast_ref::<gtk4::Widget>(),
                dark_radio_for_radio.upcast_ref::<gtk4::Widget>(),
            ] {
                widget.remove_css_class(old_class);
                widget.add_css_class(theme_class);
            }

            queue_save_preferences();

            if let Some(ref callback) = on_theme_changed.as_ref() {
                callback(editor_mode);
            }
        });
    }

    // Show the assistant
    log::info!("show_welcome_screen: Presenting welcome assistant");
    assistant.present();
    assistant.present_with_time((gtk4::glib::monotonic_time() / 1000) as u32);

    {
        let assistant = assistant.clone();
        gtk4::glib::idle_add_local_once(move || {
            apply_welcome_sidebar_classes(&assistant);
        });
    }

    initial_telemetry_enabled
}

/// Check if welcome screen should be shown.
///
/// Returns true if the dialog has not been shown yet.
pub fn should_show_welcome_screen(
    settings_manager: &Arc<marco_shared::logic::swanson::SettingsManager>,
) -> bool {
    let settings = settings_manager.get_settings();

    let dialog_shown = settings
        .telemetry
        .as_ref()
        .and_then(|t| t.first_run_dialog_shown)
        .unwrap_or(false);

    log::info!(
        "should_show_welcome_screen: first_run_dialog_shown={}",
        dialog_shown
    );

    !dialog_shown
}
