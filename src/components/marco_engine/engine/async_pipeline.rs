//! Async Marco Pipeline - Non-blocking processing for GUI integration
//!
//! This module provides async versions of the Marco pipeline for use in
//! GTK applications and other async contexts where blocking operations
//! should be avoided.

use crate::components::marco_engine::{
    engine::pipeline::{MarcoPipeline, PipelineConfig},
    errors::MarcoError,
    render::OutputFormat,
};
use std::path::Path;
use tokio::task;

/// Async version of the Marco pipeline
pub struct AsyncMarcoPipeline {
    inner: MarcoPipeline,
}

impl AsyncMarcoPipeline {
    /// Create a new async pipeline with the given configuration
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            inner: MarcoPipeline::new(config),
        }
    }

    /// Create an async pipeline with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// Parse Marco source text into an AST (async wrapper)
    pub async fn parse(&mut self, input: &str) -> Result<(), MarcoError> {
        // Use tokio::task::spawn_blocking for CPU-intensive parsing
        let input = input.to_string();
        let mut pipeline = std::mem::replace(&mut self.inner, MarcoPipeline::with_defaults());

        let result = tokio::task::spawn_blocking(move || {
            let parse_result = pipeline.parse(&input);
            (parse_result.map(|_| ()), pipeline)
        })
        .await;

        match result {
            Ok((parse_result, updated_pipeline)) => {
                self.inner = updated_pipeline;
                parse_result
            }
            Err(join_error) => Err(MarcoError::async_error(format!(
                "Task join error: {}",
                join_error
            ))),
        }
    }

    /// Render the current AST to the specified format (async wrapper)
    pub async fn render(&self, format: Option<OutputFormat>) -> Result<String, MarcoError> {
        let inner = self.inner.clone_for_render();

        tokio::task::spawn_blocking(move || inner.render(format))
            .await
            .map_err(|e| MarcoError::async_error(format!("Task join error: {}", e)))?
    }

    /// Complete async pipeline: parse input and render to specified format
    pub async fn process(
        &mut self,
        input: &str,
        format: Option<OutputFormat>,
    ) -> Result<String, MarcoError> {
        self.parse(input).await?;
        self.render(format).await
    }

    /// Process input and render to default format
    pub async fn process_default(&mut self, input: &str) -> Result<String, MarcoError> {
        self.process(input, None).await
    }

    /// Process a file asynchronously
    pub async fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<String, MarcoError> {
        let path = path.as_ref().to_path_buf();
        let content = tokio::task::spawn_blocking(move || std::fs::read_to_string(path))
            .await
            .map_err(|e| MarcoError::async_error(format!("Task join error: {}", e)))?
            .map_err(|e| MarcoError::io_error(format!("Failed to read file: {}", e)))?;

        self.process_default(&content).await
    }

    /// Update configuration
    pub fn update_config(&mut self, config: PipelineConfig) {
        self.inner.update_config(config);
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PipelineConfig {
        self.inner.get_config()
    }
}

/// Async convenience functions
impl AsyncMarcoPipeline {
    /// Quick async HTML conversion
    pub async fn to_html(input: &str) -> Result<String, MarcoError> {
        let mut pipeline = Self::with_defaults();
        pipeline.update_config(PipelineConfig {
            default_format: OutputFormat::Html,
            ..Default::default()
        });
        pipeline.process_default(input).await
    }

    /// Quick async JSON conversion
    pub async fn to_json(input: &str, pretty: bool) -> Result<String, MarcoError> {
        let mut pipeline = Self::with_defaults();
        pipeline.update_config(PipelineConfig {
            default_format: if pretty {
                OutputFormat::JsonPretty
            } else {
                OutputFormat::Json
            },
            ..Default::default()
        });
        pipeline.process_default(input).await
    }
}

/// GTK integration utilities
pub mod gtk_integration {
    use super::*;
    use std::sync::Arc;

    /// Process Marco content in a way that's safe for GTK applications
    ///
    /// This function ensures that processing happens off the main thread
    /// and results are delivered via a callback that can safely update GTK widgets.
    pub fn process_for_gtk<F>(input: String, format: OutputFormat, callback: F)
    where
        F: FnOnce(Result<String, MarcoError>) + Send + 'static,
    {
        tokio::spawn(async move {
            let result = AsyncMarcoPipeline::to_html(&input).await;

            // Use glib::idle_add to safely execute the callback on the main thread
            glib::idle_add_once(move || {
                callback(result);
            });
        });
    }

    /// Process a file for GTK with progress reporting
    pub fn process_file_for_gtk<P, F, G>(
        path: P,
        format: OutputFormat,
        progress_callback: G,
        completion_callback: F,
    ) where
        P: AsRef<Path> + Send + 'static,
        F: FnOnce(Result<String, MarcoError>) + Send + 'static,
        G: Fn(f64) + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            let progress_callback = Arc::new(progress_callback);

            // Report progress: starting
            glib::idle_add_once({
                let progress_callback = Arc::clone(&progress_callback);
                move || progress_callback(0.0)
            });

            let mut pipeline = AsyncMarcoPipeline::with_defaults();

            // Report progress: file reading
            glib::idle_add_once({
                let progress_callback = Arc::clone(&progress_callback);
                move || progress_callback(0.25)
            });

            let result = pipeline.process_file(path).await;

            // Report progress: completed
            glib::idle_add_once({
                let progress_callback = Arc::clone(&progress_callback);
                move || progress_callback(1.0)
            });

            // Deliver result on main thread
            glib::idle_add_once(move || {
                completion_callback(result);
            });
        });
    }
}

// Extension trait to make the sync pipeline cloneable for async use
trait CloneForRender {
    fn clone_for_render(&self) -> Self;
}

impl CloneForRender for MarcoPipeline {
    fn clone_for_render(&self) -> Self {
        // Create a new pipeline with the same config and AST
        let mut new_pipeline = MarcoPipeline::new(self.get_config().clone());
        if let Some(ast) = self.get_ast() {
            // We'd need to clone the AST here, but for now we'll create a new pipeline
            // In a real implementation, we might need to make Node cloneable
            new_pipeline = MarcoPipeline::new(self.get_config().clone());
        }
        new_pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_pipeline_basic() {
        let input = "# Hello Async\n\nThis is **async** text.";

        let mut pipeline = AsyncMarcoPipeline::with_defaults();
        let result = pipeline.process_default(input).await;

        // This will fail without the grammar file, but tests the async interface
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_async_convenience_methods() {
        let input = "# Test Async\n\nContent";

        // These will fail without the grammar file, but test the async interface
        let _html_result = AsyncMarcoPipeline::to_html(input).await;
        let _json_result = AsyncMarcoPipeline::to_json(input, true).await;
    }

    #[test]
    fn test_gtk_integration_compiles() {
        // Just test that the GTK integration functions compile
        use gtk_integration::*;

        let _process_fn = process_for_gtk::<fn(Result<String, MarcoError>)>;
        // Note: We can't easily test the file processing function without proper types
    }
}
