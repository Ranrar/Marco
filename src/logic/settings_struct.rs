use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub editor: Option<EditorSettings>,
    pub appearance: Option<AppearanceSettings>,
    pub layout: Option<LayoutSettings>,
    pub language: Option<LanguageSettings>,
    pub window: Option<WindowSettings>,
    pub advanced: Option<AdvancedSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorSettings {
    pub font: Option<String>,
    pub font_size: Option<u8>,
    pub line_height: Option<f32>,
    pub line_wrapping: Option<bool>,
    pub auto_pairing: Option<bool>,
    pub show_invisibles: Option<bool>,
    pub tabs_to_spaces: Option<bool>,
    pub syntax_colors: Option<bool>,
    pub linting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppearanceSettings {
    pub color_mode: Option<String>,
    pub app_theme: Option<String>,
    pub preview_theme: Option<String>,
    pub ui_font: Option<String>,
    pub ui_font_size: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutSettings {
    pub view_mode: Option<String>,
    pub sync_scrolling: Option<bool>,
    pub editor_view_split: Option<u8>,
    pub show_line_numbers: Option<bool>,
    pub text_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageSettings {
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowSettings {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvancedSettings {
    pub enabled_variants: Option<Vec<String>>,
    pub plugins: Option<Vec<String>>,
}
