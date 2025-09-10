use super::profiler::{PerformanceProfiler, ProfilerConfig};
use marco::components::marco_engine::{
    engine::pipeline::MarcoPipeline,
    grammar::{MarcoParser, Rule},
    parser::marco_parser::EnhancedMarcoParser,
};
use pest::Parser;
use std::time::Instant;

/// Comprehensive benchmarking suite for Marco parsing
pub struct MarcoBenchmarks {
    profiler: PerformanceProfiler,
    parser: EnhancedMarcoParser,
    pipeline: MarcoPipeline,
}

impl MarcoBenchmarks {
    pub fn new() -> Self {
        let config = ProfilerConfig::performance_testing();
        let profiler = PerformanceProfiler::new(config);

        Self {
            profiler,
            parser: EnhancedMarcoParser::new(),
            pipeline: MarcoPipeline::with_defaults(),
        }
    }

    /// Run comprehensive benchmark suite
    pub fn run_all_benchmarks(&mut self) -> BenchmarkResults {
        println!("🚀 Starting comprehensive Marco parsing benchmarks...");

        let mut results = BenchmarkResults::new();

        // Parser benchmarks
        results.parser_results = self.benchmark_parser();

        // AST building benchmarks
        results.ast_results = self.benchmark_ast_building();

        // Pipeline benchmarks
        results.pipeline_results = self.benchmark_pipeline();

        // Memory benchmarks
        results.memory_results = self.benchmark_memory_usage();

        // Cache benchmarks
        results.cache_results = self.benchmark_cache_performance();

        // Real-world scenario benchmarks
        results.scenario_results = self.benchmark_real_world_scenarios();

        results.profiler_report = self.profiler.generate_report();

        println!("✅ Benchmark suite completed!");
        results
    }

    /// Benchmark basic parsing performance
    fn benchmark_parser(&mut self) -> ParserBenchmarkResults {
        println!("📊 Benchmarking parser performance...");

        let test_cases = self.get_parser_test_cases();
        let mut results = ParserBenchmarkResults::new();

        for (name, input) in test_cases {
            let timer = self.profiler.start_timer(&format!("parse_{}", name));

            let start = Instant::now();
            let parse_result = MarcoParser::parse(Rule::document, &input);
            let duration = start.elapsed();

            if let Some(timer) = timer {
                self.profiler.record_operation(timer);
            }

            results.add_result(name, duration, parse_result.is_ok(), input.len());
        }

        results
    }

    /// Benchmark AST building performance
    fn benchmark_ast_building(&mut self) -> AstBenchmarkResults {
        println!("🌳 Benchmarking AST building performance...");

        let test_cases = self.get_ast_test_cases();
        let mut results = AstBenchmarkResults::new();

        for (name, input) in test_cases {
            let timer = self.profiler.start_timer(&format!("ast_{}", name));

            let start = Instant::now();
            let ast_result = self.pipeline.parse(&input);
            let duration = start.elapsed();

            if let Some(timer) = timer {
                self.profiler.record_operation(timer);
            }

            results.add_result(name, duration, ast_result.is_ok(), input.len());
        }

        results
    }

    /// Benchmark full pipeline performance
    fn benchmark_pipeline(&mut self) -> PipelineBenchmarkResults {
        println!("⚡ Benchmarking pipeline performance...");

        let test_cases = self.get_pipeline_test_cases();
        let mut results = PipelineBenchmarkResults::new();

        for (name, input) in test_cases {
            let timer = self.profiler.start_timer(&format!("pipeline_{}", name));

            let start = Instant::now();
            let result = self.pipeline.process_default(&input);
            let duration = start.elapsed();

            if let Some(timer) = timer {
                self.profiler.record_operation(timer);
            }

            results.add_result(name, duration, result.is_ok(), input.len());
        }

        results
    }

    /// Benchmark memory usage patterns
    fn benchmark_memory_usage(&mut self) -> MemoryBenchmarkResults {
        println!("💾 Benchmarking memory usage...");

        let test_cases = self.get_memory_test_cases();
        let mut results = MemoryBenchmarkResults::new();

        for (name, input) in test_cases {
            // Clear any existing state
            self.pipeline.clear_cache();

            let timer = self.profiler.start_timer(&format!("memory_{}", name));

            let start_memory = self.get_memory_usage();
            let start = Instant::now();
            let _ = self.pipeline.process_default(&input);
            let duration = start.elapsed();
            let end_memory = self.get_memory_usage();

            if let Some(timer) = timer {
                self.profiler.record_operation(timer);
            }

            let memory_used = end_memory.saturating_sub(start_memory);
            results.add_result(name, duration, memory_used, input.len());
        }

        results
    }

    /// Benchmark cache performance
    fn benchmark_cache_performance(&mut self) -> CacheBenchmarkResults {
        println!("🗃️ Benchmarking cache performance...");

        let mut results = CacheBenchmarkResults::new();
        let test_input = self.get_cache_test_input();

        // Benchmark cold cache
        self.pipeline.clear_cache();
        let timer = self.profiler.start_timer("cache_cold");
        let start = Instant::now();
        let _ = self.pipeline.process_default(&test_input);
        let cold_duration = start.elapsed();
        if let Some(timer) = timer {
            self.profiler.record_operation(timer);
        }

        // Benchmark warm cache (repeated operations)
        let timer = self.profiler.start_timer("cache_warm");
        let start = Instant::now();
        for _ in 0..10 {
            let _ = self.pipeline.process_default(&test_input);
        }
        let warm_duration = start.elapsed() / 10;
        if let Some(timer) = timer {
            self.profiler.record_operation(timer);
        }

        results.cold_cache_time = cold_duration;
        results.warm_cache_time = warm_duration;
        results.cache_speedup = cold_duration.as_nanos() as f64 / warm_duration.as_nanos() as f64;
        results.cache_stats = self.pipeline.parser_cache_stats();

        results
    }

    /// Benchmark real-world scenarios
    fn benchmark_real_world_scenarios(&mut self) -> ScenarioBenchmarkResults {
        println!("🌍 Benchmarking real-world scenarios...");

        let scenarios = self.get_real_world_scenarios();
        let mut results = ScenarioBenchmarkResults::new();

        for (name, input) in scenarios {
            let timer = self.profiler.start_timer(&format!("scenario_{}", name));

            let start = Instant::now();
            let result = self.pipeline.process_default(&input);
            let duration = start.elapsed();

            if let Some(timer) = timer {
                self.profiler.record_operation(timer);
            }

            results.add_result(name, duration, result.is_ok(), input.len());
        }

        results
    }

    // Test case generators
    fn get_parser_test_cases(&self) -> Vec<(String, String)> {
        vec![
            ("simple_text".to_string(), "Hello, world!".to_string()),
            (
                "heading".to_string(),
                "# Main Title\n## Subtitle".to_string(),
            ),
            (
                "list".to_string(),
                "- Item 1\n- Item 2\n- Item 3".to_string(),
            ),
            (
                "code_block".to_string(),
                "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```".to_string(),
            ),
            (
                "complex_formatting".to_string(),
                "**Bold** *italic* ~~strikethrough~~ `code`".to_string(),
            ),
            (
                "table".to_string(),
                "| A | B |\n|---|---|\n| 1 | 2 |".to_string(),
            ),
            (
                "admonition".to_string(),
                ":::\nnote\nThis is important\n:::".to_string(),
            ),
            ("math".to_string(), "$$\\sum_{i=1}^{n} x_i$$".to_string()),
        ]
    }

    fn get_ast_test_cases(&self) -> Vec<(String, String)> {
        vec![
            (
                "nested_lists".to_string(),
                "- Top level\n  - Nested\n    - Double nested\n- Back to top".to_string(),
            ),
            (
                "mixed_content".to_string(),
                "# Title\n\nParagraph with **bold** and *italic*.\n\n```code\nblock\n```"
                    .to_string(),
            ),
            ("large_document".to_string(), self.generate_large_document()),
            (
                "deeply_nested".to_string(),
                self.generate_deeply_nested_content(),
            ),
        ]
    }

    fn get_pipeline_test_cases(&self) -> Vec<(String, String)> {
        vec![
            (
                "full_pipeline_small".to_string(),
                self.generate_small_document(),
            ),
            (
                "full_pipeline_medium".to_string(),
                self.generate_medium_document(),
            ),
            (
                "full_pipeline_large".to_string(),
                self.generate_large_document(),
            ),
        ]
    }

    fn get_memory_test_cases(&self) -> Vec<(String, String)> {
        vec![
            (
                "memory_stress".to_string(),
                self.generate_memory_stress_test(),
            ),
            ("large_table".to_string(), self.generate_large_table()),
            ("many_headings".to_string(), self.generate_many_headings()),
        ]
    }

    fn get_cache_test_input(&self) -> String {
        "# Repeated Content\n\n**Bold text** and *italic text*.\n\n```rust\nlet x = 42;\n```"
            .to_string()
    }

    fn get_real_world_scenarios(&self) -> Vec<(String, String)> {
        vec![
            ("blog_post".to_string(), self.generate_blog_post()),
            (
                "technical_documentation".to_string(),
                self.generate_technical_docs(),
            ),
            ("user_manual".to_string(), self.generate_user_manual()),
            ("api_documentation".to_string(), self.generate_api_docs()),
        ]
    }

    // Document generators
    fn generate_large_document(&self) -> String {
        let mut content = String::new();
        content.push_str("# Large Document Test\n\n");

        for i in 1..=50 {
            content.push_str(&format!("## Section {}\n\n", i));
            content.push_str("This is a paragraph with **bold** and *italic* text. ");
            content.push_str("It contains multiple sentences to test parsing performance.\n\n");

            content.push_str("- List item 1\n");
            content.push_str("- List item 2\n");
            content.push_str("- List item 3\n\n");

            if i % 10 == 0 {
                content.push_str(
                    "```rust\nfn example() {\n    println!(\"Code block {}\");\n}\n```\n\n",
                );
            }
        }

        content
    }

    fn generate_deeply_nested_content(&self) -> String {
        let mut content = String::new();

        for level in 1..=10 {
            let indent = "  ".repeat(level - 1);
            content.push_str(&format!("{}# Heading Level {}\n\n", indent, level));
            content.push_str(&format!("{}Paragraph at level {}.\n\n", indent, level));

            content.push_str(&format!("{}- List at level {}\n", indent, level));
            content.push_str(&format!("{}  - Nested item\n", indent));
            content.push_str(&format!("{}    - Double nested\n\n", indent));
        }

        content
    }

    fn generate_small_document(&self) -> String {
        "# Small Document\n\nJust a **simple** test.".to_string()
    }

    fn generate_medium_document(&self) -> String {
        let mut content = String::new();
        content.push_str("# Medium Document\n\n");
        content.push_str("This is a medium-sized document for testing.\n\n");
        content.push_str("## Features\n\n");
        content.push_str("- **Bold text**\n");
        content.push_str("- *Italic text*\n");
        content.push_str("- `Code snippets`\n\n");
        content.push_str("```rust\nfn main() {\n    println!(\"Hello, Marco!\");\n}\n```\n\n");
        content.push_str("That's it!\n");
        content
    }

    fn generate_memory_stress_test(&self) -> String {
        let mut content = String::new();

        for i in 0..1000 {
            content.push_str(&format!(
                "Line {} with some content to stress test memory usage.\n",
                i
            ));
        }

        content
    }

    fn generate_large_table(&self) -> String {
        let mut content = String::new();
        content.push_str("| A | B | C | D | E |\n");
        content.push_str("|---|---|---|---|---|\n");

        for i in 0..100 {
            content.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                i,
                i * 2,
                i * 3,
                i * 4,
                i * 5
            ));
        }

        content
    }

    fn generate_many_headings(&self) -> String {
        let mut content = String::new();

        for i in 1..=200 {
            let level = (i % 6) + 1;
            let prefix = "#".repeat(level);
            content.push_str(&format!(
                "{} Heading {}\n\nContent for heading {}.\n\n",
                prefix, i, i
            ));
        }

        content
    }

    fn generate_blog_post(&self) -> String {
        concat!(
            "# My Blog Post\n\n",
            "Posted on March 15, 2024\n\n",
            "## Introduction\n\n",
            "Welcome to my blog! Today I want to talk about **performance optimization** in parsing.\n\n",
            "## The Problem\n\n",
            "When dealing with large documents, parsing can become a bottleneck. Here are some common issues:\n\n",
            "- *Slow regex patterns*\n",
            "- **Memory allocations**\n",
            "- Cache misses\n\n",
            "## The Solution\n\n",
            "```rust\n",
            "fn optimize_parser() {\n",
            "    // Use efficient data structures\n",
            "    let mut cache = HashMap::new();\n",
            "    \n",
            "    // Implement caching\n",
            "    cache.insert(\"key\", \"value\");\n",
            "}\n",
            "```\n\n",
            "### Performance Tips\n\n",
            "1. Use string interning\n",
            "2. Implement result caching\n",
            "3. Avoid unnecessary allocations\n\n",
            "## Conclusion\n\n",
            "With these optimizations, parsing becomes much faster!\n\n",
            "> Remember: premature optimization is the root of all evil, but knowing where to optimize is key.\n"
        ).to_string()
    }

    fn generate_technical_docs(&self) -> String {
        concat!(
            "# API Documentation\n\n",
            "## Overview\n\n",
            "This API provides access to the Marco parsing engine.\n\n",
            "### Base URL\n",
            "```\n",
            "https://api.marco.com/v1\n",
            "```\n\n",
            "## Endpoints\n\n",
            "### Parse Document\n\n",
            "**POST** `/parse`\n\n",
            "Parse a Marco document and return the AST.\n\n",
            "#### Request Body\n\n",
            "| Field | Type | Description |\n",
            "|-------|------|-------------|\n",
            "| content | string | The Marco content to parse |\n",
            "| options | object | Parse options |\n\n",
            "#### Response\n\n",
            "```json\n",
            "{\n",
            "  \"ast\": { ... },\n",
            "  \"metadata\": {\n",
            "    \"parse_time\": \"10ms\",\n",
            "    \"node_count\": 42\n",
            "  }\n",
            "}\n",
            "```\n\n",
            "### Cache Management\n\n",
            "**DELETE** `/cache`\n\n",
            "Clear the parser cache.\n\n",
            "## Error Codes\n\n",
            "- `400` - Invalid input\n",
            "- `500` - Internal error\n"
        )
        .to_string()
    }

    fn generate_user_manual(&self) -> String {
        concat!(
            "# Marco User Manual\n\n",
            "## Getting Started\n\n",
            "Welcome to Marco! This guide will help you get started.\n\n",
            "### Installation\n\n",
            "```bash\n",
            "cargo install marco\n",
            "```\n\n",
            "### Basic Usage\n\n",
            "1. Create a new file: `document.marco`\n",
            "2. Write your content\n",
            "3. Run: `marco document.marco`\n\n",
            "## Features\n\n",
            "### Text Formatting\n\n",
            "- **Bold**: `**text**`\n",
            "- *Italic*: `*text*`\n",
            "- ~~Strikethrough~~: `~~text~~`\n\n",
            "## Troubleshooting\n\n",
            "If you encounter issues:\n\n",
            "1. Check the syntax\n",
            "2. Validate your input\n",
            "3. Check the error messages\n\n"
        )
        .to_string()
    }

    fn generate_api_docs(&self) -> String {
        concat!(
            "# Marco Engine API\n\n",
            "## Classes\n\n",
            "### MarcoParser\n\n",
            "The main parser class for processing Marco documents.\n\n",
            "#### Constructor\n\n",
            "```rust\n",
            "pub fn new() -> Self\n",
            "```\n\n",
            "Creates a new parser instance.\n\n",
            "#### Methods\n\n",
            "##### parse()\n\n",
            "```rust\n",
            "pub fn parse(&mut self, input: &str) -> Result<Node, MarcoError>\n",
            "```\n\n",
            "Parse input text and return an AST.\n\n",
            "**Parameters:**\n",
            "- `input` - The text to parse\n\n",
            "**Returns:**\n",
            "- `Result<Node, MarcoError>` - The parsed AST or error\n\n",
            "**Example:**\n\n",
            "```rust\n",
            "let mut parser = MarcoParser::new();\n",
            "let ast = parser.parse(\"# Hello World\");\n",
            "```\n\n",
            "## Performance Notes\n\n",
            "- Use caching for repeated parsing\n",
            "- Consider using streaming for large documents\n",
            "- Enable parallel processing for multiple files\n"
        )
        .to_string()
    }

    fn get_memory_usage(&self) -> usize {
        // Placeholder - would use actual memory profiling in real implementation
        0
    }
}

// Result structures
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub parser_results: ParserBenchmarkResults,
    pub ast_results: AstBenchmarkResults,
    pub pipeline_results: PipelineBenchmarkResults,
    pub memory_results: MemoryBenchmarkResults,
    pub cache_results: CacheBenchmarkResults,
    pub scenario_results: ScenarioBenchmarkResults,
    pub profiler_report: super::profiler::PerformanceReport,
}

impl BenchmarkResults {
    pub fn new() -> Self {
        Self {
            parser_results: ParserBenchmarkResults::new(),
            ast_results: AstBenchmarkResults::new(),
            pipeline_results: PipelineBenchmarkResults::new(),
            memory_results: MemoryBenchmarkResults::new(),
            cache_results: CacheBenchmarkResults::new(),
            scenario_results: ScenarioBenchmarkResults::new(),
            profiler_report: super::profiler::PerformanceReport {
                profiling_duration: std::time::Duration::from_secs(0),
                operations: Vec::new(),
                cache_metrics: super::profiler::CacheMetrics {
                    hits: 0,
                    misses: 0,
                    evictions: 0,
                    hit_rate: 0.0,
                    avg_lookup_time: std::time::Duration::from_nanos(0),
                    memory_used: 0,
                    max_memory: 0,
                },
                rule_metrics: Vec::new(),
                top_bottlenecks: Vec::new(),
                memory_summary: super::profiler::MemorySummary {
                    peak_usage: 0,
                    current_usage: 0,
                    total_allocations: 0,
                    avg_allocation_size: 0,
                },
            },
        }
    }

    /// Generate a comprehensive markdown report
    pub fn generate_markdown_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Marco Performance Benchmark Report\n\n");
        report.push_str(&format!(
            "Generated on: {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!(
            "- **Parser Tests**: {} operations\n",
            self.parser_results.results.len()
        ));
        report.push_str(&format!(
            "- **AST Tests**: {} operations\n",
            self.ast_results.results.len()
        ));
        report.push_str(&format!(
            "- **Pipeline Tests**: {} operations\n",
            self.pipeline_results.results.len()
        ));
        report.push_str(&format!(
            "- **Total Profiling Duration**: {:?}\n\n",
            self.profiler_report.profiling_duration
        ));

        // Parser Results
        report.push_str("## Parser Performance\n\n");
        report.push_str("| Test Case | Duration (ms) | Success | Input Size |\n");
        report.push_str("|-----------|---------------|---------|------------|\n");
        for result in &self.parser_results.results {
            report.push_str(&format!(
                "| {} | {:.3} | {} | {} |\n",
                result.name,
                result.duration.as_secs_f64() * 1000.0,
                if result.success { "✅" } else { "❌" },
                result.input_size
            ));
        }
        report.push_str("\n");

        // Cache Performance
        report.push_str("## Cache Performance\n\n");
        report.push_str(&format!(
            "- **Cold Cache**: {:.3}ms\n",
            self.cache_results.cold_cache_time.as_secs_f64() * 1000.0
        ));
        report.push_str(&format!(
            "- **Warm Cache**: {:.3}ms\n",
            self.cache_results.warm_cache_time.as_secs_f64() * 1000.0
        ));
        report.push_str(&format!(
            "- **Speedup**: {:.2}x\n\n",
            self.cache_results.cache_speedup
        ));

        // Bottlenecks
        if !self.profiler_report.top_bottlenecks.is_empty() {
            report.push_str("## Performance Bottlenecks\n\n");
            for bottleneck in &self.profiler_report.top_bottlenecks {
                report.push_str(&format!(
                    "### {} ({})\n\n",
                    bottleneck.operation, bottleneck.severity
                ));
                report.push_str(&format!("**Issue**: {}\n\n", bottleneck.issue_type));
                report.push_str(&format!("**Description**: {}\n\n", bottleneck.description));
                report.push_str(&format!(
                    "**Recommendation**: {}\n\n",
                    bottleneck.recommendation
                ));
            }
        }

        report
    }
}

#[derive(Debug, Clone)]
pub struct ParserBenchmarkResults {
    pub results: Vec<SingleBenchmarkResult>,
}

impl ParserBenchmarkResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(
        &mut self,
        name: String,
        duration: std::time::Duration,
        success: bool,
        input_size: usize,
    ) {
        self.results.push(SingleBenchmarkResult {
            name,
            duration,
            success,
            input_size,
        });
    }
}

#[derive(Debug, Clone)]
pub struct SingleBenchmarkResult {
    pub name: String,
    pub duration: std::time::Duration,
    pub success: bool,
    pub input_size: usize,
}

#[derive(Debug, Clone)]
pub struct AstBenchmarkResults {
    pub results: Vec<SingleBenchmarkResult>,
}

impl AstBenchmarkResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(
        &mut self,
        name: String,
        duration: std::time::Duration,
        success: bool,
        input_size: usize,
    ) {
        self.results.push(SingleBenchmarkResult {
            name,
            duration,
            success,
            input_size,
        });
    }
}

#[derive(Debug, Clone)]
pub struct PipelineBenchmarkResults {
    pub results: Vec<SingleBenchmarkResult>,
}

impl PipelineBenchmarkResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(
        &mut self,
        name: String,
        duration: std::time::Duration,
        success: bool,
        input_size: usize,
    ) {
        self.results.push(SingleBenchmarkResult {
            name,
            duration,
            success,
            input_size,
        });
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResults {
    pub results: Vec<MemoryBenchmarkResult>,
}

impl MemoryBenchmarkResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(
        &mut self,
        name: String,
        duration: std::time::Duration,
        memory_used: usize,
        input_size: usize,
    ) {
        self.results.push(MemoryBenchmarkResult {
            name,
            duration,
            memory_used,
            input_size,
        });
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResult {
    pub name: String,
    pub duration: std::time::Duration,
    pub memory_used: usize,
    pub input_size: usize,
}

#[derive(Debug, Clone)]
pub struct CacheBenchmarkResults {
    pub cold_cache_time: std::time::Duration,
    pub warm_cache_time: std::time::Duration,
    pub cache_speedup: f64,
    pub cache_stats: marco::components::marco_engine::parser::CacheStats,
}

impl CacheBenchmarkResults {
    pub fn new() -> Self {
        Self {
            cold_cache_time: std::time::Duration::from_nanos(0),
            warm_cache_time: std::time::Duration::from_nanos(0),
            cache_speedup: 1.0,
            cache_stats: marco::components::marco_engine::parser::CacheStats {
                size: 0,
                max_size: 0,
                enabled: false,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioBenchmarkResults {
    pub results: Vec<SingleBenchmarkResult>,
}

impl ScenarioBenchmarkResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(
        &mut self,
        name: String,
        duration: std::time::Duration,
        success: bool,
        input_size: usize,
    ) {
        self.results.push(SingleBenchmarkResult {
            name,
            duration,
            success,
            input_size,
        });
    }
}
