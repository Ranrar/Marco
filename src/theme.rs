#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Theme {
        match s {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => Theme::System,
        }
    }
}

pub struct ThemeManager {
    current_theme: Theme,
    current_css_theme: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        ThemeManager {
            current_theme: Theme::System,
            current_css_theme: "standard".to_string(),
        }
    }

    pub fn get_theme(&self) -> Theme {
        self.current_theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }

    pub fn get_css_theme(&self) -> &str {
        &self.current_css_theme
    }

    pub fn set_css_theme(&mut self, theme: String) {
        self.current_css_theme = theme;
    }
}