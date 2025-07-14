use std::collections::HashMap;
use std::sync::Mutex;

static TRANSLATIONS: Mutex<Option<HashMap<String, HashMap<String, serde_yaml::Value>>>> =
    Mutex::new(None);
static CURRENT_LOCALE: Mutex<String> = Mutex::new(String::new());

// Instead of storing callbacks, we'll use a simple flag to indicate language changes
static LANGUAGE_CHANGED: Mutex<bool> = Mutex::new(false);

pub fn init_localization() {
    // Try to detect system locale, fallback to English
    let locale = get_system_locale().unwrap_or_else(|| "en".to_string());
    load_translations();
    set_locale(&locale);
}

pub fn check_language_changed() -> bool {
    if let Ok(mut changed) = LANGUAGE_CHANGED.lock() {
        if *changed {
            *changed = false;
            return true;
        }
    }
    false
}

fn load_translations() {
    let mut translations = HashMap::new();

    // Get the path relative to the executable or cargo workspace
    use crate::utils::cross_platform_resource::resolve_resource_path;
    // Load English translations
    let en_path = resolve_resource_path("assets/language/en", "main.yml");
    if let Ok(content) = std::fs::read_to_string(&en_path) {
        if let Ok(data) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(en_data) = data.get("en") {
                if let Some(mapping) = en_data.as_mapping() {
                    let mut en_map = HashMap::new();
                    for (k, v) in mapping {
                        if let Some(key_str) = k.as_str() {
                            en_map.insert(key_str.to_string(), v.clone());
                        }
                    }
                    translations.insert("en".to_string(), en_map);
                }
            }
        }
    }

    // Load Spanish translations
    let es_path = resolve_resource_path("assets/language/es", "main.yml");
    if let Ok(content) = std::fs::read_to_string(&es_path) {
        if let Ok(data) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(es_data) = data.get("es") {
                if let Some(mapping) = es_data.as_mapping() {
                    let mut es_map = HashMap::new();
                    for (k, v) in mapping {
                        if let Some(key_str) = k.as_str() {
                            es_map.insert(key_str.to_string(), v.clone());
                        }
                    }
                    translations.insert("es".to_string(), es_map);
                }
            }
        }
    }

    // Load French translations
    let fr_path = resolve_resource_path("assets/language/fr", "main.yml");
    if let Ok(content) = std::fs::read_to_string(&fr_path) {
        if let Ok(data) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(fr_data) = data.get("fr") {
                if let Some(mapping) = fr_data.as_mapping() {
                    let mut fr_map = HashMap::new();
                    for (k, v) in mapping {
                        if let Some(key_str) = k.as_str() {
                            fr_map.insert(key_str.to_string(), v.clone());
                        }
                    }
                    translations.insert("fr".to_string(), fr_map);
                }
            }
        }
    }

    // Load German translations
    let de_path = resolve_resource_path("assets/language/de", "main.yml");
    if let Ok(content) = std::fs::read_to_string(&de_path) {
        if let Ok(data) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
            if let Some(de_data) = data.get("de") {
                if let Some(mapping) = de_data.as_mapping() {
                    let mut de_map = HashMap::new();
                    for (k, v) in mapping {
                        if let Some(key_str) = k.as_str() {
                            de_map.insert(key_str.to_string(), v.clone());
                        }
                    }
                    translations.insert("de".to_string(), de_map);
                }
            }
        }
    }

    if let Ok(mut global_translations) = TRANSLATIONS.lock() {
        *global_translations = Some(translations);
    }
}

pub fn set_locale(locale: &str) {
    if let Ok(mut current) = CURRENT_LOCALE.lock() {
        *current = locale.to_string();
    }

    // Set the language changed flag
    if let Ok(mut changed) = LANGUAGE_CHANGED.lock() {
        *changed = true;
    }
}

pub fn get_current_locale() -> String {
    CURRENT_LOCALE
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| "en".to_string())
}

pub fn get_available_locales() -> Vec<(&'static str, &'static str)> {
    vec![
        ("en", "English"),
        ("es", "Español"),
        ("fr", "Français"),
        ("de", "Deutsch"),
    ]
}

// Get translation value by key path (e.g., "app.title")
fn get_nested_value(map: &HashMap<String, serde_yaml::Value>, key_path: &str) -> Option<String> {
    let keys: Vec<&str> = key_path.split('.').collect();
    let mut current_value;

    // Start with the root level
    if let Some(first_key) = keys.first() {
        current_value = map.get(*first_key);
    } else {
        return None;
    }

    // Navigate through the nested structure
    for key in keys.iter().skip(1) {
        if let Some(value) = current_value {
            if let Some(mapping) = value.as_mapping() {
                // Find the key in the mapping
                for (k, v) in mapping {
                    if let Some(key_str) = k.as_str() {
                        if key_str == *key {
                            current_value = Some(v);
                            break;
                        }
                    }
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    // Extract final string value
    if let Some(final_value) = current_value {
        if let Some(s) = final_value.as_str() {
            return Some(s.to_string());
        }
    }

    None
}

// Convenience function to translate text
pub fn tr(key: &str) -> String {
    let locale = get_current_locale();

    if let Ok(translations) = TRANSLATIONS.lock() {
        if let Some(ref trans) = *translations {
            if let Some(locale_trans) = trans.get(&locale) {
                if let Some(value) = get_nested_value(locale_trans, key) {
                    return value;
                }
            }

            // Fallback to English
            if let Some(en_trans) = trans.get("en") {
                if let Some(value) = get_nested_value(en_trans, key) {
                    return value;
                }
            }
        }
    }

    // Final fallback - return the key itself
    key.to_string()
}

// Convenience function to translate text with parameters
pub fn tr_with_args(key: &str, args: &HashMap<&str, &str>) -> String {
    let mut result = tr(key);

    for (k, v) in args {
        let placeholder = format!("%{{{}}}", k);
        result = result.replace(&placeholder, v);
    }

    result
}

fn get_system_locale() -> Option<String> {
    // Try to get locale from environment variables
    if let Ok(locale) = std::env::var("LANG") {
        // Extract language code (e.g., "en_US.UTF-8" -> "en")
        if let Some(lang) = locale.split('_').next() {
            return Some(lang.to_string());
        }
    }

    if let Ok(locale) = std::env::var("LC_ALL") {
        if let Some(lang) = locale.split('_').next() {
            return Some(lang.to_string());
        }
    }

    None
}
