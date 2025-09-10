//! Marco Engine Configuration Integration
//!
//! This module provides integration between the Marco engine configuration
//! and the main application settings system. It bridges the engine-specific
//! configuration with the RON-based settings infrastructure.

use super::pipeline::PipelineConfig;
use crate::components::marco_engine::{
    parser::ParserConfig,
    render::{HtmlOptions, OutputFormat},
};
use crate::logic::swanson::{
    EngineParserSettings, EnginePerformanceSettings, EngineSettings, Settings,
};

/// Configuration manager that bridges Marco engine with application settings
pub struct EngineConfig {
    /// Main application settings (contains engine settings)
    settings: Settings,
}

impl EngineConfig {
    /// Create a new configuration manager from application settings
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Get engine settings from main settings, creating defaults if needed
    pub fn engine_settings(&self) -> EngineSettings {
        self.settings.engine.clone().unwrap_or_default()
    }

    /// Update engine settings in main settings
    pub fn update_engine_settings<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut EngineSettings),
    {
        let mut engine_settings = self.settings.engine.clone().unwrap_or_default();
        updater(&mut engine_settings);
        self.settings.engine = Some(engine_settings);
    }

    /// Get current application settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable reference to application settings
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Convert to PipelineConfig for the Marco engine
    pub fn to_pipeline_config(&self) -> PipelineConfig {
        let engine_settings = self.engine_settings();

        let debug = engine_settings
            .performance
            .as_ref()
            .and_then(|p| p.debug_mode)
            .unwrap_or(false);

        let cache_ast = engine_settings
            .performance
            .as_ref()
            .and_then(|p| p.cache_ast)
            .unwrap_or(true);

        // Parallel processing is always enabled for optimal performance
        let parallel = true;

        let default_format_str = engine_settings
            .render
            .as_ref()
            .and_then(|r| r.default_format.clone())
            .unwrap_or_else(|| "html".to_string());

        let default_format = match default_format_str.as_str() {
            "html" => OutputFormat::Html,
            "json" => OutputFormat::Json,
            _ => OutputFormat::Html,
        };

        // Build HTML options
        let html_options = if let Some(render) = engine_settings.render.as_ref() {
            if let Some(html) = render.html.as_ref() {
                HtmlOptions {
                    class_prefix: "marco-".to_string(),
                    syntax_highlighting: html.syntax_highlighting.unwrap_or(false),
                    sanitize_html: true,
                    youtube_embed: true,
                    auto_links: true,
                }
            } else {
                HtmlOptions::default()
            }
        } else {
            HtmlOptions::default()
        };

        PipelineConfig {
            debug,
            default_format,
            html_options,
            cache_ast,
            parallel,
            parser_config: self.to_parser_config(),
        }
    }

    /// Convert to ParserConfig for the Marco parser
    pub fn to_parser_config(&self) -> ParserConfig {
        let engine_settings = self.engine_settings();
        let parser_settings = engine_settings.parser.as_ref();

        let track_positions = parser_settings
            .and_then(|p| p.track_positions)
            .unwrap_or(true);

        let enable_cache = parser_settings.and_then(|p| p.enable_cache).unwrap_or(true);

        let max_cache_size = parser_settings
            .and_then(|p| p.max_cache_size)
            .unwrap_or(1000);

        let detailed_errors = parser_settings
            .and_then(|p| p.detailed_errors)
            .unwrap_or(true);

        let collect_stats = parser_settings
            .and_then(|p| p.collect_stats)
            .unwrap_or(false);

        ParserConfig {
            track_positions,
            enable_cache,
            max_cache_size,
            detailed_errors,
            collect_stats,
        }
    }

    /// Get maximum cache size from performance settings
    pub fn max_cache_size(&self) -> usize {
        let engine_settings = self.engine_settings();
        let perf_settings = engine_settings.performance.as_ref();
        perf_settings
            .and_then(|p| p.max_cache_memory_mb)
            .map(|mb| mb * 1024 * 1024) // Convert MB to bytes
            .unwrap_or(64 * 1024 * 1024) // Default: 64MB
    }

    /// Check if debug mode is enabled
    pub fn is_debug_enabled(&self) -> bool {
        let engine_settings = self.engine_settings();
        engine_settings
            .performance
            .as_ref()
            .and_then(|p| p.debug_mode)
            .unwrap_or(false)
    }

    /// Check if parallel processing is enabled (always true)
    pub fn is_parallel_enabled(&self) -> bool {
        // Parallel processing is always enabled for optimal performance
        true
    }

    /// Get number of worker threads
    pub fn worker_threads(&self) -> usize {
        let engine_settings = self.engine_settings();
        engine_settings
            .performance
            .as_ref()
            .and_then(|p| p.worker_threads)
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4)
            })
    }

    /// Check if syntax highlighting is enabled
    pub fn is_syntax_highlighting_enabled(&self) -> bool {
        let engine_settings = self.engine_settings();
        engine_settings
            .render
            .as_ref()
            .and_then(|r| r.html.as_ref())
            .and_then(|h| h.syntax_highlighting)
            .unwrap_or(false)
    }

    /// Enable debug mode
    pub fn enable_debug(&mut self) {
        self.update_engine_settings(|engine_settings| {
            if engine_settings.performance.is_none() {
                engine_settings.performance = Some(EnginePerformanceSettings::default());
            }
            if let Some(ref mut perf) = engine_settings.performance {
                perf.debug_mode = Some(true);
            }
        });
    }

    /// Enable caching with specified size
    pub fn enable_cache(&mut self, max_size: Option<usize>) {
        self.update_engine_settings(|engine_settings| {
            if engine_settings.parser.is_none() {
                engine_settings.parser = Some(EngineParserSettings::default());
            }
            if let Some(ref mut parser) = engine_settings.parser {
                parser.enable_cache = Some(true);
                if let Some(size) = max_size {
                    parser.max_cache_size = Some(size);
                }
            }

            if engine_settings.performance.is_none() {
                engine_settings.performance = Some(EnginePerformanceSettings::default());
            }
            if let Some(ref mut perf) = engine_settings.performance {
                perf.cache_ast = Some(true);
            }
        });
    }

    /// Create configuration optimized for development
    pub fn development() -> Self {
        let mut settings = Settings::default();

        // Create engine settings for development
        let engine_settings = EngineSettings {
            parser: Some(EngineParserSettings {
                track_positions: Some(true),
                enable_cache: Some(false), // Disable cache for development
                max_cache_size: Some(100),
                detailed_errors: Some(true),
                collect_stats: Some(true),
            }),
            performance: Some(EnginePerformanceSettings {
                worker_threads: Some(1), // Single thread for debugging consistency
                cache_ast: Some(false),
                max_cache_memory_mb: Some(16),
                debug_mode: Some(true),
            }),
            ..Default::default()
        };

        settings.engine = Some(engine_settings);
        Self { settings }
    }

    /// Create configuration optimized for production
    pub fn production() -> Self {
        let mut settings = Settings::default();

        // Create engine settings for production
        let engine_settings = EngineSettings {
            parser: Some(EngineParserSettings {
                track_positions: Some(false), // Disable for performance
                enable_cache: Some(true),
                max_cache_size: Some(10000),
                detailed_errors: Some(false),
                collect_stats: Some(false),
            }),
            performance: Some(EnginePerformanceSettings {
                worker_threads: None, // Use all available cores for optimal performance
                cache_ast: Some(true),
                max_cache_memory_mb: Some(256),
                debug_mode: Some(false),
            }),
            ..Default::default()
        };

        settings.engine = Some(engine_settings);
        Self { settings }
    }

    /// Create configuration optimized for testing
    pub fn testing() -> Self {
        let mut settings = Settings::default();

        // Create engine settings for testing
        let engine_settings = EngineSettings {
            parser: Some(EngineParserSettings {
                track_positions: Some(true),
                enable_cache: Some(true),
                max_cache_size: Some(500),
                detailed_errors: Some(true),
                collect_stats: Some(true),
            }),
            performance: Some(EnginePerformanceSettings {
                worker_threads: Some(1), // Single thread for deterministic testing
                cache_ast: Some(true),
                max_cache_memory_mb: Some(32),
                debug_mode: Some(true),
            }),
            ..Default::default()
        };

        settings.engine = Some(engine_settings);
        Self { settings }
    }

    /// Get optimized configuration for performance
    pub fn get_performance_config(&self) -> PipelineConfig {
        let mut config = self.to_pipeline_config();

        // Override for performance
        config.debug = false;
        config.cache_ast = true;
        config.parallel = true;
        config.parser_config.enable_cache = true;
        config.parser_config.max_cache_size = 500;
        config.parser_config.track_positions = false;
        config.parser_config.detailed_errors = false;
        config.parser_config.collect_stats = false;

        config
    }

    /// Get configuration optimized for debugging
    pub fn get_debug_config(&self) -> PipelineConfig {
        let mut config = self.to_pipeline_config();

        // Override for debugging
        config.debug = true;
        config.parallel = false;
        config.parser_config.track_positions = true;
        config.parser_config.detailed_errors = true;
        config.parser_config.collect_stats = true;

        config
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let engine_settings = self.engine_settings();

        // Validate parser cache size
        if let Some(parser) = &engine_settings.parser {
            if let Some(cache_size) = parser.max_cache_size {
                if cache_size > 10000 {
                    errors.push("Parser cache size too large (max 10000)".to_string());
                }
            }
        }

        // Validate parallel worker threads
        if let Some(perf) = &engine_settings.performance {
            if let Some(threads) = perf.worker_threads {
                if threads == 0 {
                    errors.push("Worker threads must be greater than 0".to_string());
                }
                if threads > 64 {
                    errors.push("Too many worker threads (max 64)".to_string());
                }
            }
        }

        // Validate text wrap width
        if let Some(render) = &engine_settings.render {
            if let Some(text) = &render.text {
                if let Some(wrap_width) = text.wrap_width {
                    if wrap_width > 0 && wrap_width < 20 {
                        errors.push("Text wrap width too small (min 20 if enabled)".to_string());
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::new(Settings::default());
        let pipeline_config = config.to_pipeline_config();

        assert!(!pipeline_config.debug);
        assert_eq!(pipeline_config.default_format, OutputFormat::Html);
        assert!(pipeline_config.parser_config.track_positions);
        assert!(pipeline_config.parser_config.enable_cache);
    }

    #[test]
    fn test_engine_settings_access() {
        let config = EngineConfig::new(Settings::default());
        let engine_settings = config.engine_settings();

        // Should return default values when not configured
        assert!(engine_settings.parser.is_none());
        assert!(engine_settings.performance.is_none());
        assert!(engine_settings.render.is_none());
    }

    #[test]
    fn test_settings_update() {
        let mut config = EngineConfig::new(Settings::default());

        config.update_engine_settings(|settings| {
            settings.parser = Some(EngineParserSettings {
                track_positions: Some(false),
                enable_cache: Some(true),
                max_cache_size: Some(500),
                detailed_errors: Some(true),
                collect_stats: Some(false),
            });
        });

        let engine_settings = config.engine_settings();
        assert!(engine_settings.parser.is_some());
        let parser = engine_settings.parser.unwrap();
        assert_eq!(parser.track_positions, Some(false));
        assert_eq!(parser.enable_cache, Some(true));
        assert_eq!(parser.max_cache_size, Some(500));
    }
}
