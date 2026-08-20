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

pub(crate) mod default_translations;

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
    pub page_appearance: String,
    pub page_telemetry: String,

    pub language_header: String,
    pub language_description: String,

    pub appearance_header: String,
    pub appearance_description: String,
    pub appearance_light: String,
    pub appearance_dark: String,

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
    pub tools: String,
    pub blocks: String,
    pub modules: String,
    pub help: String,
    pub document: String,
    pub paragraph: String,
    pub bookmarks: String,
    pub new: String,
    pub open: String,
    pub save: String,
    pub save_as: String,
    pub print: String,
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
    pub delete: String,
    pub select_all: String,
    pub search_replace: String,
    pub no_bookmarks: String,
    pub insert: String,
    pub insert_link: String,
    pub insert_image: String,
    pub insert_table: String,
    pub insert_code_block: String,
    pub insert_mermaid: String,
    pub insert_math: String,
    pub insert_footnote: String,
    pub insert_admonition: String,
    pub insert_tab_block: String,
    pub insert_slider: String,
    pub insert_horizontal_rule: String,
    pub toggle_preview: String,
    pub side_by_side: String,
    pub toggle_line_numbers: String,
    pub toggle_wrap: String,
    pub inline: String,
    pub block: String,
    pub heading_id: String,
    pub highlight: String,
    pub superscript: String,
    pub subscript: String,
    pub math_inline: String,
    pub link_reference: String,
    pub inline_footnote: String,
    pub emoji: String,
    pub checkbox: String,
    pub mention: String,
    pub text_wrap: String,
    pub line_numbers: String,
    pub sync_scrolling: String,
    pub show_raw_html: String,
    pub show_rendered_markdown: String,
    pub strikethrough: String,
    pub heading_1: String,
    pub heading_2: String,
    pub heading_3: String,
    pub heading_4: String,
    pub heading_5: String,
    pub heading_6: String,
    pub blockquote: String,
    pub fenced_code: String,
    pub horizontal_rule: String,
    pub lists: String,
    pub bulleted_list: String,
    pub numbered_list: String,
    pub task_list: String,
    pub indent_increase: String,
    pub indent_decrease: String,
    pub markdown_reference: String,
    pub walkthrough: String,
    pub keyboard_shortcuts: String,
    pub diagnostics_reference: String,
    pub about: String,
    pub bookmark_line: String,
    pub format_table: String,
    pub insert_update_toc: String,
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
    // Block-type dropdown
    pub block_type: String,
    pub block: String,
    pub paragraph: String,
    pub quote: String,
    pub heading_id: String,
    // Inline formatting popover
    pub highlight: String,
    pub inline: String,
    pub inline_code: String,
    pub inline_code_tooltip: String,
    pub superscript: String,
    pub superscript_tooltip: String,
    pub subscript: String,
    pub subscript_tooltip: String,
    pub math: String,
    pub inline_math_tooltip: String,
    // Lists button
    pub lists: String,
    // Insert (inline items) popover
    pub link: String,
    pub link_tooltip: String,
    pub link_reference: String,
    pub link_reference_tooltip: String,
    pub image: String,
    pub image_tooltip: String,
    pub footnote: String,
    pub inline_footnote_tooltip: String,
    pub emoji: String,
    pub emoji_tooltip: String,
    pub checkbox: String,
    pub checkbox_tooltip: String,
    // Horizontal rule
    pub horizontal_rule: String,
    // Block items popover
    pub blocks: String,
    pub code_block_tooltip: String,
    pub math_block_tooltip: String,
    pub block_footnote_tooltip: String,
    // Modules (container) items popover
    pub modules: String,
    pub table: String,
    pub tab_block: String,
    pub slider_deck: String,
    pub mermaid: String,
    pub admonition: String,
    // Mentions button
    pub mentions: String,
    // Emoji popover
    pub recently_used: String,
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
    pub contents: String,
    pub no_headings: String,
    pub issues_prefix: String,
    pub no_diagnostics: String,
    pub toc_tooltip: String,
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
    // --- Common dialog action buttons (used by insert dialogs) ---
    pub insert_button: String,
    pub close_button: String,
    pub add_button: String,
    pub duplicate_button: String,
    pub use_template_button: String,
    pub live_preview_label: String,
    pub source_label: String,
    // --- Per-dialog sub-translations ---
    pub admonition: AdmonitionDialogTranslations,
    pub lists: ListsDialogTranslations,
    pub tables: TablesDialogTranslations,
    pub tabs: TabsDialogTranslations,
    pub sliderdeck: SliderDeckDialogTranslations,
    pub math: MathDialogTranslations,
    pub mermaid: MermaidDialogTranslations,
    pub mention: MentionDialogTranslations,
    pub diagnostics_reference: DiagnosticsReferenceDialogTranslations,
    pub open_local_file: OpenLocalFileDialogTranslations,
    pub export: ExportDialogTranslations,
    pub export_complete: ExportCompleteDialogTranslations,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdmonitionDialogTranslations {
    pub title: String,
    pub section_label: String,
    pub text_label: String,
    pub emoji_placeholder: String,
    pub title_placeholder: String,
    pub type_note_title: String,
    pub type_note_desc: String,
    pub type_tip_title: String,
    pub type_tip_desc: String,
    pub type_important_title: String,
    pub type_important_desc: String,
    pub type_warning_title: String,
    pub type_warning_desc: String,
    pub type_caution_title: String,
    pub type_caution_desc: String,
    pub type_custom_title: String,
    pub type_custom_desc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListsDialogTranslations {
    pub title: String,
    pub type_label: String,
    pub items_label: String,
    pub type_bullet_title: String,
    pub type_bullet_desc: String,
    pub type_ordered_title: String,
    pub type_ordered_desc: String,
    pub type_unordered_title: String,
    pub type_unordered_desc: String,
    pub type_task_title: String,
    pub type_task_desc: String,
    pub type_task_nodot_title: String,
    pub type_task_nodot_desc: String,
    pub type_definition_title: String,
    pub type_definition_desc: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TablesDialogTranslations {
    pub title: String,
    pub rows_label: String,
    pub columns_label: String,
    pub alignment_title: String,
    pub include_header: String,
    pub edit_alignment: String,
    pub align_left: String,
    pub align_center: String,
    pub align_right: String,
    pub selected_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TabsDialogTranslations {
    pub title: String,
    pub empty_label: String,
    pub add_button: String,
    pub content_for: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SliderDeckDialogTranslations {
    pub title: String,
    pub timer_label: String,
    pub timer_tooltip: String,
    pub seconds_label: String,
    pub empty_label: String,
    pub add_button: String,
    pub content_for: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MathDialogTranslations {
    pub title: String,
    pub mode_label: String,
    pub inline_radio: String,
    pub block_radio: String,
    pub templates_label: String,
    pub expression_label: String,
    pub tip_text: String,
    pub status_waiting: String,
    pub status_valid: String,
    pub no_templates: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MermaidDialogTranslations {
    pub title: String,
    pub type_label: String,
    pub diagram_flowchart: String,
    pub diagram_sequence: String,
    pub diagram_pie: String,
    pub diagram_gitgraph: String,
    pub diagram_class: String,
    pub diagram_custom: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MentionDialogTranslations {
    pub title: String,
    pub mention_label: String,
    pub username_label: String,
    pub realname_label: String,
    pub realname_placeholder: String,
    pub valid_button: String,
    pub error_button: String,
    pub status_waiting: String,
    pub status_checking: String,
    pub status_invalid_value: String,
    pub status_unsupported: String,
    pub status_not_implemented: String,
    pub status_blocked: String,
    pub status_timeout: String,
    pub status_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsReferenceDialogTranslations {
    pub title: String,
    pub search_label: String,
    pub search_placeholder: String,
    pub severity_label: String,
    pub severity_all: String,
    pub severity_error: String,
    pub severity_warning: String,
    pub severity_info: String,
    pub severity_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenLocalFileDialogTranslations {
    pub discard_open: String,
    pub save_open: String,
    pub open_button: String,
    pub cancel_button: String,
    pub tooltip_discard: String,
    pub tooltip_cancel: String,
    pub tooltip_save_open: String,
    pub tooltip_open: String,
    pub unsaved_changes_message: String,
    /// Label of the only button shown when the link's target is missing.
    pub close_button: String,
    /// Headline shown when the link's target does not exist. `{filename}`.
    pub missing_target_message: String,
    /// Detail line naming the path that was looked for. `{path}`.
    pub missing_target_detail: String,
    pub tooltip_close: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportDialogTranslations {
    pub title: String,
    pub pdf_radio: String,
    pub html_radio: String,
    pub cancel_button: String,
    pub export_button: String,
    pub save_pdf_title: String,
    pub save_html_title: String,
    pub filter_pdf: String,
    pub filter_html: String,
    pub paper_size_locked_tooltip: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportCompleteDialogTranslations {
    pub close_button: String,
    pub open_folder: String,
    pub open_document: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTranslations {
    pub title: String,
    pub close: String,
    pub not_available_yet: String,
    pub tabs: SettingsTabsTranslations,
    pub language: SettingsLanguageTranslations,
    pub editor: SettingsEditorTranslations,
    pub intelligence: SettingsIntelligenceTranslations,
    pub appearance: SettingsAppearanceTranslations,
    pub layout: SettingsLayoutTranslations,
    pub advanced: SettingsAdvancedTranslations,
    pub debug: SettingsDebugTranslations,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTabsTranslations {
    pub application: String,
    pub editor: String,
    pub intelligence: String,
    pub language: String,
    pub advanced: String,
    pub debug: String,
    pub print_preview: String,
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
    pub show_invisibles_label: String,
    pub show_invisibles_description: String,
    pub tabs_to_spaces_label: String,
    pub tabs_to_spaces_description: String,
    pub syntax_colors_label: String,
    pub syntax_colors_description: String,
    pub table_auto_align_label: String,
    pub table_auto_align_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsIntelligenceTranslations {
    pub section_intelligence: String,
    pub intro_description: String,
    pub section_issues: String,
    pub diagnostics_underlines_label: String,
    pub diagnostics_underlines_description: String,
    pub section_insights: String,
    pub markdown_insights_label: String,
    pub markdown_insights_description: String,
    pub issue_insights_label: String,
    pub issue_insights_description: String,
    pub section_highlighting: String,
    pub syntax_highlighting_label: String,
    pub syntax_highlighting_description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsAppearanceTranslations {
    pub preview_theme_label: String,
    pub preview_theme_description: String,
    pub toolbar_svg_text_label: String,
    pub toolbar_svg_text_description: String,
    pub color_mode_label: String,
    pub color_mode_description: String,
    pub color_mode_light: String,
    pub color_mode_dark: String,
    /// Colour-mode option that follows the operating system.
    pub color_mode_system: String,
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
    pub toc_depth_label: String,
    pub toc_depth_description: String,
    pub text_direction_label: String,
    pub text_direction_description: String,
    pub text_direction_ltr: String,
    pub text_direction_rtl: String,
    pub page_view_label: String,
    pub page_view_description: String,
    pub page_view_paper_label: String,
    pub page_view_paper_description: String,
    pub page_view_orientation_label: String,
    pub page_view_orientation_description: String,
    pub page_view_orientation_portrait: String,
    pub page_view_orientation_landscape: String,
    pub page_view_margin_label: String,
    pub page_view_margin_description: String,
    pub page_view_page_numbers_label: String,
    pub page_view_page_numbers_description: String,
    pub page_view_columns_label: String,
    pub page_view_columns_description: String,
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
    pub my_data_body_text: String,
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
    pub change_layout: String,
    pub layout_state_editor_only: String,
    pub layout_state_view_only: String,
    pub layout_state_editor_and_view_separate: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagesTranslations {
    pub file_saved: String,
    pub file_opened: String,
    pub export_complete: String,
    pub error_opening_file: String,
    pub error_saving_file: String,
    pub untitled_document: String,
    pub about_section: String,
    pub code_view_short: String,
    pub live_preview: String,
    pub print_preview_short: String,
    pub select_local_file: String,
    pub theme_label: String,
    pub open_in_editor_prompt: String,
    pub exporting_html: String,
    pub exporting_pdf: String,
    pub generating_html: String,
    pub generating_pdf: String,
    pub html_export_complete: String,
    pub pdf_export_complete: String,
    pub pdf_export_success: String,
    pub html_export_success: String,
    pub insert_row_above: String,
    pub insert_row_below: String,
    pub insert_column_left: String,
    pub insert_column_right: String,
    pub export_phase: ExportPhaseTranslations,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportPhaseTranslations {
    pub preparing: String,
    pub loading: String,
    pub paginating: String,
    pub applying_print_css: String,
    pub writing_output: String,
    pub restoring_preview: String,
    pub done: String,
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
                    "Invalid locale code '{}' (must be ISO 639-1, e.g. 'en', optionally with an ISO 3166-1 region subtag, e.g. 'zh-CN')",
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

/// Validate a locale code against the naming scheme accepted for
/// `assets/language/*.toml` files: either a bare ISO 639-1 language code
/// (`en`, `da`) or that code plus an ISO 3166-1 alpha-2 region subtag
/// (`zh-CN`, `zh-TW`), matching the BCP 47 convention used to disambiguate
/// regional script/dialect variants for macrolanguages like Chinese.
fn is_valid_locale_code(locale: &str) -> bool {
    match locale.split_once('-') {
        None => locale.len() == 2 && locale.chars().all(|c| c.is_ascii_lowercase()),
        Some((lang, region)) => {
            lang.len() == 2
                && lang.chars().all(|c| c.is_ascii_lowercase())
                && region.len() == 2
                && region.chars().all(|c| c.is_ascii_uppercase())
        }
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
        let shared_paths = marco_shared::paths::SharedPaths::new().map_err(|e| {
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

    /// Load `locale`, cascading through fallbacks so a language switch never
    /// leaves the UI without translations:
    /// 1. Try `locale` as given (may be region-qualified, e.g. `zh-CN`).
    /// 2. If that fails and `locale` carries a region subtag, retry with
    ///    just the bare language part (e.g. `zh`) — this is what lets a
    ///    detected `zh-HK` (no matching file) still land on `zh` if that
    ///    exists.
    /// 3. If that also fails (and step 2 didn't already try it), fall back
    ///    to English.
    ///
    /// Every step is logged.
    pub fn load_locale_with_fallback(&self, locale: &str) {
        if self.load_locale(locale).is_ok() {
            return;
        }
        log::warn!("Failed to load locale '{}'. Trying fallback.", locale);

        let bare_lang = locale.split_once('-').map(|(lang, _)| lang);
        if let Some(lang) = bare_lang {
            match self.load_locale(lang) {
                Ok(()) => return,
                Err(e) => log::warn!("Failed to load fallback locale '{}': {}.", lang, e),
            }
        }

        // Skip if step 2 already tried "en" (bare_lang == Some("en")), so we
        // always make exactly one attempt at the final fallback rather than
        // sometimes zero (when `locale` was "en" itself) or two (when
        // `locale` was e.g. "en-GB").
        if bare_lang != Some("en") {
            if let Err(e) = self.load_locale("en") {
                log::error!("Failed to load fallback locale 'en': {}", e);
            }
        }
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
                            // Validate locale code (ISO 639-1, optionally with an
                            // ISO 3166-1 region subtag, e.g. `zh-CN`).
                            if is_valid_locale_code(locale) {
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
                tools: Self::get_string(value, &["menu", "tools"], &fallback.menu.tools),
                blocks: Self::get_string(value, &["menu", "blocks"], &fallback.menu.blocks),
                modules: Self::get_string(value, &["menu", "modules"], &fallback.menu.modules),
                help: Self::get_string(value, &["menu", "help"], &fallback.menu.help),
                document: Self::get_string(value, &["menu", "document"], &fallback.menu.document),
                paragraph: Self::get_string(
                    value,
                    &["menu", "paragraph"],
                    &fallback.menu.paragraph,
                ),
                bookmarks: Self::get_string(
                    value,
                    &["menu", "bookmarks"],
                    &fallback.menu.bookmarks,
                ),
                new: Self::get_string(value, &["menu", "new"], &fallback.menu.new),
                open: Self::get_string(value, &["menu", "open"], &fallback.menu.open),
                save: Self::get_string(value, &["menu", "save"], &fallback.menu.save),
                save_as: Self::get_string(value, &["menu", "save_as"], &fallback.menu.save_as),
                print: Self::get_string(value, &["menu", "print"], &fallback.menu.print),
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
                delete: Self::get_string(value, &["menu", "delete"], &fallback.menu.delete),
                select_all: Self::get_string(
                    value,
                    &["menu", "select_all"],
                    &fallback.menu.select_all,
                ),
                search_replace: Self::get_string(
                    value,
                    &["menu", "search_replace"],
                    &fallback.menu.search_replace,
                ),
                no_bookmarks: Self::get_string(
                    value,
                    &["menu", "no_bookmarks"],
                    &fallback.menu.no_bookmarks,
                ),
                insert: Self::get_string(value, &["menu", "insert"], &fallback.menu.insert),
                insert_link: Self::get_string(
                    value,
                    &["menu", "insert_link"],
                    &fallback.menu.insert_link,
                ),
                insert_image: Self::get_string(
                    value,
                    &["menu", "insert_image"],
                    &fallback.menu.insert_image,
                ),
                insert_table: Self::get_string(
                    value,
                    &["menu", "insert_table"],
                    &fallback.menu.insert_table,
                ),
                insert_code_block: Self::get_string(
                    value,
                    &["menu", "insert_code_block"],
                    &fallback.menu.insert_code_block,
                ),
                insert_mermaid: Self::get_string(
                    value,
                    &["menu", "insert_mermaid"],
                    &fallback.menu.insert_mermaid,
                ),
                insert_math: Self::get_string(
                    value,
                    &["menu", "insert_math"],
                    &fallback.menu.insert_math,
                ),
                insert_footnote: Self::get_string(
                    value,
                    &["menu", "insert_footnote"],
                    &fallback.menu.insert_footnote,
                ),
                insert_admonition: Self::get_string(
                    value,
                    &["menu", "insert_admonition"],
                    &fallback.menu.insert_admonition,
                ),
                insert_tab_block: Self::get_string(
                    value,
                    &["menu", "insert_tab_block"],
                    &fallback.menu.insert_tab_block,
                ),
                insert_slider: Self::get_string(
                    value,
                    &["menu", "insert_slider"],
                    &fallback.menu.insert_slider,
                ),
                insert_horizontal_rule: Self::get_string(
                    value,
                    &["menu", "insert_horizontal_rule"],
                    &fallback.menu.insert_horizontal_rule,
                ),
                toggle_preview: Self::get_string(
                    value,
                    &["menu", "toggle_preview"],
                    &fallback.menu.toggle_preview,
                ),
                side_by_side: Self::get_string(
                    value,
                    &["menu", "side_by_side"],
                    &fallback.menu.side_by_side,
                ),
                toggle_line_numbers: Self::get_string(
                    value,
                    &["menu", "toggle_line_numbers"],
                    &fallback.menu.toggle_line_numbers,
                ),
                toggle_wrap: Self::get_string(
                    value,
                    &["menu", "toggle_wrap"],
                    &fallback.menu.toggle_wrap,
                ),
                inline: Self::get_string(value, &["menu", "inline"], &fallback.menu.inline),
                block: Self::get_string(value, &["menu", "block"], &fallback.menu.block),
                heading_id: Self::get_string(
                    value,
                    &["menu", "heading_id"],
                    &fallback.menu.heading_id,
                ),
                highlight: Self::get_string(
                    value,
                    &["menu", "highlight"],
                    &fallback.menu.highlight,
                ),
                superscript: Self::get_string(
                    value,
                    &["menu", "superscript"],
                    &fallback.menu.superscript,
                ),
                subscript: Self::get_string(
                    value,
                    &["menu", "subscript"],
                    &fallback.menu.subscript,
                ),
                math_inline: Self::get_string(
                    value,
                    &["menu", "math_inline"],
                    &fallback.menu.math_inline,
                ),
                link_reference: Self::get_string(
                    value,
                    &["menu", "link_reference"],
                    &fallback.menu.link_reference,
                ),
                inline_footnote: Self::get_string(
                    value,
                    &["menu", "inline_footnote"],
                    &fallback.menu.inline_footnote,
                ),
                emoji: Self::get_string(value, &["menu", "emoji"], &fallback.menu.emoji),
                checkbox: Self::get_string(value, &["menu", "checkbox"], &fallback.menu.checkbox),
                mention: Self::get_string(value, &["menu", "mention"], &fallback.menu.mention),
                text_wrap: Self::get_string(
                    value,
                    &["menu", "text_wrap"],
                    &fallback.menu.text_wrap,
                ),
                line_numbers: Self::get_string(
                    value,
                    &["menu", "line_numbers"],
                    &fallback.menu.line_numbers,
                ),
                sync_scrolling: Self::get_string(
                    value,
                    &["menu", "sync_scrolling"],
                    &fallback.menu.sync_scrolling,
                ),
                show_raw_html: Self::get_string(
                    value,
                    &["menu", "show_raw_html"],
                    &fallback.menu.show_raw_html,
                ),
                show_rendered_markdown: Self::get_string(
                    value,
                    &["menu", "show_rendered_markdown"],
                    &fallback.menu.show_rendered_markdown,
                ),
                strikethrough: Self::get_string(
                    value,
                    &["menu", "strikethrough"],
                    &fallback.menu.strikethrough,
                ),
                heading_1: Self::get_string(
                    value,
                    &["menu", "heading_1"],
                    &fallback.menu.heading_1,
                ),
                heading_2: Self::get_string(
                    value,
                    &["menu", "heading_2"],
                    &fallback.menu.heading_2,
                ),
                heading_3: Self::get_string(
                    value,
                    &["menu", "heading_3"],
                    &fallback.menu.heading_3,
                ),
                heading_4: Self::get_string(
                    value,
                    &["menu", "heading_4"],
                    &fallback.menu.heading_4,
                ),
                heading_5: Self::get_string(
                    value,
                    &["menu", "heading_5"],
                    &fallback.menu.heading_5,
                ),
                heading_6: Self::get_string(
                    value,
                    &["menu", "heading_6"],
                    &fallback.menu.heading_6,
                ),
                blockquote: Self::get_string(
                    value,
                    &["menu", "blockquote"],
                    &fallback.menu.blockquote,
                ),
                fenced_code: Self::get_string(
                    value,
                    &["menu", "fenced_code"],
                    &fallback.menu.fenced_code,
                ),
                horizontal_rule: Self::get_string(
                    value,
                    &["menu", "horizontal_rule"],
                    &fallback.menu.horizontal_rule,
                ),
                lists: Self::get_string(value, &["menu", "lists"], &fallback.menu.lists),
                bulleted_list: Self::get_string(
                    value,
                    &["menu", "bulleted_list"],
                    &fallback.menu.bulleted_list,
                ),
                numbered_list: Self::get_string(
                    value,
                    &["menu", "numbered_list"],
                    &fallback.menu.numbered_list,
                ),
                task_list: Self::get_string(
                    value,
                    &["menu", "task_list"],
                    &fallback.menu.task_list,
                ),
                indent_increase: Self::get_string(
                    value,
                    &["menu", "indent_increase"],
                    &fallback.menu.indent_increase,
                ),
                indent_decrease: Self::get_string(
                    value,
                    &["menu", "indent_decrease"],
                    &fallback.menu.indent_decrease,
                ),
                markdown_reference: Self::get_string(
                    value,
                    &["menu", "markdown_reference"],
                    &fallback.menu.markdown_reference,
                ),
                walkthrough: Self::get_string(
                    value,
                    &["menu", "walkthrough"],
                    &fallback.menu.walkthrough,
                ),
                keyboard_shortcuts: Self::get_string(
                    value,
                    &["menu", "keyboard_shortcuts"],
                    &fallback.menu.keyboard_shortcuts,
                ),
                diagnostics_reference: Self::get_string(
                    value,
                    &["menu", "diagnostics_reference"],
                    &fallback.menu.diagnostics_reference,
                ),
                about: Self::get_string(value, &["menu", "about"], &fallback.menu.about),
                bookmark_line: Self::get_string(
                    value,
                    &["menu", "bookmark_line"],
                    &fallback.menu.bookmark_line,
                ),
                format_table: Self::get_string(
                    value,
                    &["menu", "format_table"],
                    &fallback.menu.format_table,
                ),
                insert_update_toc: Self::get_string(
                    value,
                    &["menu", "insert_update_toc"],
                    &fallback.menu.insert_update_toc,
                ),
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
                block_type: Self::get_string(
                    value,
                    &["toolbar", "block_type"],
                    &fallback.toolbar.block_type,
                ),
                block: Self::get_string(value, &["toolbar", "block"], &fallback.toolbar.block),
                paragraph: Self::get_string(
                    value,
                    &["toolbar", "paragraph"],
                    &fallback.toolbar.paragraph,
                ),
                quote: Self::get_string(value, &["toolbar", "quote"], &fallback.toolbar.quote),
                heading_id: Self::get_string(
                    value,
                    &["toolbar", "heading_id"],
                    &fallback.toolbar.heading_id,
                ),
                highlight: Self::get_string(
                    value,
                    &["toolbar", "highlight"],
                    &fallback.toolbar.highlight,
                ),
                inline: Self::get_string(value, &["toolbar", "inline"], &fallback.toolbar.inline),
                inline_code: Self::get_string(
                    value,
                    &["toolbar", "inline_code"],
                    &fallback.toolbar.inline_code,
                ),
                inline_code_tooltip: Self::get_string(
                    value,
                    &["toolbar", "inline_code_tooltip"],
                    &fallback.toolbar.inline_code_tooltip,
                ),
                superscript: Self::get_string(
                    value,
                    &["toolbar", "superscript"],
                    &fallback.toolbar.superscript,
                ),
                superscript_tooltip: Self::get_string(
                    value,
                    &["toolbar", "superscript_tooltip"],
                    &fallback.toolbar.superscript_tooltip,
                ),
                subscript: Self::get_string(
                    value,
                    &["toolbar", "subscript"],
                    &fallback.toolbar.subscript,
                ),
                subscript_tooltip: Self::get_string(
                    value,
                    &["toolbar", "subscript_tooltip"],
                    &fallback.toolbar.subscript_tooltip,
                ),
                math: Self::get_string(value, &["toolbar", "math"], &fallback.toolbar.math),
                inline_math_tooltip: Self::get_string(
                    value,
                    &["toolbar", "inline_math_tooltip"],
                    &fallback.toolbar.inline_math_tooltip,
                ),
                lists: Self::get_string(value, &["toolbar", "lists"], &fallback.toolbar.lists),
                link: Self::get_string(value, &["toolbar", "link"], &fallback.toolbar.link),
                link_tooltip: Self::get_string(
                    value,
                    &["toolbar", "link_tooltip"],
                    &fallback.toolbar.link_tooltip,
                ),
                link_reference: Self::get_string(
                    value,
                    &["toolbar", "link_reference"],
                    &fallback.toolbar.link_reference,
                ),
                link_reference_tooltip: Self::get_string(
                    value,
                    &["toolbar", "link_reference_tooltip"],
                    &fallback.toolbar.link_reference_tooltip,
                ),
                image: Self::get_string(value, &["toolbar", "image"], &fallback.toolbar.image),
                image_tooltip: Self::get_string(
                    value,
                    &["toolbar", "image_tooltip"],
                    &fallback.toolbar.image_tooltip,
                ),
                footnote: Self::get_string(
                    value,
                    &["toolbar", "footnote"],
                    &fallback.toolbar.footnote,
                ),
                inline_footnote_tooltip: Self::get_string(
                    value,
                    &["toolbar", "inline_footnote_tooltip"],
                    &fallback.toolbar.inline_footnote_tooltip,
                ),
                emoji: Self::get_string(value, &["toolbar", "emoji"], &fallback.toolbar.emoji),
                emoji_tooltip: Self::get_string(
                    value,
                    &["toolbar", "emoji_tooltip"],
                    &fallback.toolbar.emoji_tooltip,
                ),
                checkbox: Self::get_string(
                    value,
                    &["toolbar", "checkbox"],
                    &fallback.toolbar.checkbox,
                ),
                checkbox_tooltip: Self::get_string(
                    value,
                    &["toolbar", "checkbox_tooltip"],
                    &fallback.toolbar.checkbox_tooltip,
                ),
                horizontal_rule: Self::get_string(
                    value,
                    &["toolbar", "horizontal_rule"],
                    &fallback.toolbar.horizontal_rule,
                ),
                blocks: Self::get_string(value, &["toolbar", "blocks"], &fallback.toolbar.blocks),
                code_block_tooltip: Self::get_string(
                    value,
                    &["toolbar", "code_block_tooltip"],
                    &fallback.toolbar.code_block_tooltip,
                ),
                math_block_tooltip: Self::get_string(
                    value,
                    &["toolbar", "math_block_tooltip"],
                    &fallback.toolbar.math_block_tooltip,
                ),
                block_footnote_tooltip: Self::get_string(
                    value,
                    &["toolbar", "block_footnote_tooltip"],
                    &fallback.toolbar.block_footnote_tooltip,
                ),
                modules: Self::get_string(
                    value,
                    &["toolbar", "modules"],
                    &fallback.toolbar.modules,
                ),
                table: Self::get_string(value, &["toolbar", "table"], &fallback.toolbar.table),
                tab_block: Self::get_string(
                    value,
                    &["toolbar", "tab_block"],
                    &fallback.toolbar.tab_block,
                ),
                slider_deck: Self::get_string(
                    value,
                    &["toolbar", "slider_deck"],
                    &fallback.toolbar.slider_deck,
                ),
                mermaid: Self::get_string(
                    value,
                    &["toolbar", "mermaid"],
                    &fallback.toolbar.mermaid,
                ),
                admonition: Self::get_string(
                    value,
                    &["toolbar", "admonition"],
                    &fallback.toolbar.admonition,
                ),
                mentions: Self::get_string(
                    value,
                    &["toolbar", "mentions"],
                    &fallback.toolbar.mentions,
                ),
                recently_used: Self::get_string(
                    value,
                    &["toolbar", "recently_used"],
                    &fallback.toolbar.recently_used,
                ),
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
                contents: Self::get_string(
                    value,
                    &["footer", "contents"],
                    &fallback.footer.contents,
                ),
                no_headings: Self::get_string(
                    value,
                    &["footer", "no_headings"],
                    &fallback.footer.no_headings,
                ),
                issues_prefix: Self::get_string(
                    value,
                    &["footer", "issues_prefix"],
                    &fallback.footer.issues_prefix,
                ),
                no_diagnostics: Self::get_string(
                    value,
                    &["footer", "no_diagnostics"],
                    &fallback.footer.no_diagnostics,
                ),
                toc_tooltip: Self::get_string(
                    value,
                    &["footer", "toc_tooltip"],
                    &fallback.footer.toc_tooltip,
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
                insert_button: Self::get_string(
                    value,
                    &["dialog", "insert_button"],
                    &fallback.dialog.insert_button,
                ),
                close_button: Self::get_string(
                    value,
                    &["dialog", "close_button"],
                    &fallback.dialog.close_button,
                ),
                add_button: Self::get_string(
                    value,
                    &["dialog", "add_button"],
                    &fallback.dialog.add_button,
                ),
                duplicate_button: Self::get_string(
                    value,
                    &["dialog", "duplicate_button"],
                    &fallback.dialog.duplicate_button,
                ),
                use_template_button: Self::get_string(
                    value,
                    &["dialog", "use_template_button"],
                    &fallback.dialog.use_template_button,
                ),
                live_preview_label: Self::get_string(
                    value,
                    &["dialog", "live_preview_label"],
                    &fallback.dialog.live_preview_label,
                ),
                source_label: Self::get_string(
                    value,
                    &["dialog", "source_label"],
                    &fallback.dialog.source_label,
                ),
                admonition: AdmonitionDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "admonition", "title"],
                        &fallback.dialog.admonition.title,
                    ),
                    section_label: Self::get_string(
                        value,
                        &["dialog", "admonition", "section_label"],
                        &fallback.dialog.admonition.section_label,
                    ),
                    text_label: Self::get_string(
                        value,
                        &["dialog", "admonition", "text_label"],
                        &fallback.dialog.admonition.text_label,
                    ),
                    emoji_placeholder: Self::get_string(
                        value,
                        &["dialog", "admonition", "emoji_placeholder"],
                        &fallback.dialog.admonition.emoji_placeholder,
                    ),
                    title_placeholder: Self::get_string(
                        value,
                        &["dialog", "admonition", "title_placeholder"],
                        &fallback.dialog.admonition.title_placeholder,
                    ),
                    type_note_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_note_title"],
                        &fallback.dialog.admonition.type_note_title,
                    ),
                    type_note_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_note_desc"],
                        &fallback.dialog.admonition.type_note_desc,
                    ),
                    type_tip_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_tip_title"],
                        &fallback.dialog.admonition.type_tip_title,
                    ),
                    type_tip_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_tip_desc"],
                        &fallback.dialog.admonition.type_tip_desc,
                    ),
                    type_important_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_important_title"],
                        &fallback.dialog.admonition.type_important_title,
                    ),
                    type_important_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_important_desc"],
                        &fallback.dialog.admonition.type_important_desc,
                    ),
                    type_warning_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_warning_title"],
                        &fallback.dialog.admonition.type_warning_title,
                    ),
                    type_warning_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_warning_desc"],
                        &fallback.dialog.admonition.type_warning_desc,
                    ),
                    type_caution_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_caution_title"],
                        &fallback.dialog.admonition.type_caution_title,
                    ),
                    type_caution_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_caution_desc"],
                        &fallback.dialog.admonition.type_caution_desc,
                    ),
                    type_custom_title: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_custom_title"],
                        &fallback.dialog.admonition.type_custom_title,
                    ),
                    type_custom_desc: Self::get_string(
                        value,
                        &["dialog", "admonition", "type_custom_desc"],
                        &fallback.dialog.admonition.type_custom_desc,
                    ),
                },
                lists: ListsDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "lists", "title"],
                        &fallback.dialog.lists.title,
                    ),
                    type_label: Self::get_string(
                        value,
                        &["dialog", "lists", "type_label"],
                        &fallback.dialog.lists.type_label,
                    ),
                    items_label: Self::get_string(
                        value,
                        &["dialog", "lists", "items_label"],
                        &fallback.dialog.lists.items_label,
                    ),
                    type_bullet_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_bullet_title"],
                        &fallback.dialog.lists.type_bullet_title,
                    ),
                    type_bullet_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_bullet_desc"],
                        &fallback.dialog.lists.type_bullet_desc,
                    ),
                    type_ordered_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_ordered_title"],
                        &fallback.dialog.lists.type_ordered_title,
                    ),
                    type_ordered_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_ordered_desc"],
                        &fallback.dialog.lists.type_ordered_desc,
                    ),
                    type_unordered_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_unordered_title"],
                        &fallback.dialog.lists.type_unordered_title,
                    ),
                    type_unordered_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_unordered_desc"],
                        &fallback.dialog.lists.type_unordered_desc,
                    ),
                    type_task_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_task_title"],
                        &fallback.dialog.lists.type_task_title,
                    ),
                    type_task_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_task_desc"],
                        &fallback.dialog.lists.type_task_desc,
                    ),
                    type_task_nodot_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_task_nodot_title"],
                        &fallback.dialog.lists.type_task_nodot_title,
                    ),
                    type_task_nodot_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_task_nodot_desc"],
                        &fallback.dialog.lists.type_task_nodot_desc,
                    ),
                    type_definition_title: Self::get_string(
                        value,
                        &["dialog", "lists", "type_definition_title"],
                        &fallback.dialog.lists.type_definition_title,
                    ),
                    type_definition_desc: Self::get_string(
                        value,
                        &["dialog", "lists", "type_definition_desc"],
                        &fallback.dialog.lists.type_definition_desc,
                    ),
                },
                tables: TablesDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "tables", "title"],
                        &fallback.dialog.tables.title,
                    ),
                    rows_label: Self::get_string(
                        value,
                        &["dialog", "tables", "rows_label"],
                        &fallback.dialog.tables.rows_label,
                    ),
                    columns_label: Self::get_string(
                        value,
                        &["dialog", "tables", "columns_label"],
                        &fallback.dialog.tables.columns_label,
                    ),
                    alignment_title: Self::get_string(
                        value,
                        &["dialog", "tables", "alignment_title"],
                        &fallback.dialog.tables.alignment_title,
                    ),
                    include_header: Self::get_string(
                        value,
                        &["dialog", "tables", "include_header"],
                        &fallback.dialog.tables.include_header,
                    ),
                    edit_alignment: Self::get_string(
                        value,
                        &["dialog", "tables", "edit_alignment"],
                        &fallback.dialog.tables.edit_alignment,
                    ),
                    align_left: Self::get_string(
                        value,
                        &["dialog", "tables", "align_left"],
                        &fallback.dialog.tables.align_left,
                    ),
                    align_center: Self::get_string(
                        value,
                        &["dialog", "tables", "align_center"],
                        &fallback.dialog.tables.align_center,
                    ),
                    align_right: Self::get_string(
                        value,
                        &["dialog", "tables", "align_right"],
                        &fallback.dialog.tables.align_right,
                    ),
                    selected_format: Self::get_string(
                        value,
                        &["dialog", "tables", "selected_format"],
                        &fallback.dialog.tables.selected_format,
                    ),
                },
                tabs: TabsDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "tabs", "title"],
                        &fallback.dialog.tabs.title,
                    ),
                    empty_label: Self::get_string(
                        value,
                        &["dialog", "tabs", "empty_label"],
                        &fallback.dialog.tabs.empty_label,
                    ),
                    add_button: Self::get_string(
                        value,
                        &["dialog", "tabs", "add_button"],
                        &fallback.dialog.tabs.add_button,
                    ),
                    content_for: Self::get_string(
                        value,
                        &["dialog", "tabs", "content_for"],
                        &fallback.dialog.tabs.content_for,
                    ),
                },
                sliderdeck: SliderDeckDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "title"],
                        &fallback.dialog.sliderdeck.title,
                    ),
                    timer_label: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "timer_label"],
                        &fallback.dialog.sliderdeck.timer_label,
                    ),
                    timer_tooltip: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "timer_tooltip"],
                        &fallback.dialog.sliderdeck.timer_tooltip,
                    ),
                    seconds_label: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "seconds_label"],
                        &fallback.dialog.sliderdeck.seconds_label,
                    ),
                    empty_label: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "empty_label"],
                        &fallback.dialog.sliderdeck.empty_label,
                    ),
                    add_button: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "add_button"],
                        &fallback.dialog.sliderdeck.add_button,
                    ),
                    content_for: Self::get_string(
                        value,
                        &["dialog", "sliderdeck", "content_for"],
                        &fallback.dialog.sliderdeck.content_for,
                    ),
                },
                math: MathDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "math", "title"],
                        &fallback.dialog.math.title,
                    ),
                    mode_label: Self::get_string(
                        value,
                        &["dialog", "math", "mode_label"],
                        &fallback.dialog.math.mode_label,
                    ),
                    inline_radio: Self::get_string(
                        value,
                        &["dialog", "math", "inline_radio"],
                        &fallback.dialog.math.inline_radio,
                    ),
                    block_radio: Self::get_string(
                        value,
                        &["dialog", "math", "block_radio"],
                        &fallback.dialog.math.block_radio,
                    ),
                    templates_label: Self::get_string(
                        value,
                        &["dialog", "math", "templates_label"],
                        &fallback.dialog.math.templates_label,
                    ),
                    expression_label: Self::get_string(
                        value,
                        &["dialog", "math", "expression_label"],
                        &fallback.dialog.math.expression_label,
                    ),
                    tip_text: Self::get_string(
                        value,
                        &["dialog", "math", "tip_text"],
                        &fallback.dialog.math.tip_text,
                    ),
                    status_waiting: Self::get_string(
                        value,
                        &["dialog", "math", "status_waiting"],
                        &fallback.dialog.math.status_waiting,
                    ),
                    status_valid: Self::get_string(
                        value,
                        &["dialog", "math", "status_valid"],
                        &fallback.dialog.math.status_valid,
                    ),
                    no_templates: Self::get_string(
                        value,
                        &["dialog", "math", "no_templates"],
                        &fallback.dialog.math.no_templates,
                    ),
                },
                mermaid: MermaidDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "mermaid", "title"],
                        &fallback.dialog.mermaid.title,
                    ),
                    type_label: Self::get_string(
                        value,
                        &["dialog", "mermaid", "type_label"],
                        &fallback.dialog.mermaid.type_label,
                    ),
                    diagram_flowchart: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_flowchart"],
                        &fallback.dialog.mermaid.diagram_flowchart,
                    ),
                    diagram_sequence: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_sequence"],
                        &fallback.dialog.mermaid.diagram_sequence,
                    ),
                    diagram_pie: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_pie"],
                        &fallback.dialog.mermaid.diagram_pie,
                    ),
                    diagram_gitgraph: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_gitgraph"],
                        &fallback.dialog.mermaid.diagram_gitgraph,
                    ),
                    diagram_class: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_class"],
                        &fallback.dialog.mermaid.diagram_class,
                    ),
                    diagram_custom: Self::get_string(
                        value,
                        &["dialog", "mermaid", "diagram_custom"],
                        &fallback.dialog.mermaid.diagram_custom,
                    ),
                },
                mention: MentionDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "mention", "title"],
                        &fallback.dialog.mention.title,
                    ),
                    mention_label: Self::get_string(
                        value,
                        &["dialog", "mention", "mention_label"],
                        &fallback.dialog.mention.mention_label,
                    ),
                    username_label: Self::get_string(
                        value,
                        &["dialog", "mention", "username_label"],
                        &fallback.dialog.mention.username_label,
                    ),
                    realname_label: Self::get_string(
                        value,
                        &["dialog", "mention", "realname_label"],
                        &fallback.dialog.mention.realname_label,
                    ),
                    realname_placeholder: Self::get_string(
                        value,
                        &["dialog", "mention", "realname_placeholder"],
                        &fallback.dialog.mention.realname_placeholder,
                    ),
                    valid_button: Self::get_string(
                        value,
                        &["dialog", "mention", "valid_button"],
                        &fallback.dialog.mention.valid_button,
                    ),
                    error_button: Self::get_string(
                        value,
                        &["dialog", "mention", "error_button"],
                        &fallback.dialog.mention.error_button,
                    ),
                    status_waiting: Self::get_string(
                        value,
                        &["dialog", "mention", "status_waiting"],
                        &fallback.dialog.mention.status_waiting,
                    ),
                    status_checking: Self::get_string(
                        value,
                        &["dialog", "mention", "status_checking"],
                        &fallback.dialog.mention.status_checking,
                    ),
                    status_invalid_value: Self::get_string(
                        value,
                        &["dialog", "mention", "status_invalid_value"],
                        &fallback.dialog.mention.status_invalid_value,
                    ),
                    status_unsupported: Self::get_string(
                        value,
                        &["dialog", "mention", "status_unsupported"],
                        &fallback.dialog.mention.status_unsupported,
                    ),
                    status_not_implemented: Self::get_string(
                        value,
                        &["dialog", "mention", "status_not_implemented"],
                        &fallback.dialog.mention.status_not_implemented,
                    ),
                    status_blocked: Self::get_string(
                        value,
                        &["dialog", "mention", "status_blocked"],
                        &fallback.dialog.mention.status_blocked,
                    ),
                    status_timeout: Self::get_string(
                        value,
                        &["dialog", "mention", "status_timeout"],
                        &fallback.dialog.mention.status_timeout,
                    ),
                    status_prefix: Self::get_string(
                        value,
                        &["dialog", "mention", "status_prefix"],
                        &fallback.dialog.mention.status_prefix,
                    ),
                },
                diagnostics_reference: DiagnosticsReferenceDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "title"],
                        &fallback.dialog.diagnostics_reference.title,
                    ),
                    search_label: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "search_label"],
                        &fallback.dialog.diagnostics_reference.search_label,
                    ),
                    search_placeholder: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "search_placeholder"],
                        &fallback.dialog.diagnostics_reference.search_placeholder,
                    ),
                    severity_label: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_label"],
                        &fallback.dialog.diagnostics_reference.severity_label,
                    ),
                    severity_all: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_all"],
                        &fallback.dialog.diagnostics_reference.severity_all,
                    ),
                    severity_error: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_error"],
                        &fallback.dialog.diagnostics_reference.severity_error,
                    ),
                    severity_warning: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_warning"],
                        &fallback.dialog.diagnostics_reference.severity_warning,
                    ),
                    severity_info: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_info"],
                        &fallback.dialog.diagnostics_reference.severity_info,
                    ),
                    severity_hint: Self::get_string(
                        value,
                        &["dialog", "diagnostics_reference", "severity_hint"],
                        &fallback.dialog.diagnostics_reference.severity_hint,
                    ),
                },
                open_local_file: OpenLocalFileDialogTranslations {
                    discard_open: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "discard_open"],
                        &fallback.dialog.open_local_file.discard_open,
                    ),
                    save_open: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "save_open"],
                        &fallback.dialog.open_local_file.save_open,
                    ),
                    open_button: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "open_button"],
                        &fallback.dialog.open_local_file.open_button,
                    ),
                    cancel_button: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "cancel_button"],
                        &fallback.dialog.open_local_file.cancel_button,
                    ),
                    tooltip_discard: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "tooltip_discard"],
                        &fallback.dialog.open_local_file.tooltip_discard,
                    ),
                    tooltip_cancel: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "tooltip_cancel"],
                        &fallback.dialog.open_local_file.tooltip_cancel,
                    ),
                    tooltip_save_open: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "tooltip_save_open"],
                        &fallback.dialog.open_local_file.tooltip_save_open,
                    ),
                    tooltip_open: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "tooltip_open"],
                        &fallback.dialog.open_local_file.tooltip_open,
                    ),
                    unsaved_changes_message: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "unsaved_changes_message"],
                        &fallback.dialog.open_local_file.unsaved_changes_message,
                    ),
                    close_button: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "close_button"],
                        &fallback.dialog.open_local_file.close_button,
                    ),
                    missing_target_message: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "missing_target_message"],
                        &fallback.dialog.open_local_file.missing_target_message,
                    ),
                    missing_target_detail: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "missing_target_detail"],
                        &fallback.dialog.open_local_file.missing_target_detail,
                    ),
                    tooltip_close: Self::get_string(
                        value,
                        &["dialog", "open_local_file", "tooltip_close"],
                        &fallback.dialog.open_local_file.tooltip_close,
                    ),
                },
                export: ExportDialogTranslations {
                    title: Self::get_string(
                        value,
                        &["dialog", "export", "title"],
                        &fallback.dialog.export.title,
                    ),
                    pdf_radio: Self::get_string(
                        value,
                        &["dialog", "export", "pdf_radio"],
                        &fallback.dialog.export.pdf_radio,
                    ),
                    html_radio: Self::get_string(
                        value,
                        &["dialog", "export", "html_radio"],
                        &fallback.dialog.export.html_radio,
                    ),
                    cancel_button: Self::get_string(
                        value,
                        &["dialog", "export", "cancel_button"],
                        &fallback.dialog.export.cancel_button,
                    ),
                    export_button: Self::get_string(
                        value,
                        &["dialog", "export", "export_button"],
                        &fallback.dialog.export.export_button,
                    ),
                    save_pdf_title: Self::get_string(
                        value,
                        &["dialog", "export", "save_pdf_title"],
                        &fallback.dialog.export.save_pdf_title,
                    ),
                    save_html_title: Self::get_string(
                        value,
                        &["dialog", "export", "save_html_title"],
                        &fallback.dialog.export.save_html_title,
                    ),
                    filter_pdf: Self::get_string(
                        value,
                        &["dialog", "export", "filter_pdf"],
                        &fallback.dialog.export.filter_pdf,
                    ),
                    filter_html: Self::get_string(
                        value,
                        &["dialog", "export", "filter_html"],
                        &fallback.dialog.export.filter_html,
                    ),
                    paper_size_locked_tooltip: Self::get_string(
                        value,
                        &["dialog", "export", "paper_size_locked_tooltip"],
                        &fallback.dialog.export.paper_size_locked_tooltip,
                    ),
                },
                export_complete: ExportCompleteDialogTranslations {
                    close_button: Self::get_string(
                        value,
                        &["dialog", "export_complete", "close_button"],
                        &fallback.dialog.export_complete.close_button,
                    ),
                    open_folder: Self::get_string(
                        value,
                        &["dialog", "export_complete", "open_folder"],
                        &fallback.dialog.export_complete.open_folder,
                    ),
                    open_document: Self::get_string(
                        value,
                        &["dialog", "export_complete", "open_document"],
                        &fallback.dialog.export_complete.open_document,
                    ),
                },
            },
            settings: SettingsTranslations {
                title: Self::get_string(value, &["settings", "title"], &fallback.settings.title),
                close: Self::get_string(value, &["settings", "close"], &fallback.settings.close),
                not_available_yet: Self::get_string(
                    value,
                    &["settings", "not_available_yet"],
                    &fallback.settings.not_available_yet,
                ),
                tabs: SettingsTabsTranslations {
                    application: Self::get_string(
                        value,
                        &["settings", "tabs", "application"],
                        &fallback.settings.tabs.application,
                    ),
                    editor: Self::get_string(
                        value,
                        &["settings", "tabs", "editor"],
                        &fallback.settings.tabs.editor,
                    ),
                    intelligence: Self::get_string(
                        value,
                        &["settings", "tabs", "intelligence"],
                        &fallback.settings.tabs.intelligence,
                    ),
                    language: Self::get_string(
                        value,
                        &["settings", "tabs", "language"],
                        &fallback.settings.tabs.language,
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
                    print_preview: Self::get_string(
                        value,
                        &["settings", "tabs", "print_preview"],
                        &fallback.settings.tabs.print_preview,
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
                    table_auto_align_label: Self::get_string(
                        value,
                        &["settings", "editor", "table_auto_align_label"],
                        &fallback.settings.editor.table_auto_align_label,
                    ),
                    table_auto_align_description: Self::get_string(
                        value,
                        &["settings", "editor", "table_auto_align_description"],
                        &fallback.settings.editor.table_auto_align_description,
                    ),
                },
                intelligence: SettingsIntelligenceTranslations {
                    section_intelligence: Self::get_string(
                        value,
                        &["settings", "intelligence", "section_intelligence"],
                        &fallback.settings.intelligence.section_intelligence,
                    ),
                    intro_description: Self::get_string(
                        value,
                        &["settings", "intelligence", "intro_description"],
                        &fallback.settings.intelligence.intro_description,
                    ),
                    section_issues: Self::get_string(
                        value,
                        &["settings", "intelligence", "section_issues"],
                        &fallback.settings.intelligence.section_issues,
                    ),
                    diagnostics_underlines_label: Self::get_string(
                        value,
                        &["settings", "intelligence", "diagnostics_underlines_label"],
                        &fallback.settings.intelligence.diagnostics_underlines_label,
                    ),
                    diagnostics_underlines_description: Self::get_string(
                        value,
                        &[
                            "settings",
                            "intelligence",
                            "diagnostics_underlines_description",
                        ],
                        &fallback
                            .settings
                            .intelligence
                            .diagnostics_underlines_description,
                    ),
                    section_insights: Self::get_string(
                        value,
                        &["settings", "intelligence", "section_insights"],
                        &fallback.settings.intelligence.section_insights,
                    ),
                    markdown_insights_label: Self::get_string(
                        value,
                        &["settings", "intelligence", "markdown_insights_label"],
                        &fallback.settings.intelligence.markdown_insights_label,
                    ),
                    markdown_insights_description: Self::get_string(
                        value,
                        &["settings", "intelligence", "markdown_insights_description"],
                        &fallback.settings.intelligence.markdown_insights_description,
                    ),
                    issue_insights_label: Self::get_string(
                        value,
                        &["settings", "intelligence", "issue_insights_label"],
                        &fallback.settings.intelligence.issue_insights_label,
                    ),
                    issue_insights_description: Self::get_string(
                        value,
                        &["settings", "intelligence", "issue_insights_description"],
                        &fallback.settings.intelligence.issue_insights_description,
                    ),
                    section_highlighting: Self::get_string(
                        value,
                        &["settings", "intelligence", "section_highlighting"],
                        &fallback.settings.intelligence.section_highlighting,
                    ),
                    syntax_highlighting_label: Self::get_string(
                        value,
                        &["settings", "intelligence", "syntax_highlighting_label"],
                        &fallback.settings.intelligence.syntax_highlighting_label,
                    ),
                    syntax_highlighting_description: Self::get_string(
                        value,
                        &[
                            "settings",
                            "intelligence",
                            "syntax_highlighting_description",
                        ],
                        &fallback
                            .settings
                            .intelligence
                            .syntax_highlighting_description,
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
                    toolbar_svg_text_label: Self::get_string(
                        value,
                        &["settings", "appearance", "toolbar_svg_text_label"],
                        &fallback.settings.appearance.toolbar_svg_text_label,
                    ),
                    toolbar_svg_text_description: Self::get_string(
                        value,
                        &["settings", "appearance", "toolbar_svg_text_description"],
                        &fallback.settings.appearance.toolbar_svg_text_description,
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
                    color_mode_system: Self::get_string(
                        value,
                        &["settings", "appearance", "color_mode_system"],
                        &fallback.settings.appearance.color_mode_system,
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
                    toc_depth_label: Self::get_string(
                        value,
                        &["settings", "layout", "toc_depth_label"],
                        &fallback.settings.layout.toc_depth_label,
                    ),
                    toc_depth_description: Self::get_string(
                        value,
                        &["settings", "layout", "toc_depth_description"],
                        &fallback.settings.layout.toc_depth_description,
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
                    page_view_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_label"],
                        &fallback.settings.layout.page_view_label,
                    ),
                    page_view_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_description"],
                        &fallback.settings.layout.page_view_description,
                    ),
                    page_view_paper_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_paper_label"],
                        &fallback.settings.layout.page_view_paper_label,
                    ),
                    page_view_paper_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_paper_description"],
                        &fallback.settings.layout.page_view_paper_description,
                    ),
                    page_view_orientation_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_orientation_label"],
                        &fallback.settings.layout.page_view_orientation_label,
                    ),
                    page_view_orientation_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_orientation_description"],
                        &fallback.settings.layout.page_view_orientation_description,
                    ),
                    page_view_orientation_portrait: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_orientation_portrait"],
                        &fallback.settings.layout.page_view_orientation_portrait,
                    ),
                    page_view_orientation_landscape: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_orientation_landscape"],
                        &fallback.settings.layout.page_view_orientation_landscape,
                    ),
                    page_view_margin_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_margin_label"],
                        &fallback.settings.layout.page_view_margin_label,
                    ),
                    page_view_margin_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_margin_description"],
                        &fallback.settings.layout.page_view_margin_description,
                    ),
                    page_view_page_numbers_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_page_numbers_label"],
                        &fallback.settings.layout.page_view_page_numbers_label,
                    ),
                    page_view_page_numbers_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_page_numbers_description"],
                        &fallback.settings.layout.page_view_page_numbers_description,
                    ),
                    page_view_columns_label: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_columns_label"],
                        &fallback.settings.layout.page_view_columns_label,
                    ),
                    page_view_columns_description: Self::get_string(
                        value,
                        &["settings", "layout", "page_view_columns_description"],
                        &fallback.settings.layout.page_view_columns_description,
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
                    my_data_body_text: Self::get_string(
                        value,
                        &["settings", "advanced", "my_data_body_text"],
                        &fallback.settings.advanced.my_data_body_text,
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
                page_appearance: Self::get_string(
                    value,
                    &["welcome", "page_appearance"],
                    &fallback.welcome.page_appearance,
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

                appearance_header: Self::get_string(
                    value,
                    &["welcome", "appearance_header"],
                    &fallback.welcome.appearance_header,
                ),
                appearance_description: Self::get_string(
                    value,
                    &["welcome", "appearance_description"],
                    &fallback.welcome.appearance_description,
                ),
                appearance_light: Self::get_string(
                    value,
                    &["welcome", "appearance_light"],
                    &fallback.welcome.appearance_light,
                ),
                appearance_dark: Self::get_string(
                    value,
                    &["welcome", "appearance_dark"],
                    &fallback.welcome.appearance_dark,
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
                change_layout: Self::get_string(
                    value,
                    &["titlebar", "change_layout"],
                    &fallback.titlebar.change_layout,
                ),
                layout_state_editor_only: Self::get_string(
                    value,
                    &["titlebar", "layout_state_editor_only"],
                    &fallback.titlebar.layout_state_editor_only,
                ),
                layout_state_view_only: Self::get_string(
                    value,
                    &["titlebar", "layout_state_view_only"],
                    &fallback.titlebar.layout_state_view_only,
                ),
                layout_state_editor_and_view_separate: Self::get_string(
                    value,
                    &["titlebar", "layout_state_editor_and_view_separate"],
                    &fallback.titlebar.layout_state_editor_and_view_separate,
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
                about_section: Self::get_string(
                    value,
                    &["messages", "about_section"],
                    &fallback.messages.about_section,
                ),
                code_view_short: Self::get_string(
                    value,
                    &["messages", "code_view_short"],
                    &fallback.messages.code_view_short,
                ),
                live_preview: Self::get_string(
                    value,
                    &["messages", "live_preview"],
                    &fallback.messages.live_preview,
                ),
                print_preview_short: Self::get_string(
                    value,
                    &["messages", "print_preview_short"],
                    &fallback.messages.print_preview_short,
                ),
                select_local_file: Self::get_string(
                    value,
                    &["messages", "select_local_file"],
                    &fallback.messages.select_local_file,
                ),
                theme_label: Self::get_string(
                    value,
                    &["messages", "theme_label"],
                    &fallback.messages.theme_label,
                ),
                open_in_editor_prompt: Self::get_string(
                    value,
                    &["messages", "open_in_editor_prompt"],
                    &fallback.messages.open_in_editor_prompt,
                ),
                exporting_html: Self::get_string(
                    value,
                    &["messages", "exporting_html"],
                    &fallback.messages.exporting_html,
                ),
                exporting_pdf: Self::get_string(
                    value,
                    &["messages", "exporting_pdf"],
                    &fallback.messages.exporting_pdf,
                ),
                generating_html: Self::get_string(
                    value,
                    &["messages", "generating_html"],
                    &fallback.messages.generating_html,
                ),
                generating_pdf: Self::get_string(
                    value,
                    &["messages", "generating_pdf"],
                    &fallback.messages.generating_pdf,
                ),
                html_export_complete: Self::get_string(
                    value,
                    &["messages", "html_export_complete"],
                    &fallback.messages.html_export_complete,
                ),
                pdf_export_complete: Self::get_string(
                    value,
                    &["messages", "pdf_export_complete"],
                    &fallback.messages.pdf_export_complete,
                ),
                pdf_export_success: Self::get_string(
                    value,
                    &["messages", "pdf_export_success"],
                    &fallback.messages.pdf_export_success,
                ),
                html_export_success: Self::get_string(
                    value,
                    &["messages", "html_export_success"],
                    &fallback.messages.html_export_success,
                ),
                insert_row_above: Self::get_string(
                    value,
                    &["messages", "insert_row_above"],
                    &fallback.messages.insert_row_above,
                ),
                insert_row_below: Self::get_string(
                    value,
                    &["messages", "insert_row_below"],
                    &fallback.messages.insert_row_below,
                ),
                insert_column_left: Self::get_string(
                    value,
                    &["messages", "insert_column_left"],
                    &fallback.messages.insert_column_left,
                ),
                insert_column_right: Self::get_string(
                    value,
                    &["messages", "insert_column_right"],
                    &fallback.messages.insert_column_right,
                ),
                export_phase: ExportPhaseTranslations {
                    preparing: Self::get_string(
                        value,
                        &["messages", "export_phase", "preparing"],
                        &fallback.messages.export_phase.preparing,
                    ),
                    loading: Self::get_string(
                        value,
                        &["messages", "export_phase", "loading"],
                        &fallback.messages.export_phase.loading,
                    ),
                    paginating: Self::get_string(
                        value,
                        &["messages", "export_phase", "paginating"],
                        &fallback.messages.export_phase.paginating,
                    ),
                    applying_print_css: Self::get_string(
                        value,
                        &["messages", "export_phase", "applying_print_css"],
                        &fallback.messages.export_phase.applying_print_css,
                    ),
                    writing_output: Self::get_string(
                        value,
                        &["messages", "export_phase", "writing_output"],
                        &fallback.messages.export_phase.writing_output,
                    ),
                    restoring_preview: Self::get_string(
                        value,
                        &["messages", "export_phase", "restoring_preview"],
                        &fallback.messages.export_phase.restoring_preview,
                    ),
                    done: Self::get_string(
                        value,
                        &["messages", "export_phase", "done"],
                        &fallback.messages.export_phase.done,
                    ),
                },
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
        // Validate locale code (ISO 639-1, optionally with an ISO 3166-1
        // region subtag, e.g. `zh-CN`).
        if !is_valid_locale_code(locale) {
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
        assert!(manager.load_locale("zh-cn").is_err()); // Region must be uppercase
        assert!(manager.load_locale("zh_CN").is_err()); // Underscore, not hyphen
        assert!(manager.load_locale("zho-CN").is_err()); // 3-letter language part
        assert!(manager.load_locale("zh-CHN").is_err()); // 3-letter region part
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
    fn smoke_test_load_locale_with_fallback() {
        let manager = match SimpleLocalizationManager::new() {
            Ok(m) => m,
            Err(_) => {
                println!("Skipping: assets not available in test environment");
                return;
            }
        };

        // A locale with no matching file (region-qualified or not) lands on
        // English rather than leaving the manager on whatever was loaded
        // before.
        manager.load_locale_with_fallback("zz-XX");
        assert_eq!(manager.current_locale(), "en");

        manager.load_locale_with_fallback("zz");
        assert_eq!(manager.current_locale(), "en");

        // A directly-loadable locale doesn't fall through at all.
        manager.load_locale_with_fallback("en");
        assert_eq!(manager.current_locale(), "en");
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
        // Bare ISO 639-1 codes
        assert!(is_valid_locale_code("en"));
        assert!(is_valid_locale_code("da"));
        assert!(is_valid_locale_code("de"));
        assert!(is_valid_locale_code("fr"));

        assert!(!is_valid_locale_code("eng")); // Too long
        assert!(!is_valid_locale_code("e")); // Too short
        assert!(!is_valid_locale_code("EN")); // Uppercase
        assert!(!is_valid_locale_code("e1")); // Contains digit
        assert!(!is_valid_locale_code("e-")); // Contains special char

        // ISO 639-1 + ISO 3166-1 region subtag (BCP 47 style)
        assert!(is_valid_locale_code("zh-CN"));
        assert!(is_valid_locale_code("zh-TW"));

        assert!(!is_valid_locale_code("zh-cn")); // Region must be uppercase
        assert!(!is_valid_locale_code("ZH-CN")); // Language must be lowercase
        assert!(!is_valid_locale_code("zh_CN")); // Underscore, not hyphen
        assert!(!is_valid_locale_code("zho-CN")); // 3-letter language part
        assert!(!is_valid_locale_code("zh-CHN")); // 3-letter region part
        assert!(!is_valid_locale_code("zh-1N")); // Region contains digit
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

    /// Loads a value for every newly-added key (from a distinct-from-default
    /// TOML fixture) to catch a typo'd `get_string` key path — which would
    /// otherwise silently fall back to the (identical-looking) English
    /// default and go unnoticed.
    #[test]
    fn smoke_test_newly_added_keys_load_from_toml() {
        let fallback = SimpleLocalizationManager::load_default_translations();
        let toml = r#"
[menu]
bookmark_line = "XLine {}"
format_table = "XFormat table"
insert_update_toc = "XInsert / Update TOC"

[footer]
contents = "XContents"
no_headings = "XNo headings"
issues_prefix = "XIssue"
no_diagnostics = "XNo diagnostics"
toc_tooltip = "XToggle TOC"

[dialog.open_local_file]
unsaved_changes_message = "X{document} unsaved"

[dialog.export]
paper_size_locked_tooltip = "XLocked"

[settings]
not_available_yet = "XNot available"

[settings.layout]
page_view_columns_label = "XPages per Row"
page_view_columns_description = "XColumns desc"

[settings.advanced]
my_data_body_text = "XBody text"

[titlebar]
change_layout = "XChange layout"
layout_state_editor_only = "Xeditor only"
layout_state_view_only = "Xview only"
layout_state_editor_and_view_separate = "Xseparate"

[messages]
about_section = "XAbout"
code_view_short = "XCode"
live_preview = "XLive preview"
print_preview_short = "XPrint preview"
select_local_file = "XSelect Local File"
theme_label = "XTheme:"
open_in_editor_prompt = "XOpen {filename}?"
exporting_html = "XExporting HTML"
exporting_pdf = "XExporting PDF"
generating_html = "XGenerating HTML"
generating_pdf = "XGenerating PDF"
html_export_complete = "XHTML done"
pdf_export_complete = "XPDF done"
pdf_export_success = "XPDF ok"
html_export_success = "XHTML ok"
insert_row_above = "XRow above"
insert_row_below = "XRow below"
insert_column_left = "XColumn left"
insert_column_right = "XColumn right"

[messages.export_phase]
preparing = "XPreparing"
loading = "XLoading"
paginating = "XPaginating"
applying_print_css = "XApplying"
writing_output = "XWriting"
restoring_preview = "XRestoring"
done = "XDone"
"#;

        let value: toml::Value = toml::from_str(toml).expect("valid toml");
        let t = SimpleLocalizationManager::load_translations_from_value(&value, &fallback);

        assert_eq!(t.menu.bookmark_line, "XLine {}");
        assert_eq!(t.menu.format_table, "XFormat table");
        assert_eq!(t.menu.insert_update_toc, "XInsert / Update TOC");

        assert_eq!(t.footer.contents, "XContents");
        assert_eq!(t.footer.no_headings, "XNo headings");
        assert_eq!(t.footer.issues_prefix, "XIssue");
        assert_eq!(t.footer.no_diagnostics, "XNo diagnostics");
        assert_eq!(t.footer.toc_tooltip, "XToggle TOC");

        assert_eq!(
            t.dialog.open_local_file.unsaved_changes_message,
            "X{document} unsaved"
        );
        assert_eq!(t.dialog.export.paper_size_locked_tooltip, "XLocked");

        assert_eq!(t.settings.not_available_yet, "XNot available");
        assert_eq!(t.settings.layout.page_view_columns_label, "XPages per Row");
        assert_eq!(
            t.settings.layout.page_view_columns_description,
            "XColumns desc"
        );
        assert_eq!(t.settings.advanced.my_data_body_text, "XBody text");

        assert_eq!(t.titlebar.change_layout, "XChange layout");
        assert_eq!(t.titlebar.layout_state_editor_only, "Xeditor only");
        assert_eq!(t.titlebar.layout_state_view_only, "Xview only");
        assert_eq!(
            t.titlebar.layout_state_editor_and_view_separate,
            "Xseparate"
        );

        assert_eq!(t.messages.about_section, "XAbout");
        assert_eq!(t.messages.code_view_short, "XCode");
        assert_eq!(t.messages.live_preview, "XLive preview");
        assert_eq!(t.messages.print_preview_short, "XPrint preview");
        assert_eq!(t.messages.select_local_file, "XSelect Local File");
        assert_eq!(t.messages.theme_label, "XTheme:");
        assert_eq!(t.messages.open_in_editor_prompt, "XOpen {filename}?");
        assert_eq!(t.messages.exporting_html, "XExporting HTML");
        assert_eq!(t.messages.exporting_pdf, "XExporting PDF");
        assert_eq!(t.messages.generating_html, "XGenerating HTML");
        assert_eq!(t.messages.generating_pdf, "XGenerating PDF");
        assert_eq!(t.messages.html_export_complete, "XHTML done");
        assert_eq!(t.messages.pdf_export_complete, "XPDF done");
        assert_eq!(t.messages.pdf_export_success, "XPDF ok");
        assert_eq!(t.messages.html_export_success, "XHTML ok");
        assert_eq!(t.messages.insert_row_above, "XRow above");
        assert_eq!(t.messages.insert_row_below, "XRow below");
        assert_eq!(t.messages.insert_column_left, "XColumn left");
        assert_eq!(t.messages.insert_column_right, "XColumn right");

        assert_eq!(t.messages.export_phase.preparing, "XPreparing");
        assert_eq!(t.messages.export_phase.loading, "XLoading");
        assert_eq!(t.messages.export_phase.paginating, "XPaginating");
        assert_eq!(t.messages.export_phase.applying_print_css, "XApplying");
        assert_eq!(t.messages.export_phase.writing_output, "XWriting");
        assert_eq!(t.messages.export_phase.restoring_preview, "XRestoring");
        assert_eq!(t.messages.export_phase.done, "XDone");
    }
}
