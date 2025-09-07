//! Marco Engine - Core orchestration and pipeline management
//!
//! This module provides the main engine components that coordinate parsing,
//! AST building, and rendering operations with support for async processing
//! and parallel execution.

pub mod async_pipeline;
pub mod parallel;
pub mod pipeline;

pub use crate::components::marco_engine::grammar::MarcoParser;
pub use async_pipeline::{gtk_integration, AsyncMarcoPipeline};
pub use parallel::{ParallelConfig, ParallelMarcoPipeline, ParallelStats};
pub use pipeline::{MarcoPipeline, PipelineConfig};

use crate::components::marco_engine::{errors::MarcoError, render::OutputFormat};

/// Main Marco Engine - provides unified access to all pipeline variants
pub struct MarcoEngine;

impl MarcoEngine {
    /// Quick HTML conversion using default pipeline
    pub fn to_html(input: &str) -> Result<String, MarcoError> {
        MarcoPipeline::to_html(input)
    }

    /// Quick text conversion using default pipeline
    pub fn to_text(input: &str) -> Result<String, MarcoError> {
        MarcoPipeline::to_text(input)
    }

    /// Quick JSON conversion using default pipeline
    pub fn to_json(input: &str, pretty: bool) -> Result<String, MarcoError> {
        MarcoPipeline::to_json(input, pretty)
    }

    /// Async HTML conversion
    pub async fn to_html_async(input: &str) -> Result<String, MarcoError> {
        AsyncMarcoPipeline::to_html(input).await
    }

    /// Async text conversion
    pub async fn to_text_async(input: &str) -> Result<String, MarcoError> {
        AsyncMarcoPipeline::to_text(input).await
    }

    /// Async JSON conversion
    pub async fn to_json_async(input: &str, pretty: bool) -> Result<String, MarcoError> {
        AsyncMarcoPipeline::to_json(input, pretty).await
    }

    /// Batch processing with parallel pipeline
    pub fn process_batch(
        inputs: Vec<String>,
        format: OutputFormat,
    ) -> Vec<Result<String, MarcoError>> {
        let pipeline = ParallelMarcoPipeline::with_defaults();
        pipeline.process_batch(inputs, format)
    }

    /// Get engine information
    pub fn info() -> EngineInfo {
        EngineInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            features: vec![
                "pest-parser".to_string(),
                "enhanced-parser".to_string(),
                "position-tracking".to_string(),
                "parse-caching".to_string(),
                "rule-analysis".to_string(),
                "async-processing".to_string(),
                "parallel-processing".to_string(),
                "html-rendering".to_string(),
                "text-rendering".to_string(),
                "json-rendering".to_string(),
                "gtk-integration".to_string(),
            ],
            supported_formats: vec![
                OutputFormat::Html,
                OutputFormat::Text,
                OutputFormat::Json,
                OutputFormat::JsonPretty,
            ],
        }
    }

    /// Validate syntax with a specific rule
    pub fn validate_syntax(rule_name: &str, input: &str) -> Result<bool, MarcoError> {
        let mut pipeline = MarcoPipeline::with_defaults();
        pipeline.validate_syntax(rule_name, input)
    }

    /// Analyze rule usage in document
    pub fn analyze_document(input: &str) -> Result<crate::components::marco_engine::parser::RuleAnalysis, MarcoError> {
        let mut pipeline = MarcoPipeline::with_defaults();
        pipeline.analyze_rule_usage(input)
    }

    /// Create a pipeline with enhanced parser features enabled
    pub fn create_enhanced_pipeline() -> MarcoPipeline {
        let config = pipeline::PipelineConfig {
            parser_config: crate::components::marco_engine::parser::ParserConfig {
                track_positions: true,
                enable_cache: true,
                detailed_errors: true,
                collect_stats: true,
                ..Default::default()
            },
            debug: false,
            cache_ast: true,
            ..Default::default()
        };
        MarcoPipeline::new(config)
    }
}

/// Information about the Marco engine
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub version: String,
    pub features: Vec<String>,
    pub supported_formats: Vec<OutputFormat>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_info() {
        let info = MarcoEngine::info();
        assert!(!info.version.is_empty());
        assert!(info.features.contains(&"async-processing".to_string()));
        assert!(info.supported_formats.contains(&OutputFormat::Html));
    }

    #[test]
    fn test_engine_convenience_methods() {
        let input = "# Test\n\nContent";

        // These will fail without the grammar file, but test the interface
        let _html_result = MarcoEngine::to_html(input);
        let _text_result = MarcoEngine::to_text(input);
        let _json_result = MarcoEngine::to_json(input, true);
    }

    #[tokio::test]
    async fn test_engine_async_methods() {
        let input = "# Test Async\n\nContent";

        // These will fail without the grammar file, but test the async interface
        let _html_result = MarcoEngine::to_html_async(input).await;
        let _text_result = MarcoEngine::to_text_async(input).await;
        let _json_result = MarcoEngine::to_json_async(input, true).await;
    }

    #[test]
    fn test_batch_processing() {
        let inputs = vec!["# Doc 1".to_string(), "# Doc 2".to_string()];

        let results = MarcoEngine::process_batch(inputs, OutputFormat::Html);
        assert_eq!(results.len(), 2);
    }
}
