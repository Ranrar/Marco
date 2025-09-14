use anyhow::Result;
use fontconfig::Fontconfig;
use log::{debug, error, warn};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Global font cache for fast access to monospace fonts
static MONOSPACE_FONT_CACHE: OnceLock<Vec<FontFamily>> = OnceLock::new();

/// Font family information
#[derive(Debug, Clone, PartialEq)]
pub struct FontFamily {
    pub name: String,
    pub is_monospace: bool,
}

/// Available fonts categorized by type
#[derive(Debug, Clone, Default)]
pub struct AvailableFonts {
    pub monospace: Vec<FontFamily>,
}

/// Font loader using fontconfig on Linux
pub struct FontLoader {
    fc: Option<Fontconfig>,
}

impl FontLoader {
    /// Create a new font loader instance
    pub fn new() -> Result<Self> {
        debug!("Initializing font loader with fontconfig");
        let fc =
            Fontconfig::new().ok_or_else(|| anyhow::anyhow!("Failed to initialize fontconfig"))?;
        Ok(Self { fc: Some(fc) })
    }

    /// Initialize monospace font cache at startup (fast)
    pub fn init_monospace_cache() -> Result<()> {
        debug!("Initializing monospace font cache at startup");
        let loader = Self::new()?;
        let monospace_fonts = loader.load_monospace_fonts_only()?;

        MONOSPACE_FONT_CACHE
            .set(monospace_fonts.clone())
            .map_err(|_| {
                anyhow::anyhow!("Failed to initialize font cache - already initialized")
            })?;

        debug!(
            "Monospace font cache initialized with {} fonts",
            monospace_fonts.len()
        );
        Ok(())
    }

    /// Get cached monospace fonts (very fast)
    pub fn get_cached_monospace_fonts() -> Vec<FontFamily> {
        MONOSPACE_FONT_CACHE.get().cloned().unwrap_or_else(|| {
            warn!("Monospace font cache not initialized, using fallback");
            vec![
                FontFamily {
                    name: "Monospace".to_string(),
                    is_monospace: true,
                },
                FontFamily {
                    name: "Courier New".to_string(),
                    is_monospace: true,
                },
                FontFamily {
                    name: "Fixed".to_string(),
                    is_monospace: true,
                },
            ]
        })
    }

    /// Load only monospace fonts (optimized for startup)
    pub fn load_monospace_fonts_only(&self) -> Result<Vec<FontFamily>> {
        let fc = self
            .fc
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Fontconfig not initialized"))?;

        debug!("Loading only monospace fonts for fast startup");

        let mut monospace_fonts = Vec::new();
        let mut font_map = HashMap::new();

        // Create pattern specifically for monospace fonts
        let pattern = fontconfig::Pattern::new(fc);
        // Note: We'll rely on name-based detection since fontconfig pattern API varies

        let fonts = fontconfig::list_fonts(&pattern, None);

        for font_pattern in fonts.iter() {
            if let Some(name) = font_pattern.name() {
                let font_name = name.to_string();

                // Skip empty names or duplicates
                if font_name.is_empty() || font_map.contains_key(&font_name) {
                    continue;
                }

                // Double-check it's actually monospace
                if self.is_monospace_font(&font_name) {
                    let font_family = FontFamily {
                        name: font_name.clone(),
                        is_monospace: true,
                    };

                    font_map.insert(font_name, font_family.clone());
                    monospace_fonts.push(font_family);
                }
            }
        }

        // Add common monospace fallbacks if not found
        self.add_monospace_fallbacks(&mut monospace_fonts);

        // Sort alphabetically
        monospace_fonts.sort_by(|a, b| a.name.cmp(&b.name));

        debug!("Loaded {} monospace fonts for cache", monospace_fonts.len());
        Ok(monospace_fonts)
    }

    /// Check if a font is monospace by its name or properties
    fn is_monospace_font(&self, font_name: &str) -> bool {
        let mono_keywords = [
            "mono",
            "monospace",
            "courier",
            "console",
            "terminal",
            "fixed",
            "typewriter",
            "code",
            "programming",
            "hack",
            "fira code",
            "source code",
            "jetbrains",
            "inconsolata",
            "ubuntu mono",
            "droid sans mono",
            "liberation mono",
            "dejavu sans mono",
        ];

        let name_lower = font_name.to_lowercase();
        mono_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    /// Add common monospace fallback fonts if they're not available
    fn add_monospace_fallbacks(&self, fonts: &mut Vec<FontFamily>) {
        let common_monospace = [
            "Monospace",
            "Fixed",
            "Courier New",
            "Courier",
            "Liberation Mono",
            "DejaVu Sans Mono",
        ];

        for mono_font in &common_monospace {
            if !fonts.iter().any(|f| f.name == *mono_font) {
                let font = FontFamily {
                    name: mono_font.to_string(),
                    is_monospace: true,
                };
                fonts.push(font);
            }
        }
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        match Self::new() {
            Ok(loader) => loader,
            Err(e) => {
                error!("Failed to create font loader: {}", e);
                Self { fc: None }
            }
        }
    }
}
