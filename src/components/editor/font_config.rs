use crate::logic::{
    loaders::font_loader::{AvailableFonts, FontLoader},
    swanson::{EditorSettings, Settings as AppSettings},
};
use anyhow::Result;
use gtk4::{pango, prelude::*, WrapMode};
use log::{debug, warn};
use sourceview5::{prelude::*, View as SourceView};

/// Editor configuration manager for fonts and display settings
pub struct EditorConfiguration {
    fonts: AvailableFonts,
    settings_path: String,
}

impl EditorConfiguration {
    /// Create a new editor configuration instance using cached fonts (fast)
    pub fn new(settings_path: &str) -> Result<Self> {
        // Use cached monospace fonts for fast loading
        let monospace_fonts = FontLoader::get_cached_monospace_fonts();

        let fonts = AvailableFonts {
            monospace: monospace_fonts.clone(),
        };

        Ok(Self {
            fonts,
            settings_path: settings_path.to_string(),
        })
    }

    /// Get current editor settings from storage
    pub fn get_current_editor_settings(&self) -> EditorDisplaySettings {
        match AppSettings::load_from_file(&self.settings_path) {
            Ok(settings) => {
                let editor = settings.editor.unwrap_or_default();
                let font_family = editor
                    .font
                    .unwrap_or_else(|| self.get_default_editor_font());
                let font_size = editor.font_size.unwrap_or(14);
                let line_height = editor.line_height.unwrap_or(1.4);
                let line_wrapping = editor.line_wrapping.unwrap_or(false);
                let show_invisibles = editor.show_invisibles.unwrap_or(false);
                let tabs_to_spaces = editor.tabs_to_spaces.unwrap_or(false);
                let syntax_colors = editor.syntax_colors.unwrap_or(true);

                debug!(
                    "Loaded editor settings from {}: {} {}px line-height:{} wrap:{} show_invisibles:{} tabs_to_spaces:{} syntax_colors:{}",
                    self.settings_path, font_family, font_size, line_height, line_wrapping, show_invisibles, tabs_to_spaces, syntax_colors
                );

                let show_line_numbers = settings
                    .layout
                    .as_ref()
                    .and_then(|l| l.show_line_numbers)
                    .unwrap_or(true);

                EditorDisplaySettings {
                    font_family,
                    font_size,
                    line_height,
                    line_wrapping,
                    show_invisibles,
                    tabs_to_spaces,
                    syntax_colors,
                    show_line_numbers,
                }
            }
            Err(e) => {
                warn!("Failed to load editor settings: {}", e);
                let default_settings = EditorDisplaySettings {
                    font_family: self.get_default_editor_font(),
                    font_size: 14,
                    line_height: 1.4,
                    line_wrapping: false,
                    show_invisibles: false,
                    tabs_to_spaces: false,
                    syntax_colors: true,
                    show_line_numbers: true,
                };
                debug!(
                    "Using default editor settings: {} {}px line-height:{} wrap:{} show_invisibles:{} tabs_to_spaces:{} syntax_colors:{}",
                    default_settings.font_family,
                    default_settings.font_size,
                    default_settings.line_height,
                    default_settings.line_wrapping,
                    default_settings.show_invisibles,
                    default_settings.tabs_to_spaces,
                    default_settings.syntax_colors
                );
                default_settings
            }
        }
    }

    /// Save editor settings to storage
    pub fn save_editor_settings(&self, editor_settings: &EditorDisplaySettings) -> Result<()> {
        debug!(
            "Saving editor settings: {} {}px line-height:{} wrap:{} show_invisibles:{} tabs_to_spaces:{} syntax_colors:{}",
            editor_settings.font_family,
            editor_settings.font_size,
            editor_settings.line_height,
            editor_settings.line_wrapping,
            editor_settings.show_invisibles,
            editor_settings.tabs_to_spaces,
            editor_settings.syntax_colors
        );

        let mut settings = AppSettings::load_from_file(&self.settings_path).unwrap_or_default();

        // Ensure editor settings exist
        if settings.editor.is_none() {
            settings.editor = Some(EditorSettings::default());
        }

        // Update editor settings
        if let Some(ref mut editor) = settings.editor {
            editor.font = Some(editor_settings.font_family.clone());
            editor.font_size = Some(editor_settings.font_size);
            editor.line_height = Some(editor_settings.line_height);
            editor.line_wrapping = Some(editor_settings.line_wrapping);
            editor.show_invisibles = Some(editor_settings.show_invisibles);
            editor.tabs_to_spaces = Some(editor_settings.tabs_to_spaces);
            editor.syntax_colors = Some(editor_settings.syntax_colors);
        }

        settings.save_to_file(&self.settings_path)?;
        debug!("Editor settings saved successfully");
        Ok(())
    }

    /// Apply editor settings to a SourceView widget
    pub fn apply_to_sourceview(
        &self,
        sourceview: &SourceView,
        editor_settings: &EditorDisplaySettings,
    ) {
        let font_desc = format!(
            "{} {}px",
            editor_settings.font_family, editor_settings.font_size
        );
        debug!(
            "Applying editor settings to SourceView: {} with line-height: {} wrap:{} show_invisibles:{} tabs_to_spaces:{} syntax_colors:{}",
            font_desc, editor_settings.line_height, editor_settings.line_wrapping,
            editor_settings.show_invisibles, editor_settings.tabs_to_spaces, editor_settings.syntax_colors
        );

        // Parse font description using Pango
        let _font_desc_pango = pango::FontDescription::from_string(&font_desc);

        // Apply line wrapping
        let wrap_mode = if editor_settings.line_wrapping {
            WrapMode::Word
        } else {
            WrapMode::None
        };
        sourceview.set_wrap_mode(wrap_mode);

        // Apply tabs to spaces setting
        sourceview.set_insert_spaces_instead_of_tabs(editor_settings.tabs_to_spaces);

        // Apply show invisible characters setting
        self.apply_show_invisibles_setting(sourceview, editor_settings.show_invisibles);

        // Apply syntax highlighting setting
        self.apply_syntax_highlighting_setting(sourceview, editor_settings.syntax_colors);

        // Apply line numbers setting
        sourceview.set_show_line_numbers(editor_settings.show_line_numbers);

        // Apply font and line height using CSS provider for more reliable rendering
        self.apply_font_and_line_height_via_css(
            sourceview,
            &editor_settings.font_family,
            editor_settings.font_size,
            editor_settings.line_height,
        );
    }

    /// Apply font settings and line height via CSS for more reliable font rendering
    fn apply_font_and_line_height_via_css(
        &self,
        sourceview: &SourceView,
        font_family: &str,
        font_size: u8,
        line_height: f32,
    ) {
        let css = format!(
            r#"
            textview {{
                font-family: "{}";
                font-size: {}px;
                line-height: {};
            }}
            textview text {{
                font-family: "{}";
                font-size: {}px;
                line-height: {};
            }}
            "#,
            font_family, font_size, line_height, font_family, font_size, line_height
        );

        debug!("Applying CSS font and line-height styles: {}", css);

        // Create CSS provider
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(&css);

        // Apply to the SourceView widget
        let style_context = sourceview.style_context();
        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    /// Apply show invisible characters setting using GtkSourceSpaceDrawer
    fn apply_show_invisibles_setting(&self, sourceview: &SourceView, show_invisibles: bool) {
        debug!("Setting show_invisibles to: {}", show_invisibles);

        let space_drawer = sourceview.space_drawer();

        if show_invisibles {
            // Enable showing all whitespace types at all locations
            use sourceview5::{SpaceLocationFlags, SpaceTypeFlags};

            // Show spaces, tabs, and newlines everywhere
            space_drawer.set_types_for_locations(
                SpaceLocationFlags::ALL,
                SpaceTypeFlags::SPACE | SpaceTypeFlags::TAB | SpaceTypeFlags::NEWLINE,
            );
        } else {
            // Disable whitespace visualization
            space_drawer.set_types_for_locations(
                sourceview5::SpaceLocationFlags::ALL,
                sourceview5::SpaceTypeFlags::NONE,
            );
        }

        // Enable or disable the space drawer matrix
        space_drawer.set_enable_matrix(show_invisibles);
    }

    /// Apply syntax highlighting setting using GtkSourceBuffer
    fn apply_syntax_highlighting_setting(&self, sourceview: &SourceView, syntax_colors: bool) {
        debug!("Setting syntax_colors to: {}", syntax_colors);

        if let Ok(buffer) = sourceview.buffer().downcast::<sourceview5::Buffer>() {
            buffer.set_highlight_syntax(syntax_colors);

            if syntax_colors {
                // Set up markdown language and style scheme when enabling syntax highlighting
                self.setup_markdown_syntax_highlighting(&buffer);
            }
        } else {
            warn!("Could not get SourceBuffer from SourceView for syntax highlighting");
        }
    }

    /// Set up markdown language and style scheme for syntax highlighting
    fn setup_markdown_syntax_highlighting(&self, buffer: &sourceview5::Buffer) {
        use sourceview5::LanguageManager;

        let language_manager = LanguageManager::default();

        // Set markdown language
        if let Some(markdown_language) = language_manager.language("markdown") {
            buffer.set_language(Some(&markdown_language));
            debug!("Set buffer language to markdown for syntax highlighting");
        } else {
            warn!("Could not find markdown language definition for syntax highlighting");
        }

        // Note: We do NOT set a style scheme here because the buffer already has
        // the correct marco-dark or marco-light scheme applied from ThemeManager.
        // Setting a different scheme here would override the custom background colors
        // and break the theme consistency.
    }

    /// Get default editor font (prefers monospace)
    pub fn get_default_editor_font(&self) -> String {
        if !self.fonts.monospace.is_empty() {
            // Prefer common monospace fonts for coding
            let preferred = [
                "Hack",
                "Fira Code",
                "Source Code Pro",
                "JetBrains Mono",
                "Ubuntu Mono",
                "Consolas",
                "Monaco",
                "Inconsolata",
            ];

            for font_name in &preferred {
                if self
                    .fonts
                    .monospace
                    .iter()
                    .any(|f| f.name.contains(font_name))
                {
                    return font_name.to_string();
                }
            }

            // Fallback to first available monospace
            if let Some(first_mono) = self.fonts.monospace.first() {
                return first_mono.name.clone();
            }
        }

        // Ultimate fallback
        "Monospace".to_string()
    }
}

/// Editor display settings structure
#[derive(Debug, Clone, PartialEq)]
pub struct EditorDisplaySettings {
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
    pub line_wrapping: bool,
    pub show_invisibles: bool,
    pub tabs_to_spaces: bool,
    pub syntax_colors: bool,
    pub show_line_numbers: bool,
}

impl Default for EditorDisplaySettings {
    fn default() -> Self {
        Self {
            font_family: "Monospace".to_string(),
            font_size: 16,
            line_height: 1.0,
            line_wrapping: false,
            show_invisibles: false,
            tabs_to_spaces: false,
            syntax_colors: true,
            show_line_numbers: true,
        }
    }
}
