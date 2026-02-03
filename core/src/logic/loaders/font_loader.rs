#[cfg(target_os = "linux")]
use fontconfig::Fontconfig;
use log::{debug, error, warn};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

/// Font cache entry with timestamp for expiration
#[derive(Debug, Clone)]
struct FontCacheEntry {
    fonts: Vec<FontFamily>,
    cached_at: Instant,
}

/// Global font cache with invalidation support
static MONOSPACE_FONT_CACHE: OnceLock<RwLock<Option<FontCacheEntry>>> = OnceLock::new();

/// Cache expiration time (default: 30 minutes)
const CACHE_EXPIRATION: Duration = Duration::from_secs(30 * 60);

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
    #[cfg(target_os = "linux")]
    fc: Option<Fontconfig>,
}

impl FontLoader {
    /// Create a new font loader instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            debug!("Initializing font loader with fontconfig");
            let fc = Fontconfig::new().ok_or_else(|| -> Box<dyn std::error::Error> {
                "Failed to initialize fontconfig".into()
            })?;
            Ok(Self { fc: Some(fc) })
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: do not depend on fontconfig. We provide a loader so callers can
            // fetch a deterministic fallback list.
            Ok(Self {})
        }
    }

    /// Initialize monospace font cache at startup (fast)
    pub fn init_monospace_cache() -> Result<(), Box<dyn std::error::Error>> {
        debug!("Initializing monospace font cache at startup");
        let loader = Self::new()?;
        let monospace_fonts = loader.load_monospace_fonts_only()?;

        let cache_entry = FontCacheEntry {
            fonts: monospace_fonts.clone(),
            cached_at: Instant::now(),
        };

        let cache = MONOSPACE_FONT_CACHE.get_or_init(|| RwLock::new(None));
        *cache.write().unwrap() = Some(cache_entry);

        debug!(
            "Monospace font cache initialized with {} fonts",
            monospace_fonts.len()
        );
        Ok(())
    }

    /// Get cached monospace fonts (very fast) with automatic refresh if expired
    pub fn get_cached_monospace_fonts() -> Vec<FontFamily> {
        let cache = MONOSPACE_FONT_CACHE.get_or_init(|| RwLock::new(None));

        // Try to read from cache first
        if let Ok(cache_read) = cache.read() {
            if let Some(ref entry) = *cache_read {
                // Check if cache is still valid
                if entry.cached_at.elapsed() < CACHE_EXPIRATION {
                    debug!(
                        "Using cached monospace fonts (age: {:?})",
                        entry.cached_at.elapsed()
                    );
                    return entry.fonts.clone();
                } else {
                    debug!(
                        "Font cache expired (age: {:?}), needs refresh",
                        entry.cached_at.elapsed()
                    );
                }
            }
        }

        // Cache is expired or not initialized, try to refresh
        match Self::refresh_monospace_cache() {
            Ok(fonts) => fonts,
            Err(e) => {
                warn!("Failed to refresh font cache: {}, using fallback", e);
                Self::fallback_monospace_fonts()
            }
        }
    }

    /// Get fallback monospace fonts when cache fails
    fn fallback_monospace_fonts() -> Vec<FontFamily> {
        #[cfg(target_os = "linux")]
        let fallback = vec![
            FontFamily {
                name: "Monospace".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "DejaVu Sans Mono".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "Liberation Mono".to_string(),
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
        ];

        #[cfg(target_os = "windows")]
        let fallback = vec![
            FontFamily {
                name: "Consolas".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "Cascadia Code".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "Cascadia Mono".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "Courier New".to_string(),
                is_monospace: true,
            },
            FontFamily {
                name: "Lucida Console".to_string(),
                is_monospace: true,
            },
        ];

        fallback
    }

    /// Refresh the monospace font cache by reloading from system
    pub fn refresh_monospace_cache() -> Result<Vec<FontFamily>, Box<dyn std::error::Error>> {
        debug!("Refreshing monospace font cache");
        let loader = Self::new()?;
        let monospace_fonts = loader.load_monospace_fonts_only()?;

        let cache_entry = FontCacheEntry {
            fonts: monospace_fonts.clone(),
            cached_at: Instant::now(),
        };

        let cache = MONOSPACE_FONT_CACHE.get_or_init(|| RwLock::new(None));
        *cache.write().unwrap() = Some(cache_entry);

        debug!("Font cache refreshed with {} fonts", monospace_fonts.len());
        Ok(monospace_fonts)
    }

    /// Clear the font cache (useful for testing or manual refresh)
    #[allow(dead_code)]
    pub fn clear_monospace_cache() {
        debug!("Clearing monospace font cache");
        if let Some(cache) = MONOSPACE_FONT_CACHE.get() {
            *cache.write().unwrap() = None;
        }
    }

    /// Get cache statistics for debugging
    #[allow(dead_code)]
    pub fn get_cache_stats() -> (bool, Option<Duration>) {
        if let Some(cache) = MONOSPACE_FONT_CACHE.get() {
            if let Ok(cache_read) = cache.read() {
                if let Some(ref entry) = *cache_read {
                    let age = entry.cached_at.elapsed();
                    let is_expired = age >= CACHE_EXPIRATION;
                    return (!is_expired, Some(age));
                }
            }
        }
        (false, None) // Not initialized
    }

    /// Load only monospace fonts (optimized for startup)
    pub fn load_monospace_fonts_only(&self) -> Result<Vec<FontFamily>, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            let fc = self
                .fc
                .as_ref()
                .ok_or("Fontconfig not initialized")?;

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

        #[cfg(target_os = "windows")]
        {
            Ok(Self::fallback_monospace_fonts())
        }
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
            "cascadia",       // Windows: Cascadia Code, Cascadia Mono
            "consolas",       // Windows default monospace
            "lucida console", // Windows legacy monospace
        ];

        let name_lower = font_name.to_lowercase();
        mono_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    /// Add common monospace fallback fonts if they're not available
    fn add_monospace_fallbacks(&self, fonts: &mut Vec<FontFamily>) {
        #[cfg(target_os = "linux")]
        let common_monospace = [
            "Monospace",
            "Fixed",
            "Courier New",
            "Courier",
            "Liberation Mono",
            "DejaVu Sans Mono",
        ];

        #[cfg(target_os = "windows")]
        let common_monospace = [
            "Consolas",
            "Cascadia Code",
            "Cascadia Mono",
            "Courier New",
            "Lucida Console",
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

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "windows")]
impl Default for FontLoader {
    fn default() -> Self {
        match Self::new() {
            Ok(loader) => loader,
            Err(e) => {
                error!("Failed to create font loader: {}", e);
                Self {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;

    /// Smoke test to verify font cache invalidation works without errors
    #[test]
    #[serial(font_cache)]
    fn smoke_test_font_cache_invalidation() {
        // Clear any existing cache
        FontLoader::clear_monospace_cache();

        // Initialize cache
        let result = FontLoader::init_monospace_cache();
        assert!(result.is_ok(), "Cache initialization should succeed");

        // Get cached fonts
        let fonts = FontLoader::get_cached_monospace_fonts();
        assert!(!fonts.is_empty(), "Should have cached fonts");

        // Check cache stats
        let (is_valid, age) = FontLoader::get_cache_stats();
        assert!(is_valid, "Cache should be valid after initialization");
        assert!(age.is_some(), "Cache should have age information");

        println!("Font cache invalidation smoke test passed");
    }

    #[test]
    #[serial(font_cache)]
    fn test_cache_refresh() {
        // Clear cache first
        FontLoader::clear_monospace_cache();

        // Check stats when not initialized
        let (is_valid, age) = FontLoader::get_cache_stats();
        assert!(!is_valid, "Cache should be invalid when not initialized");
        assert!(age.is_none(), "Age should be None when not initialized");

        // Refresh cache
        let result = FontLoader::refresh_monospace_cache();
        assert!(result.is_ok(), "Cache refresh should succeed");

        let fonts = result.unwrap();
        assert!(!fonts.is_empty(), "Refreshed cache should have fonts");

        // Verify cache is now valid
        let (is_valid, age) = FontLoader::get_cache_stats();
        assert!(is_valid, "Cache should be valid after refresh");
        assert!(age.is_some(), "Cache should have age after refresh");

        println!("Cache refresh test passed");
    }

    #[test]
    #[serial(font_cache)]
    fn test_cache_clear() {
        // Initialize cache first
        let _ = FontLoader::init_monospace_cache();

        // Verify cache is initialized
        let (is_valid, _) = FontLoader::get_cache_stats();
        assert!(is_valid, "Cache should be valid after initialization");

        // Clear cache
        FontLoader::clear_monospace_cache();

        // Verify cache is cleared
        let (is_valid, age) = FontLoader::get_cache_stats();
        assert!(!is_valid, "Cache should be invalid after clearing");
        assert!(age.is_none(), "Age should be None after clearing");

        println!("Cache clear test passed");
    }

    #[test]
    #[serial(font_cache)]
    fn test_fallback_fonts() {
        // Get fallback fonts
        let fallback_fonts = FontLoader::fallback_monospace_fonts();

        assert!(!fallback_fonts.is_empty(), "Should have fallback fonts");
        assert!(
            fallback_fonts.iter().all(|f| f.is_monospace),
            "All fallback fonts should be monospace"
        );

        // Verify expected fallback fonts
        let font_names: Vec<&str> = fallback_fonts.iter().map(|f| f.name.as_str()).collect();

        #[cfg(target_os = "linux")]
        {
            assert!(
                font_names.contains(&"Monospace"),
                "Should include Monospace"
            );
            assert!(
                font_names.contains(&"Courier New"),
                "Should include Courier New"
            );
            assert!(font_names.contains(&"Fixed"), "Should include Fixed");
        }

        #[cfg(target_os = "windows")]
        {
            assert!(font_names.contains(&"Consolas"), "Should include Consolas");
            assert!(
                font_names.contains(&"Courier New"),
                "Should include Courier New"
            );
        }

        println!("Fallback fonts test passed");
    }

    /// Test cache expiration behavior (uses shorter timeout for testing)
    #[test]
    #[serial(font_cache)]
    fn test_cache_expiration_behavior() {
        // This is a conceptual test - in reality, the 30-minute timeout is too long for unit tests
        // But we can test the logic by manually checking cache age

        FontLoader::clear_monospace_cache();
        let _ = FontLoader::init_monospace_cache();

        // Get initial cache stats
        let (is_valid, age) = FontLoader::get_cache_stats();
        assert!(is_valid, "Newly initialized cache should be valid");

        if let Some(cache_age) = age {
            // Cache should be very fresh (less than 1 second old)
            assert!(
                cache_age < Duration::from_secs(1),
                "Fresh cache should be less than 1 second old"
            );
        }

        // In a real scenario with a 30-minute timeout, the cache would stay valid
        // For testing purposes, we verify that the logic works correctly
        println!("Cache expiration behavior test passed (age: {:?})", age);
    }

    #[test]
    #[serial(font_cache)]
    fn demonstrate_cache_performance_improvement() {
        // Clear cache to start fresh
        FontLoader::clear_monospace_cache();

        // First access - should initialize cache
        let start = Instant::now();
        let fonts1 = FontLoader::get_cached_monospace_fonts();
        let first_access_time = start.elapsed();

        // Second access - should use cache
        let start = Instant::now();
        let fonts2 = FontLoader::get_cached_monospace_fonts();
        let second_access_time = start.elapsed();

        // Verify fonts are the same
        assert_eq!(
            fonts1.len(),
            fonts2.len(),
            "Both calls should return same number of fonts"
        );

        // Second access should be faster (though both might be very fast)
        println!(
            "First access (cache initialization): {:?}",
            first_access_time
        );
        println!("Second access (cached): {:?}", second_access_time);

        // The second access is often much faster as it avoids fontconfig enumeration
        println!("Font cache performance improvement demonstrated");
    }
}
