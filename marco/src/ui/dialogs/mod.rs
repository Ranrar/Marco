pub mod admonition;
pub mod diagnostics_reference;
pub mod export;
pub mod export_complete;
pub mod exporting;
pub mod lists;
pub mod math;
pub mod mention;
pub mod mermaid;
pub mod open_local_file;
pub mod save;
pub mod search;
pub mod sliderdeck;
pub mod tables;
pub mod tabs;
pub mod welcome_screen;

/// Helper used by dialog modules to fetch the current UI translations.
///
/// Reads the configured locale from the persisted settings (falling back to
/// the detected system locale, then English) and loads the matching
/// translation file. Falls back to compiled-in English defaults if the
/// localisation manager cannot be initialised (for example when running
/// outside a normal build where translation assets are missing).
pub(crate) fn current_translations() -> crate::components::language::Translations {
    use crate::components::language::{LocalizationProvider, SimpleLocalizationManager};

    let manager = match SimpleLocalizationManager::new() {
        Ok(m) => m,
        Err(_) => {
            return crate::components::language::default_translations::load_default_translations();
        }
    };

    // Resolve the locale code the same way `marco/src/main.rs` does at
    // startup: explicit setting → detected system locale → "en".
    let locale_code = marco_shared::paths::MarcoPaths::new()
        .ok()
        .and_then(|paths| {
            marco_shared::logic::swanson::SettingsManager::initialize(paths.settings_file()).ok()
        })
        .and_then(|sm| sm.get_settings().language.and_then(|l| l.language))
        .or_else(marco_shared::paths::detect_system_locale_bcp47)
        .unwrap_or_else(|| "en".to_string());

    // `new()` already loaded English during construction, so skip the
    // reload when that's also the resolved locale.
    if locale_code != "en" {
        manager.load_locale_with_fallback(&locale_code);
    }

    manager.translations()
}
