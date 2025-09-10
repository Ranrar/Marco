//! Parallel processing utilities for Marco engine using Rayon
//!
//! This module provides parallel processing capabilities for batch operations,
//! large document processing, and performance optimization.

use crate::components::marco_engine::{
    engine::pipeline::{MarcoPipeline, PipelineConfig},
    errors::MarcoError,
    render::OutputFormat,
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for parallel processing
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of threads to use (None = use all available)
    pub max_threads: Option<usize>,
    /// Chunk size for batch processing
    pub chunk_size: usize,
    /// Enable parallel rendering (vs just parallel parsing)
    pub parallel_rendering: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: None,
            chunk_size: 100,
            parallel_rendering: true,
        }
    }
}

/// Parallel Marco pipeline for batch processing
pub struct ParallelMarcoPipeline {
    config: ParallelConfig,
    pipeline_config: PipelineConfig,
}

impl ParallelMarcoPipeline {
    /// Create a new parallel pipeline
    pub fn new(config: ParallelConfig, pipeline_config: PipelineConfig) -> Self {
        // Configure Rayon thread pool if specified
        if let Some(max_threads) = config.max_threads {
            let _ = rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build_global();
        }

        Self {
            config,
            pipeline_config,
        }
    }

    /// Create with default configurations
    pub fn with_defaults() -> Self {
        Self::new(ParallelConfig::default(), PipelineConfig::default())
    }

    /// Process multiple inputs in parallel
    pub fn process_batch(
        &self,
        inputs: Vec<String>,
        format: OutputFormat,
    ) -> Vec<Result<String, MarcoError>> {
        inputs
            .into_par_iter()
            .map(|input| {
                let mut pipeline = MarcoPipeline::new(self.pipeline_config.clone());
                pipeline.process(&input, Some(format.clone()))
            })
            .collect()
    }

    /// Process multiple files in parallel
    pub fn process_files<P: AsRef<Path> + Send + Sync>(
        &self,
        paths: Vec<P>,
        format: OutputFormat,
    ) -> Vec<(PathBuf, Result<String, MarcoError>)> {
        paths
            .into_par_iter()
            .map(|path| {
                let path_buf = path.as_ref().to_path_buf();
                let mut pipeline = MarcoPipeline::new(self.pipeline_config.clone());
                let result = pipeline.process_file(&path);
                (path_buf, result)
            })
            .collect()
    }

    /// Process a large document by splitting it into chunks
    pub fn process_large_document(
        &self,
        input: &str,
        format: OutputFormat,
    ) -> Result<String, MarcoError> {
        // Split document into logical chunks (by paragraphs/sections)
        let chunks = self.split_document(input);

        if chunks.len() == 1 {
            // Single chunk, no need for parallel processing
            let mut pipeline = MarcoPipeline::new(self.pipeline_config.clone());
            return pipeline.process(input, Some(format));
        }

        // Process chunks in parallel
        let results: Result<Vec<String>, MarcoError> = chunks
            .into_par_iter()
            .map(|chunk| {
                let mut pipeline = MarcoPipeline::new(self.pipeline_config.clone());
                pipeline.process(&chunk, Some(format.clone()))
            })
            .collect();

        match results {
            Ok(rendered_chunks) => Ok(self.merge_chunks(rendered_chunks, &format)),
            Err(e) => Err(e),
        }
    }

    /// Split a document into processable chunks
    fn split_document(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();

        if lines.len() <= self.config.chunk_size {
            return vec![input.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut in_code_block = false;

        for line in lines {
            current_chunk.push(line);

            // Track code blocks to avoid splitting them
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
            }

            // Check if we should start a new chunk
            let should_split = current_chunk.len() >= self.config.chunk_size
                && !in_code_block
                && (line.trim().is_empty() || line.starts_with('#'));

            if should_split {
                chunks.push(current_chunk.join("\n"));
                current_chunk.clear();
            }
        }

        // Add remaining content
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join("\n"));
        }

        chunks
    }

    /// Merge rendered chunks back together
    fn merge_chunks(&self, chunks: Vec<String>, format: &OutputFormat) -> String {
        match format {
            OutputFormat::Html => {
                // Wrap in a container div
                let content = chunks.join("\n");
                format!("<div class=\"marco-document\">{}</div>", content)
            }
            OutputFormat::Json | OutputFormat::JsonPretty => {
                // For JSON, we'd need to merge the AST structures
                // This is simplified - in practice we'd need proper JSON merging
                format!("{{\"chunks\": [{}]}}", chunks.join(","))
            }
        }
    }

    /// Get processing statistics
    pub fn get_stats(&self) -> ParallelStats {
        ParallelStats {
            max_threads: self.config.max_threads,
            chunk_size: self.config.chunk_size,
            current_threads: rayon::current_num_threads(),
        }
    }
}

/// Statistics about parallel processing
#[derive(Debug, Clone)]
pub struct ParallelStats {
    pub max_threads: Option<usize>,
    pub chunk_size: usize,
    pub current_threads: usize,
}

/// Batch processing utilities
pub mod batch {
    use super::*;
    use std::collections::HashMap;

    /// Process a directory of Marco files
    pub fn process_directory<P: AsRef<Path>>(
        dir_path: P,
        format: OutputFormat,
        recursive: bool,
    ) -> Result<HashMap<PathBuf, Result<String, MarcoError>>, std::io::Error> {
        let mut files = Vec::new();
        collect_marco_files(dir_path.as_ref(), &mut files, recursive)?;

        let pipeline = ParallelMarcoPipeline::with_defaults();
        let results = pipeline.process_files(files, format);

        Ok(results.into_iter().collect())
    }

    /// Collect all .marco files in a directory
    fn collect_marco_files(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        recursive: bool,
    ) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "marco" || ext == "md" {
                        files.push(path);
                    }
                }
            } else if path.is_dir() && recursive {
                collect_marco_files(&path, files, recursive)?;
            }
        }

        Ok(())
    }

    /// Performance testing utilities
    pub fn benchmark_parallel_vs_sequential(
        inputs: Vec<String>,
        format: OutputFormat,
    ) -> BenchmarkResults {
        use std::time::Instant;

        // Sequential processing
        let start = Instant::now();
        let mut sequential_results = Vec::new();
        for input in &inputs {
            let mut pipeline = MarcoPipeline::new(PipelineConfig::default());
            sequential_results.push(pipeline.process(input, Some(format.clone())));
        }
        let sequential_time = start.elapsed();

        // Parallel processing
        let start = Instant::now();
        let parallel_pipeline = ParallelMarcoPipeline::with_defaults();
        let parallel_results = parallel_pipeline.process_batch(inputs, format);
        let parallel_time = start.elapsed();

        BenchmarkResults {
            sequential_time,
            parallel_time,
            speedup: sequential_time.as_secs_f64() / parallel_time.as_secs_f64(),
            sequential_success: sequential_results.iter().filter(|r| r.is_ok()).count(),
            parallel_success: parallel_results.iter().filter(|r| r.is_ok()).count(),
            total_items: sequential_results.len(),
        }
    }

    #[derive(Debug)]
    pub struct BenchmarkResults {
        pub sequential_time: std::time::Duration,
        pub parallel_time: std::time::Duration,
        pub speedup: f64,
        pub sequential_success: usize,
        pub parallel_success: usize,
        pub total_items: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_pipeline_creation() {
        let config = ParallelConfig {
            max_threads: Some(2),
            chunk_size: 50,
            parallel_rendering: true,
        };

        let pipeline = ParallelMarcoPipeline::new(config, PipelineConfig::default());
        let stats = pipeline.get_stats();

        assert_eq!(stats.chunk_size, 50);
        assert!(stats.current_threads > 0);
    }

    #[test]
    fn test_document_splitting() {
        let pipeline = ParallelMarcoPipeline::with_defaults();

        let small_doc = "# Title\n\nSmall content";
        let chunks = pipeline.split_document(small_doc);
        assert_eq!(chunks.len(), 1);

        let large_doc = (0..200)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let chunks = pipeline.split_document(&large_doc);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_chunk_merging() {
        let pipeline = ParallelMarcoPipeline::with_defaults();

        let chunks = vec![
            "<h1>Title 1</h1>".to_string(),
            "<p>Content 1</p>".to_string(),
            "<h1>Title 2</h1>".to_string(),
        ];

        let merged = pipeline.merge_chunks(chunks, &OutputFormat::Html);
        assert!(merged.contains("marco-document"));
        assert!(merged.contains("<h1>Title 1</h1>"));
    }
}
