use gtk4::prelude::*;
use gtk4::Box;
use std::rc::Rc;
// Import unified helper
use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::LocaleInfo;
use crate::components::language::SettingsLanguageTranslations;
use crate::components::language::Translations;

pub fn build_language_tab(
    settings_path: &str,
    translations: &SettingsLanguageTranslations,
    available_locales: &[LocaleInfo],
    i18n: &SettingsI18nRegistry,
    on_language_changed: Option<std::boxed::Box<dyn Fn(Option<String>) + 'static>>,
) -> Box {
    use gtk4::{
        Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression, StringList,
        StringObject,
    };
    use std::path::PathBuf;

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Language (Dropdown)
    // Create language dropdown with automatic checkmarks
    let mut language_labels: Vec<String> = Vec::with_capacity(1 + available_locales.len());
    let mut language_codes: Vec<Option<String>> = Vec::with_capacity(1 + available_locales.len());

    language_labels.push(translations.system_default.clone());
    language_codes.push(None);

    for locale in available_locales {
        // Display native language name from assets (fallback is locale code).
        language_labels.push(format!("{} ({})", locale.native_name, locale.code));
        language_codes.push(Some(locale.code.clone()));
    }

    let language_options: Vec<&str> = language_labels.iter().map(|s| s.as_str()).collect();

    // Step 1: Create StringList from language options
    let language_string_list = StringList::new(&language_options);

    // Bind "System Default" label for runtime language switching.
    // The other entries come from assets and remain stable.
    i18n.bind_string_list_item(
        &language_string_list,
        0,
        Rc::new(|t: &Translations| t.settings.language.system_default.clone()),
    );

    let language_codes = Rc::new(language_codes);

    // Step 2: Create PropertyExpression for string matching (required for DropDown)
    let language_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");

    // Step 3: Create DropDown with automatic checkmarks
    let lang_combo = DropDown::new(Some(language_string_list), Some(language_expression));
    lang_combo.add_css_class("marco-dropdown");
    let settings_manager_opt =
        match core::logic::swanson::SettingsManager::initialize(PathBuf::from(settings_path)) {
            Ok(settings_manager) => Some(settings_manager),
            Err(e) => {
                log::warn!(
                    "Failed to initialize SettingsManager for language settings: {}",
                    e
                );
                None
            }
        };

    let current_locale = settings_manager_opt
        .as_ref()
        .and_then(|manager| manager.get_settings().language)
        .and_then(|lang| lang.language);

    let initial_index = current_locale
        .as_deref()
        .and_then(|code| {
            language_codes
                .iter()
                .position(|entry| entry.as_deref() == Some(code))
        })
        .unwrap_or(0);
    lang_combo.set_selected(initial_index as u32); // Default to "System Default"

    // Create language row using unified helper (first and only row)
    if let Some(settings_manager) = settings_manager_opt {
        let on_language_changed = on_language_changed;
        let language_codes = language_codes.clone();
        lang_combo.connect_selected_notify(move |combo| {
            let selected_index = combo.selected() as usize;
            let selected_code = language_codes
                .get(selected_index)
                .and_then(|entry| entry.clone());

            if let Err(e) = settings_manager.update_settings(|settings| {
                if settings.language.is_none() {
                    settings.language = Some(core::logic::swanson::LanguageSettings::default());
                }
                if let Some(ref mut language) = settings.language {
                    language.language = selected_code.clone();
                }
            }) {
                log::error!("Failed to update language setting: {}", e);
            }

            if let Some(ref callback) = on_language_changed {
                callback(selected_code);
            }
        });
    }

    let lang_row = add_setting_row_i18n(
        i18n,
        &translations.label,
        &translations.description,
        Rc::new(|t: &Translations| t.settings.language.label.clone()),
        Rc::new(|t: &Translations| t.settings.language.description.clone()),
        &lang_combo,
        true, // First row - no top margin
    );
    container.append(&lang_row);

    container
}
