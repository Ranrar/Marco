//! Language component - Runtime-switchable translations for Marco UI
//!
//! This module provides internationalization support via simple TOML translation files
//! following the ISO 639-1 standard (two-letter language codes).
//!
//! ## Architecture
//! - `LocalizationProvider` trait for loading and managing translations
//! - `SimpleLocalizationManager` implementation with fallback to English
//! - `Translations` struct representing the complete UI text
//!
//! ## Usage
//! ```rust
//! use crate::components::language::{SimpleLocalizationManager, LocalizationProvider};
//!
//! let manager = SimpleLocalizationManager::new()?;
//! manager.load_locale("en")?;
//! let translations = manager.translations();
//!
//! // Use in UI
//! button.set_label(&translations.menu.file);
//! ```

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

mod default_translations;

/// Translation key-value store representing all UI strings
#[derive(Debug, Clone, Deserialize)]
pub struct Translations {
    pub menu: MenuTranslations,
    pub toolbar: ToolbarTranslations,
    pub footer: FooterTranslations,
    pub dialog: DialogTranslations,
    pub settings: SettingsTranslations,
    pub welcome: WelcomeTranslations,
    pub titlebar: TitlebarTranslations,
    pub messages: MessagesTranslations,
    pub search: SearchTranslations,
}

/// Welcome assistant (first-run) translations.
#[derive(Debug, Clone, Deserialize)]
pub struct WelcomeTranslations {
    pub window_title: String,
    pub subtitle: String,
    pub key_features_title: String,

    pub page_info: String,
    pub page_language: String,
    pub page_telemetry: String,

    pub language_header: String,
    pub language_description: String,

    pub telemetry_header: String,
    pub telemetry_intro: String,
    pub telemetry_checkbox_label: String,
    pub telemetry_privacy_details: String,
    pub telemetry_not_implemented: String,

    pub back_button: String,
    pub next_button: String,
    pub finish_button: String,

    pub feature_live_preview_title: String,
    pub feature_live_preview_description: String,
    pub feature_themes_title: String,
    pub feature_themes_description: String,
    pub feature_fast_title: String,
    pub feature_fast_description: String,
    pub feature_privacy_title: String,
    pub feature_privacy_description: String,
    pub feature_markdown_title: String,
    pub feature_markdown_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuTranslations {
    pub file: String,
    pub edit: String,
    pub view: String,
    pub format: String,
    pub help: String,
    pub document: String,
    pub bookmarks: String,
    pub new: String,
    pub open: String,
    pub save: String,
    pub save_as: String,
    pub export: String,
    pub export_pdf: String,
    pub settings: String,
    pub preferences: String,
    pub quit: String,
    pub recent: String,
    pub recent_files: String,
    pub no_recent: String,
    pub clear_recent: String,
    pub bold: String,
    pub italic: String,
    pub code: String,
    pub html_preview: String,
    pub code_view: String,
    pub link: String,
    pub image: String,
    pub undo: String,
    pub redo: String,
    pub cut: String,
    pub copy: String,
    pub paste: String,
    pub search_replace: String,
    pub document_builder: String,
    pub document_splitter: String,
    pub no_bookmarks: String,
    pub about: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolbarTranslations {
    pub headings: String,
    pub insert: String,
    pub bold: String,
    pub italic: String,
    pub code: String,
    pub strikethrough: String,
    pub bullet_list: String,
    pub numbered_list: String,
    pub h1: String,
    pub h2: String,
    pub h3: String,
    pub h4: String,
    pub h5: String,
    pub h6: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FooterTranslations {
    pub row: String,
    pub column: String,
    pub words: String,
    pub characters: String,
    pub ins: String,
    pub ovr: String,
    pub encoding_utf8: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DialogTranslations {
    pub open_file: String,
    pub save_file: String,
    pub unsaved_changes: String,
    pub unsaved_message: String,
    pub open_button: String,
    pub save_button: String,
    pub dont_save_button: String,
    pub cancel_button: String,
    pub about_title: String,
    pub preferences_title: String,
    pub save_changes_title: String,
    pub save_changes_prompt: String,
    pub save_changes_action_opening: String,
    pub save_changes_action_new_document: String,
    pub save_changes_action_quitting: String,
    pub save_changes_prefix: String,
    pub save_changes_secondary: String,
    pub save_without_saving: String,
    pub save_as_button: String,
    pub discard_tooltip: String,
    pub cancel_tooltip: String,
    pub save_tooltip: String,
    pub close_tooltip: String,
    pub open_markdown_title: String,
    pub save_markdown_title: String,
    pub filter_markdown: String,
    pub filter_all: String,
    pub overwrite_title: String,
    pub overwrite_secondary: String,
    pub overwrite_replace: String,
    pub overwrite_cancel: String,
    pub error_title_prefix: String,
    pub error_message_prefix: String,
    pub info_title_file_saved: String,
    pub info_message_file_saved: String,
    pub about_app_name: String,
    pub about_tagline: String,
    pub about_version: String,
    pub about_description: String,
    pub about_resources_title: String,
    pub about_link_github: String,
    pub about_link_issues: String,
    pub about_link_discuss: String,
    pub about_link_changelog: String,
    pub about_link_website: String,
    pub about_license_text: String,
    pub about_copyright: String,
    pub about_close_button: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTranslations {
    pub title: String,
    pub close: String,
    pub tabs: SettingsTabsTranslations,
    pub language: SettingsLanguageTranslations,
    pub editor: SettingsEditorTranslations,
    pub appearance: SettingsAppearanceTranslations,
    pub layout: SettingsLayoutTranslations,
    pub markdown: SettingsMarkdownTranslations,
    pub advanced: SettingsAdvancedTranslations,
    pub debug: SettingsDebugTranslations,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTabsTranslations {
    pub editor: String,
    pub layout: String,
    pub appearance: String,
    pub language: String,
    pub markdown: String,
    pub advanced: String,
    pub debug: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLanguageTranslations {
    pub label: String,
    pub description: String,
    pub system_default: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsEditorTranslations {
    pub font_label: String,
    pub font_description: String,
    pub font_size_label: String,
    pub font_size_description: String,
    pub line_height_label: String,
    pub line_height_description: String,
    pub line_wrapping_label: String,
    pub line_wrapping_description: String,
    pub auto_pairing_label: String,
    pub auto_pairing_description: String,
    pub show_invisibles_label: String,
    pub show_invisibles_description: String,
    pub tabs_to_spaces_label: String,
    pub tabs_to_spaces_description: String,
    pub syntax_colors_label: String,
    pub syntax_colors_description: String,
    pub linting_label: String,
    pub linting_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsAppearanceTranslations {
    pub preview_theme_label: String,
    pub preview_theme_description: String,
    pub color_mode_label: String,
    pub color_mode_description: String,
    pub color_mode_light: String,
    pub color_mode_dark: String,
    pub custom_css_label: String,
    pub custom_css_description: String,
    pub custom_css_button: String,
    pub ui_font_label: String,
    pub ui_font_description: String,
    pub ui_font_size_label: String,
    pub ui_font_size_description: String,
    pub ui_font_system_default: String,
    pub ui_font_sans: String,
    pub ui_font_serif: String,
    pub ui_font_monospace: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLayoutTranslations {
    pub view_mode_label: String,
    pub view_mode_description: String,
    pub view_mode_html: String,
    pub view_mode_source: String,
    pub sync_scrolling_label: String,
    pub sync_scrolling_description: String,
    pub split_label: String,
    pub split_description: String,
    pub line_numbers_label: String,
    pub line_numbers_description: String,
    pub text_direction_label: String,
    pub text_direction_description: String,
    pub text_direction_ltr: String,
    pub text_direction_rtl: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsMarkdownTranslations {
    pub toc_label: String,
    pub toc_description: String,
    pub metadata_label: String,
    pub metadata_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsAdvancedTranslations {
    pub telemetry_label: String,
    pub telemetry_description: String,
    pub log_to_file_label: String,
    pub log_to_file_description: String,
    pub my_data_label: String,
    pub my_data_description: String,
    pub my_data_button: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsDebugTranslations {
    pub debug_label: String,
    pub debug_description: String,
    pub debug_checkbox: String,
    pub umami_test_label: String,
    pub umami_test_description: String,
    pub umami_test_button: String,
    pub welcome_label: String,
    pub welcome_description: String,
    pub welcome_button: String,
    pub log_label: String,
    pub log_description: String,
    pub log_checkbox: String,
    pub log_paths_template: String,
    pub log_size_template: String,
    pub open_logs_button: String,
    pub delete_logs_button: String,
    pub refresh_button: String,
    pub delete_logs_confirm: String,
    pub log_enable_failed_title: String,
    pub log_enable_failed_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TitlebarTranslations {
    pub app_tooltip: String,
    pub layout_editor_only: String,
    pub layout_view_only: String,
    pub layout_detach_view: String,
    pub layout_restore_split: String,
    pub window_minimize: String,
    pub window_maximize_restore: String,
    pub window_close: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagesTranslations {
    pub file_saved: String,
    pub file_opened: String,
    pub export_complete: String,
    pub error_opening_file: String,
    pub error_saving_file: String,
    pub untitled_document: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchTranslations {
    pub title: String,
    pub close_tooltip: String,
    pub find_label: String,
    pub replace_label: String,
    pub search_placeholder: String,
    pub replace_placeholder: String,
    pub match_case: String,
    pub match_whole_word: String,
    pub match_markdown: String,
    pub use_regex: String,
    pub previous_button: String,
    pub next_button: String,
    pub replace_button: String,
    pub replace_all_button: String,
}

/// LocalizationProvider trait for translation management
pub trait LocalizationProvider {
    fn load_locale(&self, locale: &str) -> Result<(), LocalizationError>;
    fn translations(&self) -> Translations;
}

/// Locale discovered from `assets/language/*.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleInfo {
    pub code: String,
    pub native_name: String,
}

#[derive(Debug)]
pub enum LocalizationError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    InvalidLocaleCode(String),
    LocaleNotFound(String),
}

impl std::fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalizationError::Io(e) => write!(f, "I/O error: {}", e),
            LocalizationError::Parse(e) => write!(f, "Parse error: {}", e),
            LocalizationError::InvalidLocaleCode(code) => {
                write!(
                    f,
                    "Invalid locale code '{}' (must be ISO 639-1: 2 letters)",
                    code
                )
            }
            LocalizationError::LocaleNotFound(locale) => write!(f, "Locale '{}' not found", locale),
        }
    }
}

impl std::error::Error for LocalizationError {}

impl From<std::io::Error> for LocalizationError {
    fn from(error: std::io::Error) -> Self {
        LocalizationError::Io(error)
    }
}

impl From<toml::de::Error> for LocalizationError {
    fn from(error: toml::de::Error) -> Self {
        LocalizationError::Parse(error)
    }
}

/// Simple localization manager with fallback to English
pub struct SimpleLocalizationManager {
    current_locale: Arc<RwLock<String>>,
    translations: Arc<RwLock<Translations>>,
    assets_path: PathBuf,
    available_locales: Vec<LocaleInfo>,
}

impl SimpleLocalizationManager {
    /// Create a new localization manager
    ///
    /// Automatically loads English as the default locale.
    pub fn new() -> Result<Self, LocalizationError> {
        let shared_paths = core::paths::SharedPaths::new().map_err(|e| {
            LocalizationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to find asset root: {}", e),
            ))
        })?;
        let assets_path = shared_paths.asset_root().to_path_buf();

        let available_locales = Self::load_available_locale_infos(&assets_path);

        let manager = Self {
            current_locale: Arc::new(RwLock::new("en".to_string())),
            translations: Arc::new(RwLock::new(Self::load_default_translations())),
            assets_path,
            available_locales,
        };

        // Load English by default
        manager.load_locale("en")?;

        Ok(manager)
    }

    /// Return the locales discovered at startup from `assets/language/*.toml`.
    pub fn available_locale_infos(&self) -> Vec<LocaleInfo> {
        self.available_locales.clone()
    }

    /// Get the current active locale code.
    /// Used primarily for testing to verify locale switching.
    #[allow(dead_code)]
    pub fn current_locale(&self) -> String {
        self.current_locale.read().unwrap().clone()
    }

    fn load_available_locale_infos(assets_path: &Path) -> Vec<LocaleInfo> {
        let mut infos = Vec::new();

        for code in Self::scan_available_locales(assets_path) {
            let locale_path = assets_path.join("language").join(format!("{}.toml", code));

            let native_name = fs::read_to_string(&locale_path)
                .ok()
                .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
                .and_then(|value| {
                    value
                        .get("language")
                        .and_then(|section| section.get("native_name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| code.clone());

            infos.push(LocaleInfo { code, native_name });
        }

        infos
    }

    /// Load default/fallback English translations (minimal set to prevent crashes)
    fn load_default_translations() -> Translations {
        default_translations::load_default_translations()
    }

    /// Scan the language directory for available locale files
    fn scan_available_locales(assets_path: &Path) -> Vec<String> {
        let lang_dir = assets_path.join("language");
        let mut locales = Vec::new();

        if let Ok(entries) = fs::read_dir(lang_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".toml") && !filename.starts_with('.') {
                        if let Some(locale) = filename.strip_suffix(".toml") {
                            // Validate ISO 639-1 format (2 letters)
                            if locale.len() == 2 && locale.chars().all(|c| c.is_ascii_lowercase()) {
                                locales.push(locale.to_string());
                            }
                        }
                    }
                }
            }
        }

        locales.sort();
        locales
    }

    fn load_translations_from_value(value: &toml::Value, fallback: &Translations) -> Translations {
        Translations {
            menu: MenuTranslations {
                file: Self::get_string(value, &["menu", "file"], &fallback.menu.file),
                edit: Self::get_string(value, &["menu", "edit"], &fallback.menu.edit),
                view: Self::get_string(value, &["menu", "view"], &fallback.menu.view),
                format: Self::get_string(value, &["menu", "format"], &fallback.menu.format),
                help: Self::get_string(value, &["menu", "help"], &fallback.menu.help),
                document: Self::get_string(value, &["menu", "document"], &fallback.menu.document),
                bookmarks: Self::get_string(
                    value,
                    &["menu", "bookmarks"],
                    &fallback.menu.bookmarks,
                ),
                new: Self::get_string(value, &["menu", "new"], &fallback.menu.new),
                open: Self::get_string(value, &["menu", "open"], &fallback.menu.open),
                save: Self::get_string(value, &["menu", "save"], &fallback.menu.save),
                save_as: Self::get_string(value, &["menu", "save_as"], &fallback.menu.save_as),
                export: Self::get_string(value, &["menu", "export"], &fallback.menu.export),
                export_pdf: Self::get_string(
                    value,
                    &["menu", "export_pdf"],
                    &fallback.menu.export_pdf,
                ),
                settings: Self::get_string(value, &["menu", "settings"], &fallback.menu.settings),
                preferences: Self::get_string(
                    value,
                    &["menu", "preferences"],
                    &fallback.menu.preferences,
                ),
                quit: Self::get_string(value, &["menu", "quit"], &fallback.menu.quit),
                recent: Self::get_string(value, &["menu", "recent"], &fallback.menu.recent),
                recent_files: Self::get_string(
                    value,
                    &["menu", "recent_files"],
                    &fallback.menu.recent_files,
                ),
                no_recent: Self::get_string(
                    value,
                    &["menu", "no_recent"],
                    &fallback.menu.no_recent,
                ),
                clear_recent: Self::get_string(
                    value,
                    &["menu", "clear_recent"],
                    &fallback.menu.clear_recent,
                ),
                bold: Self::get_string(value, &["menu", "bold"], &fallback.menu.bold),
                italic: Self::get_string(value, &["menu", "italic"], &fallback.menu.italic),
                code: Self::get_string(value, &["menu", "code"], &fallback.menu.code),
                html_preview: Self::get_string(
                    value,
                    &["menu", "html_preview"],
                    &fallback.menu.html_preview,
                ),
                code_view: Self::get_string(
                    value,
                    &["menu", "code_view"],
                    &fallback.menu.code_view,
                ),
                link: Self::get_string(value, &["menu", "link"], &fallback.menu.link),
                image: Self::get_string(value, &["menu", "image"], &fallback.menu.image),
                undo: Self::get_string(value, &["menu", "undo"], &fallback.menu.undo),
                redo: Self::get_string(value, &["menu", "redo"], &fallback.menu.redo),
                cut: Self::get_string(value, &["menu", "cut"], &fallback.menu.cut),
                copy: Self::get_string(value, &["menu", "copy"], &fallback.menu.copy),
                paste: Self::get_string(value, &["menu", "paste"], &fallback.menu.paste),
                search_replace: Self::get_string(
                    value,
                    &["menu", "search_replace"],
                    &fallback.menu.search_replace,
                ),
                document_builder: Self::get_string(
                    value,
                    &["menu", "document_builder"],
                    &fallback.menu.document_builder,
                ),
                document_splitter: Self::get_string(
                    value,
                    &["menu", "document_splitter"],
                    &fallback.menu.document_splitter,
                ),
                no_bookmarks: Self::get_string(
                    value,
                    &["menu", "no_bookmarks"],
                    &fallback.menu.no_bookmarks,
                ),
                about: Self::get_string(value, &["menu", "about"], &fallback.menu.about),
            },
            toolbar: ToolbarTranslations {
                headings: Self::get_string(
                    value,
                    &["toolbar", "headings"],
                    &fallback.toolbar.headings,
                ),
                insert: Self::get_string(value, &["toolbar", "insert"], &fallback.toolbar.insert),
                bold: Self::get_string(value, &["toolbar", "bold"], &fallback.toolbar.bold),
                italic: Self::get_string(value, &["toolbar", "italic"], &fallback.toolbar.italic),
                code: Self::get_string(value, &["toolbar", "code"], &fallback.toolbar.code),
                strikethrough: Self::get_string(
                    value,
                    &["toolbar", "strikethrough"],
                    &fallback.toolbar.strikethrough,
                ),
                bullet_list: Self::get_string(
                    value,
                    &["toolbar", "bullet_list"],
                    &fallback.toolbar.bullet_list,
                ),
                numbered_list: Self::get_string(
                    value,
                    &["toolbar", "numbered_list"],
                    &fallback.toolbar.numbered_list,
                ),
                h1: Self::get_string(value, &["toolbar", "h1"], &fallback.toolbar.h1),
                h2: Self::get_string(value, &["toolbar", "h2"], &fallback.toolbar.h2),
                h3: Self::get_string(value, &["toolbar", "h3"], &fallback.toolbar.h3),
                h4: Self::get_string(value, &["toolbar", "h4"], &fallback.toolbar.h4),
                h5: Self::get_string(value, &["toolbar", "h5"], &fallback.toolbar.h5),
                h6: Self::get_string(value, &["toolbar", "h6"], &fallback.toolbar.h6),
            },
            footer: FooterTranslations {
                row: Self::get_string(value, &["footer", "row"], &fallback.footer.row),
                column: Self::get_string(value, &["footer", "column"], &fallback.footer.column),
                words: Self::get_string(value, &["footer", "words"], &fallback.footer.words),
                characters: Self::get_string(
                    value,
                    &["footer", "characters"],
                    &fallback.footer.characters,
                ),
                ins: Self::get_string(value, &["footer", "ins"], &fallback.footer.ins),
                ovr: Self::get_string(value, &["footer", "ovr"], &fallback.footer.ovr),
                encoding_utf8: Self::get_string(
                    value,
                    &["footer", "encoding_utf8"],
                    &fallback.footer.encoding_utf8,
                ),
            },
            dialog: DialogTranslations {
                open_file: Self::get_string(
                    value,
                    &["dialog", "open_file"],
                    &fallback.dialog.open_file,
                ),
                save_file: Self::get_string(
                    value,
                    &["dialog", "save_file"],
                    &fallback.dialog.save_file,
                ),
                unsaved_changes: Self::get_string(
                    value,
                    &["dialog", "unsaved_changes"],
                    &fallback.dialog.unsaved_changes,
                ),
                unsaved_message: Self::get_string(
                    value,
                    &["dialog", "unsaved_message"],
                    &fallback.dialog.unsaved_message,
                ),
                open_button: Self::get_string(
                    value,
                    &["dialog", "open_button"],
                    &fallback.dialog.open_button,
                ),
                save_button: Self::get_string(
                    value,
                    &["dialog", "save_button"],
                    &fallback.dialog.save_button,
                ),
                dont_save_button: Self::get_string(
                    value,
                    &["dialog", "dont_save_button"],
                    &fallback.dialog.dont_save_button,
                ),
                cancel_button: Self::get_string(
                    value,
                    &["dialog", "cancel_button"],
                    &fallback.dialog.cancel_button,
                ),
                about_title: Self::get_string(
                    value,
                    &["dialog", "about_title"],
                    &fallback.dialog.about_title,
                ),
                preferences_title: Self::get_string(
                    value,
                    &["dialog", "preferences_title"],
                    &fallback.dialog.preferences_title,
                ),
                save_changes_title: Self::get_string(
                    value,
                    &["dialog", "save_changes_title"],
                    &fallback.dialog.save_changes_title,
                ),
                save_changes_prompt: Self::get_string(
                    value,
                    &["dialog", "save_changes_prompt"],
                    &fallback.dialog.save_changes_prompt,
                ),
                save_changes_action_opening: Self::get_string(
                    value,
                    &["dialog", "save_changes_action_opening"],
                    &fallback.dialog.save_changes_action_opening,
                ),
                save_changes_action_new_document: Self::get_string(
                    value,
                    &["dialog", "save_changes_action_new_document"],
                    &fallback.dialog.save_changes_action_new_document,
                ),
                save_changes_action_quitting: Self::get_string(
                    value,
                    &["dialog", "save_changes_action_quitting"],
                    &fallback.dialog.save_changes_action_quitting,
                ),
                save_changes_prefix: Self::get_string(
                    value,
                    &["dialog", "save_changes_prefix"],
                    &fallback.dialog.save_changes_prefix,
                ),
                save_changes_secondary: Self::get_string(
                    value,
                    &["dialog", "save_changes_secondary"],
                    &fallback.dialog.save_changes_secondary,
                ),
                save_without_saving: Self::get_string(
                    value,
                    &["dialog", "save_without_saving"],
                    &fallback.dialog.save_without_saving,
                ),
                save_as_button: Self::get_string(
                    value,
                    &["dialog", "save_as_button"],
                    &fallback.dialog.save_as_button,
                ),
                discard_tooltip: Self::get_string(
                    value,
                    &["dialog", "discard_tooltip"],
                    &fallback.dialog.discard_tooltip,
                ),
                cancel_tooltip: Self::get_string(
                    value,
                    &["dialog", "cancel_tooltip"],
                    &fallback.dialog.cancel_tooltip,
                ),
                save_tooltip: Self::get_string(
                    value,
                    &["dialog", "save_tooltip"],
                    &fallback.dialog.save_tooltip,
                ),
                close_tooltip: Self::get_string(
                    value,
                    &["dialog", "close_tooltip"],
                    &fallback.dialog.close_tooltip,
                ),
                open_markdown_title: Self::get_string(
                    value,
                    &["dialog", "open_markdown_title"],
                    &fallback.dialog.open_markdown_title,
                ),
                save_markdown_title: Self::get_string(
                    value,
                    &["dialog", "save_markdown_title"],
                    &fallback.dialog.save_markdown_title,
                ),
                filter_markdown: Self::get_string(
                    value,
                    &["dialog", "filter_markdown"],
                    &fallback.dialog.filter_markdown,
                ),
                filter_all: Self::get_string(
                    value,
                    &["dialog", "filter_all"],
                    &fallback.dialog.filter_all,
                ),
                overwrite_title: Self::get_string(
                    value,
                    &["dialog", "overwrite_title"],
                    &fallback.dialog.overwrite_title,
                ),
                overwrite_secondary: Self::get_string(
                    value,
                    &["dialog", "overwrite_secondary"],
                    &fallback.dialog.overwrite_secondary,
                ),
                overwrite_replace: Self::get_string(
                    value,
                    &["dialog", "overwrite_replace"],
                    &fallback.dialog.overwrite_replace,
                ),
                overwrite_cancel: Self::get_string(
                    value,
                    &["dialog", "overwrite_cancel"],
                    &fallback.dialog.overwrite_cancel,
                ),
                error_title_prefix: Self::get_string(
                    value,
                    &["dialog", "error_title_prefix"],
                    &fallback.dialog.error_title_prefix,
                ),
                error_message_prefix: Self::get_string(
                    value,
                    &["dialog", "error_message_prefix"],
                    &fallback.dialog.error_message_prefix,
                ),
                info_title_file_saved: Self::get_string(
                    value,
                    &["dialog", "info_title_file_saved"],
                    &fallback.dialog.info_title_file_saved,
                ),
                info_message_file_saved: Self::get_string(
                    value,
                    &["dialog", "info_message_file_saved"],
                    &fallback.dialog.info_message_file_saved,
                ),
                about_app_name: Self::get_string(
                    value,
                    &["dialog", "about_app_name"],
                    &fallback.dialog.about_app_name,
                ),
                about_tagline: Self::get_string(
                    value,
                    &["dialog", "about_tagline"],
                    &fallback.dialog.about_tagline,
                ),
                about_version: Self::get_string(
                    value,
                    &["dialog", "about_version"],
                    &fallback.dialog.about_version,
                ),
                about_description: Self::get_string(
                    value,
                    &["dialog", "about_description"],
                    &fallback.dialog.about_description,
                ),
                about_resources_title: Self::get_string(
                    value,
                    &["dialog", "about_resources_title"],
                    &fallback.dialog.about_resources_title,
                ),
                about_link_github: Self::get_string(
                    value,
                    &["dialog", "about_link_github"],
                    &fallback.dialog.about_link_github,
                ),
                about_link_issues: Self::get_string(
                    value,
                    &["dialog", "about_link_issues"],
                    &fallback.dialog.about_link_issues,
                ),
                about_link_discuss: Self::get_string(
                    value,
                    &["dialog", "about_link_discuss"],
                    &fallback.dialog.about_link_discuss,
                ),
                about_link_changelog: Self::get_string(
                    value,
                    &["dialog", "about_link_changelog"],
                    &fallback.dialog.about_link_changelog,
                ),
                about_link_website: Self::get_string(
                    value,
                    &["dialog", "about_link_website"],
                    &fallback.dialog.about_link_website,
                ),
                about_license_text: Self::get_string(
                    value,
                    &["dialog", "about_license_text"],
                    &fallback.dialog.about_license_text,
                ),
                about_copyright: Self::get_string(
                    value,
                    &["dialog", "about_copyright"],
                    &fallback.dialog.about_copyright,
                ),
                about_close_button: Self::get_string(
                    value,
                    &["dialog", "about_close_button"],
                    &fallback.dialog.about_close_button,
                ),
            },
            settings: SettingsTranslations {
                title: Self::get_string(value, &["settings", "title"], &fallback.settings.title),
                close: Self::get_string(value, &["settings", "close"], &fallback.settings.close),
                tabs: SettingsTabsTranslations {
                    editor: Self::get_string(
                        value,
                        &["settings", "tabs", "editor"],
                        &fallback.settings.tabs.editor,
                    ),
                    layout: Self::get_string(
                        value,
                        &["settings", "tabs", "layout"],
                        &fallback.settings.tabs.layout,
                    ),
                    appearance: Self::get_string(
                        value,
                        &["settings", "tabs", "appearance"],
                        &fallback.settings.tabs.appearance,
                    ),
                    language: Self::get_string(
                        value,
                        &["settings", "tabs", "language"],
                        &fallback.settings.tabs.language,
                    ),
                    markdown: Self::get_string(
                        value,
                        &["settings", "tabs", "markdown"],
                        &fallback.settings.tabs.markdown,
                    ),
                    advanced: Self::get_string(
                        value,
                        &["settings", "tabs", "advanced"],
                        &fallback.settings.tabs.advanced,
                    ),
                    debug: Self::get_string(
                        value,
                        &["settings", "tabs", "debug"],
                        &fallback.settings.tabs.debug,
                    ),
                },
                language: SettingsLanguageTranslations {
                    label: Self::get_string(
                        value,
                        &["settings", "language", "label"],
                        &fallback.settings.language.label,
                    ),
                    description: Self::get_string(
                        value,
                        &["settings", "language", "description"],
                        &fallback.settings.language.description,
                    ),
                    system_default: Self::get_string(
                        value,
                        &["settings", "language", "system_default"],
                        &fallback.settings.language.system_default,
                    ),
                },
                editor: SettingsEditorTranslations {
                    font_label: Self::get_string(
                        value,
                        &["settings", "editor", "font_label"],
                        &fallback.settings.editor.font_label,
                    ),
                    font_description: Self::get_string(
                        value,
                        &["settings", "editor", "font_description"],
                        &fallback.settings.editor.font_description,
                    ),
                    font_size_label: Self::get_string(
                        value,
                        &["settings", "editor", "font_size_label"],
                        &fallback.settings.editor.font_size_label,
                    ),
                    font_size_description: Self::get_string(
                        value,
                        &["settings", "editor", "font_size_description"],
                        &fallback.settings.editor.font_size_description,
                    ),
                    line_height_label: Self::get_string(
                        value,
                        &["settings", "editor", "line_height_label"],
                        &fallback.settings.editor.line_height_label,
                    ),
                    line_height_description: Self::get_string(
                        value,
                        &["settings", "editor", "line_height_description"],
                        &fallback.settings.editor.line_height_description,
                    ),
                    line_wrapping_label: Self::get_string(
                        value,
                        &["settings", "editor", "line_wrapping_label"],
                        &fallback.settings.editor.line_wrapping_label,
                    ),
                    line_wrapping_description: Self::get_string(
                        value,
                        &["settings", "editor", "line_wrapping_description"],
                        &fallback.settings.editor.line_wrapping_description,
                    ),
                    auto_pairing_label: Self::get_string(
                        value,
                        &["settings", "editor", "auto_pairing_label"],
                        &fallback.settings.editor.auto_pairing_label,
                    ),
                    auto_pairing_description: Self::get_string(
                        value,
                        &["settings", "editor", "auto_pairing_description"],
                        &fallback.settings.editor.auto_pairing_description,
                    ),
                    show_invisibles_label: Self::get_string(
                        value,
                        &["settings", "editor", "show_invisibles_label"],
                        &fallback.settings.editor.show_invisibles_label,
                    ),
                    show_invisibles_description: Self::get_string(
                        value,
                        &["settings", "editor", "show_invisibles_description"],
                        &fallback.settings.editor.show_invisibles_description,
                    ),
                    tabs_to_spaces_label: Self::get_string(
                        value,
                        &["settings", "editor", "tabs_to_spaces_label"],
                        &fallback.settings.editor.tabs_to_spaces_label,
                    ),
                    tabs_to_spaces_description: Self::get_string(
                        value,
                        &["settings", "editor", "tabs_to_spaces_description"],
                        &fallback.settings.editor.tabs_to_spaces_description,
                    ),
                    syntax_colors_label: Self::get_string(
                        value,
                        &["settings", "editor", "syntax_colors_label"],
                        &fallback.settings.editor.syntax_colors_label,
                    ),
                    syntax_colors_description: Self::get_string(
                        value,
                        &["settings", "editor", "syntax_colors_description"],
                        &fallback.settings.editor.syntax_colors_description,
                    ),
                    linting_label: Self::get_string(
                        value,
                        &["settings", "editor", "linting_label"],
                        &fallback.settings.editor.linting_label,
                    ),
                    linting_description: Self::get_string(
                        value,
                        &["settings", "editor", "linting_description"],
                        &fallback.settings.editor.linting_description,
                    ),
                },
                appearance: SettingsAppearanceTranslations {
                    preview_theme_label: Self::get_string(
                        value,
                        &["settings", "appearance", "preview_theme_label"],
                        &fallback.settings.appearance.preview_theme_label,
                    ),
                    preview_theme_description: Self::get_string(
                        value,
                        &["settings", "appearance", "preview_theme_description"],
                        &fallback.settings.appearance.preview_theme_description,
                    ),
                    color_mode_label: Self::get_string(
                        value,
                        &["settings", "appearance", "color_mode_label"],
                        &fallback.settings.appearance.color_mode_label,
                    ),
                    color_mode_description: Self::get_string(
                        value,
                        &["settings", "appearance", "color_mode_description"],
                        &fallback.settings.appearance.color_mode_description,
                    ),
                    color_mode_light: Self::get_string(
                        value,
                        &["settings", "appearance", "color_mode_light"],
                        &fallback.settings.appearance.color_mode_light,
                    ),
                    color_mode_dark: Self::get_string(
                        value,
                        &["settings", "appearance", "color_mode_dark"],
                        &fallback.settings.appearance.color_mode_dark,
                    ),
                    custom_css_label: Self::get_string(
                        value,
                        &["settings", "appearance", "custom_css_label"],
                        &fallback.settings.appearance.custom_css_label,
                    ),
                    custom_css_description: Self::get_string(
                        value,
                        &["settings", "appearance", "custom_css_description"],
                        &fallback.settings.appearance.custom_css_description,
                    ),
                    custom_css_button: Self::get_string(
                        value,
                        &["settings", "appearance", "custom_css_button"],
                        &fallback.settings.appearance.custom_css_button,
                    ),
                    ui_font_label: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_label"],
                        &fallback.settings.appearance.ui_font_label,
                    ),
                    ui_font_description: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_description"],
                        &fallback.settings.appearance.ui_font_description,
                    ),
                    ui_font_size_label: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_size_label"],
                        &fallback.settings.appearance.ui_font_size_label,
                    ),
                    ui_font_size_description: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_size_description"],
                        &fallback.settings.appearance.ui_font_size_description,
                    ),
                    ui_font_system_default: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_system_default"],
                        &fallback.settings.appearance.ui_font_system_default,
                    ),
                    ui_font_sans: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_sans"],
                        &fallback.settings.appearance.ui_font_sans,
                    ),
                    ui_font_serif: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_serif"],
                        &fallback.settings.appearance.ui_font_serif,
                    ),
                    ui_font_monospace: Self::get_string(
                        value,
                        &["settings", "appearance", "ui_font_monospace"],
                        &fallback.settings.appearance.ui_font_monospace,
                    ),
                },
                layout: SettingsLayoutTranslations {
                    view_mode_label: Self::get_string(
                        value,
                        &["settings", "layout", "view_mode_label"],
                        &fallback.settings.layout.view_mode_label,
                    ),
                    view_mode_description: Self::get_string(
                        value,
                        &["settings", "layout", "view_mode_description"],
                        &fallback.settings.layout.view_mode_description,
                    ),
                    view_mode_html: Self::get_string(
                        value,
                        &["settings", "layout", "view_mode_html"],
                        &fallback.settings.layout.view_mode_html,
                    ),
                    view_mode_source: Self::get_string(
                        value,
                        &["settings", "layout", "view_mode_source"],
                        &fallback.settings.layout.view_mode_source,
                    ),
                    sync_scrolling_label: Self::get_string(
                        value,
                        &["settings", "layout", "sync_scrolling_label"],
                        &fallback.settings.layout.sync_scrolling_label,
                    ),
                    sync_scrolling_description: Self::get_string(
                        value,
                        &["settings", "layout", "sync_scrolling_description"],
                        &fallback.settings.layout.sync_scrolling_description,
                    ),
                    split_label: Self::get_string(
                        value,
                        &["settings", "layout", "split_label"],
                        &fallback.settings.layout.split_label,
                    ),
                    split_description: Self::get_string(
                        value,
                        &["settings", "layout", "split_description"],
                        &fallback.settings.layout.split_description,
                    ),
                    line_numbers_label: Self::get_string(
                        value,
                        &["settings", "layout", "line_numbers_label"],
                        &fallback.settings.layout.line_numbers_label,
                    ),
                    line_numbers_description: Self::get_string(
                        value,
                        &["settings", "layout", "line_numbers_description"],
                        &fallback.settings.layout.line_numbers_description,
                    ),
                    text_direction_label: Self::get_string(
                        value,
                        &["settings", "layout", "text_direction_label"],
                        &fallback.settings.layout.text_direction_label,
                    ),
                    text_direction_description: Self::get_string(
                        value,
                        &["settings", "layout", "text_direction_description"],
                        &fallback.settings.layout.text_direction_description,
                    ),
                    text_direction_ltr: Self::get_string(
                        value,
                        &["settings", "layout", "text_direction_ltr"],
                        &fallback.settings.layout.text_direction_ltr,
                    ),
                    text_direction_rtl: Self::get_string(
                        value,
                        &["settings", "layout", "text_direction_rtl"],
                        &fallback.settings.layout.text_direction_rtl,
                    ),
                },
                markdown: SettingsMarkdownTranslations {
                    toc_label: Self::get_string(
                        value,
                        &["settings", "markdown", "toc_label"],
                        &fallback.settings.markdown.toc_label,
                    ),
                    toc_description: Self::get_string(
                        value,
                        &["settings", "markdown", "toc_description"],
                        &fallback.settings.markdown.toc_description,
                    ),
                    metadata_label: Self::get_string(
                        value,
                        &["settings", "markdown", "metadata_label"],
                        &fallback.settings.markdown.metadata_label,
                    ),
                    metadata_description: Self::get_string(
                        value,
                        &["settings", "markdown", "metadata_description"],
                        &fallback.settings.markdown.metadata_description,
                    ),
                },
                advanced: SettingsAdvancedTranslations {
                    telemetry_label: Self::get_string(
                        value,
                        &["settings", "advanced", "telemetry_label"],
                        &fallback.settings.advanced.telemetry_label,
                    ),
                    telemetry_description: Self::get_string(
                        value,
                        &["settings", "advanced", "telemetry_description"],
                        &fallback.settings.advanced.telemetry_description,
                    ),
                    log_to_file_label: Self::get_string(
                        value,
                        &["settings", "advanced", "log_to_file_label"],
                        &fallback.settings.advanced.log_to_file_label,
                    ),
                    log_to_file_description: Self::get_string(
                        value,
                        &["settings", "advanced", "log_to_file_description"],
                        &fallback.settings.advanced.log_to_file_description,
                    ),
                    my_data_label: Self::get_string(
                        value,
                        &["settings", "advanced", "my_data_label"],
                        &fallback.settings.advanced.my_data_label,
                    ),
                    my_data_description: Self::get_string(
                        value,
                        &["settings", "advanced", "my_data_description"],
                        &fallback.settings.advanced.my_data_description,
                    ),
                    my_data_button: Self::get_string(
                        value,
                        &["settings", "advanced", "my_data_button"],
                        &fallback.settings.advanced.my_data_button,
                    ),
                },
                debug: SettingsDebugTranslations {
                    debug_label: Self::get_string(
                        value,
                        &["settings", "debug", "debug_label"],
                        &fallback.settings.debug.debug_label,
                    ),
                    debug_description: Self::get_string(
                        value,
                        &["settings", "debug", "debug_description"],
                        &fallback.settings.debug.debug_description,
                    ),
                    debug_checkbox: Self::get_string(
                        value,
                        &["settings", "debug", "debug_checkbox"],
                        &fallback.settings.debug.debug_checkbox,
                    ),
                    umami_test_label: Self::get_string(
                        value,
                        &["settings", "debug", "umami_test_label"],
                        &fallback.settings.debug.umami_test_label,
                    ),
                    umami_test_description: Self::get_string(
                        value,
                        &["settings", "debug", "umami_test_description"],
                        &fallback.settings.debug.umami_test_description,
                    ),
                    umami_test_button: Self::get_string(
                        value,
                        &["settings", "debug", "umami_test_button"],
                        &fallback.settings.debug.umami_test_button,
                    ),
                    welcome_label: Self::get_string(
                        value,
                        &["settings", "debug", "welcome_label"],
                        &fallback.settings.debug.welcome_label,
                    ),
                    welcome_description: Self::get_string(
                        value,
                        &["settings", "debug", "welcome_description"],
                        &fallback.settings.debug.welcome_description,
                    ),
                    welcome_button: Self::get_string(
                        value,
                        &["settings", "debug", "welcome_button"],
                        &fallback.settings.debug.welcome_button,
                    ),
                    log_label: Self::get_string(
                        value,
                        &["settings", "debug", "log_label"],
                        &fallback.settings.debug.log_label,
                    ),
                    log_description: Self::get_string(
                        value,
                        &["settings", "debug", "log_description"],
                        &fallback.settings.debug.log_description,
                    ),
                    log_checkbox: Self::get_string(
                        value,
                        &["settings", "debug", "log_checkbox"],
                        &fallback.settings.debug.log_checkbox,
                    ),
                    log_paths_template: Self::get_string(
                        value,
                        &["settings", "debug", "log_paths_template"],
                        &fallback.settings.debug.log_paths_template,
                    ),
                    log_size_template: Self::get_string(
                        value,
                        &["settings", "debug", "log_size_template"],
                        &fallback.settings.debug.log_size_template,
                    ),
                    open_logs_button: Self::get_string(
                        value,
                        &["settings", "debug", "open_logs_button"],
                        &fallback.settings.debug.open_logs_button,
                    ),
                    delete_logs_button: Self::get_string(
                        value,
                        &["settings", "debug", "delete_logs_button"],
                        &fallback.settings.debug.delete_logs_button,
                    ),
                    refresh_button: Self::get_string(
                        value,
                        &["settings", "debug", "refresh_button"],
                        &fallback.settings.debug.refresh_button,
                    ),
                    delete_logs_confirm: Self::get_string(
                        value,
                        &["settings", "debug", "delete_logs_confirm"],
                        &fallback.settings.debug.delete_logs_confirm,
                    ),
                    log_enable_failed_title: Self::get_string(
                        value,
                        &["settings", "debug", "log_enable_failed_title"],
                        &fallback.settings.debug.log_enable_failed_title,
                    ),
                    log_enable_failed_message: Self::get_string(
                        value,
                        &["settings", "debug", "log_enable_failed_message"],
                        &fallback.settings.debug.log_enable_failed_message,
                    ),
                },
            },
            welcome: WelcomeTranslations {
                window_title: Self::get_string(
                    value,
                    &["welcome", "window_title"],
                    &fallback.welcome.window_title,
                ),
                subtitle: Self::get_string(
                    value,
                    &["welcome", "subtitle"],
                    &fallback.welcome.subtitle,
                ),
                key_features_title: Self::get_string(
                    value,
                    &["welcome", "key_features_title"],
                    &fallback.welcome.key_features_title,
                ),

                page_info: Self::get_string(
                    value,
                    &["welcome", "page_info"],
                    &fallback.welcome.page_info,
                ),
                page_language: Self::get_string(
                    value,
                    &["welcome", "page_language"],
                    &fallback.welcome.page_language,
                ),
                page_telemetry: Self::get_string(
                    value,
                    &["welcome", "page_telemetry"],
                    &fallback.welcome.page_telemetry,
                ),

                language_header: Self::get_string(
                    value,
                    &["welcome", "language_header"],
                    &fallback.welcome.language_header,
                ),
                language_description: Self::get_string(
                    value,
                    &["welcome", "language_description"],
                    &fallback.welcome.language_description,
                ),

                telemetry_header: Self::get_string(
                    value,
                    &["welcome", "telemetry_header"],
                    &fallback.welcome.telemetry_header,
                ),
                telemetry_intro: Self::get_string(
                    value,
                    &["welcome", "telemetry_intro"],
                    &fallback.welcome.telemetry_intro,
                ),
                telemetry_checkbox_label: Self::get_string(
                    value,
                    &["welcome", "telemetry_checkbox_label"],
                    &fallback.welcome.telemetry_checkbox_label,
                ),
                telemetry_privacy_details: Self::get_string(
                    value,
                    &["welcome", "telemetry_privacy_details"],
                    &fallback.welcome.telemetry_privacy_details,
                ),
                telemetry_not_implemented: Self::get_string(
                    value,
                    &["welcome", "telemetry_not_implemented"],
                    &fallback.welcome.telemetry_not_implemented,
                ),

                back_button: Self::get_string(
                    value,
                    &["welcome", "back_button"],
                    &fallback.welcome.back_button,
                ),
                next_button: Self::get_string(
                    value,
                    &["welcome", "next_button"],
                    &fallback.welcome.next_button,
                ),
                finish_button: Self::get_string(
                    value,
                    &["welcome", "finish_button"],
                    &fallback.welcome.finish_button,
                ),

                feature_live_preview_title: Self::get_string(
                    value,
                    &["welcome", "feature_live_preview_title"],
                    &fallback.welcome.feature_live_preview_title,
                ),
                feature_live_preview_description: Self::get_string(
                    value,
                    &["welcome", "feature_live_preview_description"],
                    &fallback.welcome.feature_live_preview_description,
                ),
                feature_themes_title: Self::get_string(
                    value,
                    &["welcome", "feature_themes_title"],
                    &fallback.welcome.feature_themes_title,
                ),
                feature_themes_description: Self::get_string(
                    value,
                    &["welcome", "feature_themes_description"],
                    &fallback.welcome.feature_themes_description,
                ),
                feature_fast_title: Self::get_string(
                    value,
                    &["welcome", "feature_fast_title"],
                    &fallback.welcome.feature_fast_title,
                ),
                feature_fast_description: Self::get_string(
                    value,
                    &["welcome", "feature_fast_description"],
                    &fallback.welcome.feature_fast_description,
                ),
                feature_privacy_title: Self::get_string(
                    value,
                    &["welcome", "feature_privacy_title"],
                    &fallback.welcome.feature_privacy_title,
                ),
                feature_privacy_description: Self::get_string(
                    value,
                    &["welcome", "feature_privacy_description"],
                    &fallback.welcome.feature_privacy_description,
                ),
                feature_markdown_title: Self::get_string(
                    value,
                    &["welcome", "feature_markdown_title"],
                    &fallback.welcome.feature_markdown_title,
                ),
                feature_markdown_description: Self::get_string(
                    value,
                    &["welcome", "feature_markdown_description"],
                    &fallback.welcome.feature_markdown_description,
                ),
            },
            titlebar: TitlebarTranslations {
                app_tooltip: Self::get_string(
                    value,
                    &["titlebar", "app_tooltip"],
                    &fallback.titlebar.app_tooltip,
                ),
                layout_editor_only: Self::get_string(
                    value,
                    &["titlebar", "layout_editor_only"],
                    &fallback.titlebar.layout_editor_only,
                ),
                layout_view_only: Self::get_string(
                    value,
                    &["titlebar", "layout_view_only"],
                    &fallback.titlebar.layout_view_only,
                ),
                layout_detach_view: Self::get_string(
                    value,
                    &["titlebar", "layout_detach_view"],
                    &fallback.titlebar.layout_detach_view,
                ),
                layout_restore_split: Self::get_string(
                    value,
                    &["titlebar", "layout_restore_split"],
                    &fallback.titlebar.layout_restore_split,
                ),
                window_minimize: Self::get_string(
                    value,
                    &["titlebar", "window_minimize"],
                    &fallback.titlebar.window_minimize,
                ),
                window_maximize_restore: Self::get_string(
                    value,
                    &["titlebar", "window_maximize_restore"],
                    &fallback.titlebar.window_maximize_restore,
                ),
                window_close: Self::get_string(
                    value,
                    &["titlebar", "window_close"],
                    &fallback.titlebar.window_close,
                ),
            },
            messages: MessagesTranslations {
                file_saved: Self::get_string(
                    value,
                    &["messages", "file_saved"],
                    &fallback.messages.file_saved,
                ),
                file_opened: Self::get_string(
                    value,
                    &["messages", "file_opened"],
                    &fallback.messages.file_opened,
                ),
                export_complete: Self::get_string(
                    value,
                    &["messages", "export_complete"],
                    &fallback.messages.export_complete,
                ),
                error_opening_file: Self::get_string(
                    value,
                    &["messages", "error_opening_file"],
                    &fallback.messages.error_opening_file,
                ),
                error_saving_file: Self::get_string(
                    value,
                    &["messages", "error_saving_file"],
                    &fallback.messages.error_saving_file,
                ),
                untitled_document: Self::get_string(
                    value,
                    &["messages", "untitled_document"],
                    &fallback.messages.untitled_document,
                ),
            },
            search: SearchTranslations {
                title: Self::get_string(value, &["search", "title"], &fallback.search.title),
                close_tooltip: Self::get_string(
                    value,
                    &["search", "close_tooltip"],
                    &fallback.search.close_tooltip,
                ),
                find_label: Self::get_string(
                    value,
                    &["search", "find_label"],
                    &fallback.search.find_label,
                ),
                replace_label: Self::get_string(
                    value,
                    &["search", "replace_label"],
                    &fallback.search.replace_label,
                ),
                search_placeholder: Self::get_string(
                    value,
                    &["search", "search_placeholder"],
                    &fallback.search.search_placeholder,
                ),
                replace_placeholder: Self::get_string(
                    value,
                    &["search", "replace_placeholder"],
                    &fallback.search.replace_placeholder,
                ),
                match_case: Self::get_string(
                    value,
                    &["search", "match_case"],
                    &fallback.search.match_case,
                ),
                match_whole_word: Self::get_string(
                    value,
                    &["search", "match_whole_word"],
                    &fallback.search.match_whole_word,
                ),
                match_markdown: Self::get_string(
                    value,
                    &["search", "match_markdown"],
                    &fallback.search.match_markdown,
                ),
                use_regex: Self::get_string(
                    value,
                    &["search", "use_regex"],
                    &fallback.search.use_regex,
                ),
                previous_button: Self::get_string(
                    value,
                    &["search", "previous_button"],
                    &fallback.search.previous_button,
                ),
                next_button: Self::get_string(
                    value,
                    &["search", "next_button"],
                    &fallback.search.next_button,
                ),
                replace_button: Self::get_string(
                    value,
                    &["search", "replace_button"],
                    &fallback.search.replace_button,
                ),
                replace_all_button: Self::get_string(
                    value,
                    &["search", "replace_all_button"],
                    &fallback.search.replace_all_button,
                ),
            },
        }
    }

    fn get_string(value: &toml::Value, path: &[&str], fallback: &str) -> String {
        Self::get_value(value, path)
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| fallback.to_string())
    }

    fn get_value<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
        let mut current = value;
        for key in path {
            current = current.get(*key)?;
        }
        Some(current)
    }
}

impl LocalizationProvider for SimpleLocalizationManager {
    fn load_locale(&self, locale: &str) -> Result<(), LocalizationError> {
        // Validate ISO 639-1 format (must be 2 lowercase letters)
        if locale.len() != 2 || !locale.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(LocalizationError::InvalidLocaleCode(locale.to_string()));
        }

        let locale_path = self
            .assets_path
            .join("language")
            .join(format!("{}.toml", locale));

        if !locale_path.exists() {
            return Err(LocalizationError::LocaleNotFound(locale.to_string()));
        }

        let content = fs::read_to_string(&locale_path)?;
        let value: toml::Value = toml::from_str(&content)?;
        let fallback = Self::load_default_translations();
        let translations = Self::load_translations_from_value(&value, &fallback);

        // Update state
        *self.current_locale.write().unwrap() = locale.to_string();
        *self.translations.write().unwrap() = translations;

        log::info!("Loaded locale: {}", locale);
        Ok(())
    }

    fn translations(&self) -> Translations {
        self.translations.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_localization_manager() {
        // Test that we can create manager and load English
        // Note: This test may fail if assets are not available (e.g., in some CI environments)
        let manager = match SimpleLocalizationManager::new() {
            Ok(m) => m,
            Err(LocalizationError::Io(_)) | Err(LocalizationError::LocaleNotFound(_)) => {
                // Skip test if assets are not available
                println!("Skipping: assets not available in test environment");
                return;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        };

        assert_eq!(manager.current_locale(), "en");

        let t = manager.translations();
        assert_eq!(t.menu.file, "File");
        assert_eq!(t.toolbar.bold, "Bold");
        assert_eq!(t.footer.row, "Row");
    }

    #[test]
    fn smoke_test_invalid_locale_code() {
        let manager = match SimpleLocalizationManager::new() {
            Ok(m) => m,
            Err(_) => {
                println!("Skipping: assets not available in test environment");
                return;
            }
        };

        // Test invalid locale codes
        assert!(manager.load_locale("eng").is_err()); // Too long
        assert!(manager.load_locale("e").is_err()); // Too short
        assert!(manager.load_locale("E1").is_err()); // Not all letters
        assert!(manager.load_locale("EN").is_err()); // Uppercase
    }

    #[test]
    fn smoke_test_locale_not_found() {
        let manager = match SimpleLocalizationManager::new() {
            Ok(m) => m,
            Err(_) => {
                println!("Skipping: assets not available in test environment");
                return;
            }
        };

        // Test non-existent locale (valid format but doesn't exist)
        let result = manager.load_locale("zz");
        assert!(result.is_err());
        match result {
            Err(LocalizationError::LocaleNotFound(code)) => assert_eq!(code, "zz"),
            _ => panic!("Expected LocaleNotFound error"),
        }
    }

    #[test]
    fn smoke_test_default_translations() {
        // Test that default translations can be created without file I/O
        let t = SimpleLocalizationManager::load_default_translations();
        assert_eq!(t.menu.file, "File");
        assert_eq!(t.menu.edit, "Edit");
        assert_eq!(t.toolbar.bold, "Bold");
        assert_eq!(t.footer.row, "Row");
        assert_eq!(t.dialog.save_button, "Save");
        assert_eq!(t.settings.title, "Settings");
        assert_eq!(t.messages.file_saved, "File saved successfully");
    }

    #[test]
    fn smoke_test_locale_code_validation() {
        // Test the locale code validation logic without requiring file system access
        let is_valid = |code: &str| -> bool {
            code.len() == 2 && code.chars().all(|c| c.is_ascii_lowercase())
        };

        // Valid codes
        assert!(is_valid("en"));
        assert!(is_valid("da"));
        assert!(is_valid("de"));
        assert!(is_valid("fr"));

        // Invalid codes
        assert!(!is_valid("eng")); // Too long
        assert!(!is_valid("e")); // Too short
        assert!(!is_valid("EN")); // Uppercase
        assert!(!is_valid("e1")); // Contains digit
        assert!(!is_valid("e-")); // Contains special char
    }

    #[test]
    fn smoke_test_locale_fallback_missing_keys() {
        let fallback = SimpleLocalizationManager::load_default_translations();
        let toml = r#"
[menu]
file = "Fichier"
"#;

        let value: toml::Value = toml::from_str(toml).expect("valid toml");
        let translations =
            SimpleLocalizationManager::load_translations_from_value(&value, &fallback);

        assert_eq!(translations.menu.file, "Fichier");
        assert_eq!(translations.menu.edit, fallback.menu.edit);
        assert_eq!(translations.toolbar.bold, fallback.toolbar.bold);
        assert_eq!(translations.footer.row, fallback.footer.row);
    }
}
