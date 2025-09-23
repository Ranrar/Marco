//! Interactive baseline management
//!
//! This module provides interactive functionality for creating and managing
//! test baselines, allowing users to approve new test cases and update existing ones.

use crate::spec::{TestCase, TestSpec, TestResult};
use crate::runner::TestRunner;
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Interactive baseline manager
pub struct InteractiveManager {
    runner: TestRunner,
}

impl InteractiveManager {
    /// Create a new interactive manager
    pub fn new(runner: TestRunner) -> Self {
        Self { runner }
    }
    
    /// Run the interactive session
    pub fn run(&self) -> Result<()> {
        println!("{}", "Marco Test Suite - Interactive Mode".blue().bold());
        println!("{}", "=====================================".blue());
        println!();
        
        loop {
            self.show_menu();
            
            match self.get_user_choice()? {
                1 => self.add_new_test()?,
                2 => self.test_markdown_string()?,
                3 => self.review_failing_tests()?,
                4 => self.update_baselines()?,
                5 => self.browse_specifications()?,
                6 => {
                    println!("{}", "Goodbye!".green());
                    break;
                }
                _ => {
                    println!("{}", "Invalid choice. Please try again.".red());
                }
            }
            println!();
        }
        
        Ok(())
    }
    
    /// Show the main menu
    fn show_menu(&self) {
        println!("{}", "Choose an option:".cyan().bold());
        println!("1. Add new test case");
        println!("2. Test markdown string");
        println!("3. Review failing tests");
        println!("4. Update test baselines");
        println!("5. Browse specifications");
        println!("6. Exit");
        print!("Enter your choice (1-6): ");
        io::stdout().flush().unwrap();
    }
    
    /// Get user choice from input
    fn get_user_choice(&self) -> Result<u32> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        input.trim().parse::<u32>()
            .map_err(|_| anyhow::anyhow!("Invalid input"))
    }
    
    /// Add a new test case interactively
    fn add_new_test(&self) -> Result<()> {
        println!("{}", "Adding New Test Case".yellow().bold());
        println!("{}", "====================".yellow());
        
        // Get markdown input
        println!("Enter markdown content (end with a line containing only '---'):");
        let markdown = self.get_multiline_input()?;
        
        // Process markdown through Marco
        let html_result = self.runner.process_markdown(&markdown)
            .map_err(|e| anyhow::anyhow!("Failed to process markdown: {}", e))?;
            
        // Show the result
        println!();
        println!("{}", "Generated HTML:".green().bold());
        println!("{}", html_result);
        println!();
        
        // Ask if user wants to save this as a test case
        if self.confirm("Save this as a new test case?")? {
            // Get test metadata
            println!("Enter section name:");
            let section = self.get_string_input()?;
            
            // Load or create spec file
            let spec_path = if self.confirm("Add to Marco-specific tests? (otherwise CommonMark)")? {
                PathBuf::from("tests/spec/marco.json")
            } else {
                PathBuf::from("tests/spec/commonmark.json")
            };
            
            let mut spec = if spec_path.exists() {
                TestSpec::load_from_file(&spec_path)
                    .with_context(|| format!("Failed to load spec from {:?}", spec_path))?
            } else {
                TestSpec {
                    tests: vec![],
                    source: spec_path.file_name().unwrap().to_string_lossy().to_string(),
                }
            };
            
            // Create new test case
            let test_case = TestCase {
                markdown,
                html: html_result,
                example: spec.next_example_number(),
                start_line: 0, // These would be filled in if parsing from a spec document
                end_line: 0,
                section,
            };
            
            spec.add_test(test_case.clone());
            
            // Save updated spec
            spec.save_to_file(&spec_path)
                .with_context(|| format!("Failed to save spec to {:?}", spec_path))?;
                
            println!("{} Added test case {} to {}!", "✓".green().bold(), test_case.example, spec.source);
        }
        
        Ok(())
    }
    
    /// Test a markdown string interactively
    fn test_markdown_string(&self) -> Result<()> {
        println!("{}", "Test Markdown String".yellow().bold());
        println!("{}", "===================".yellow());
        
        println!("Enter markdown content (end with a line containing only '---'):");
        let markdown = self.get_multiline_input()?;
        
        // Process and show result
        match self.runner.process_markdown(&markdown) {
            Ok(html) => {
                println!();
                println!("{}", "Generated HTML:".green().bold());
                println!("{}", html);
            }
            Err(e) => {
                println!("{}: {}", "Error".red().bold(), e);
            }
        }
        
        Ok(())
    }
    
    /// Review failing tests and optionally update them
    fn review_failing_tests(&self) -> Result<()> {
        println!("{}", "Reviewing Failing Tests".yellow().bold());
        println!("{}", "======================".yellow());
        
        let (results, summary) = self.runner.run_all_tests()?;
        
        if summary.failed == 0 {
            println!("{}", "No failing tests found!".green().bold());
            return Ok(());
        }
        
        println!("Found {} failing tests:", summary.failed);
        println!();
        
        for (test_case, result) in results {
            if let TestResult::Failed { expected: _, actual: _, diff } = result {
                println!("{} Example {} ({}):", "✗".red().bold(), test_case.example, test_case.section);
                println!("{}", diff);
                
                if self.confirm("Update this test's expected result?")? {
                    // This would update the test case in the appropriate spec file
                    println!("{}", "Baseline update functionality coming soon!".yellow().italic());
                }
                
                if !self.confirm("Continue to next failing test?")? {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Update test baselines
    fn update_baselines(&self) -> Result<()> {
        println!("{}", "Update Test Baselines".yellow().bold());
        println!("{}", "====================".yellow());
        println!("{}", "This feature will allow updating existing test baselines.".yellow().italic());
        println!("{}", "Coming soon!".green().italic());
        Ok(())
    }
    
    /// Browse test specifications
    fn browse_specifications(&self) -> Result<()> {
        println!("{}", "Browse Test Specifications".yellow().bold());
        println!("{}", "=========================".yellow());
        
        // Show available spec files
        let spec_files = [
            PathBuf::from("tests/spec/commonmark.json"),
            PathBuf::from("tests/spec/marco.json"),
        ];
        
        for (i, spec_file) in spec_files.iter().enumerate() {
            if spec_file.exists() {
                match TestSpec::load_from_file(spec_file) {
                    Ok(spec) => {
                        println!("{}. {} ({} tests)", i + 1, spec.source, spec.tests.len());
                    }
                    Err(_) => {
                        println!("{}. {} (failed to load)", i + 1, spec_file.file_name().unwrap().to_string_lossy());
                    }
                }
            } else {
                println!("{}. {} (not found)", i + 1, spec_file.file_name().unwrap().to_string_lossy());
            }
        }
        
        println!();
        println!("Enter number to browse a specification (or 0 to return):");
        
        match self.get_user_choice()? {
            0 => return Ok(()),
            n if n <= spec_files.len() as u32 => {
                let spec_file = &spec_files[(n - 1) as usize];
                if spec_file.exists() {
                    self.browse_spec_file(spec_file)?;
                } else {
                    println!("{}", "Specification file not found.".red());
                }
            }
            _ => {
                println!("{}", "Invalid selection.".red());
            }
        }
        
        Ok(())
    }
    
    /// Browse a specific specification file
    fn browse_spec_file(&self, spec_file: &PathBuf) -> Result<()> {
        let spec = TestSpec::load_from_file(spec_file)
            .with_context(|| format!("Failed to load spec from {:?}", spec_file))?;
            
        println!();
        println!("{}: {}", "Browsing".cyan().bold(), spec.source);
        println!("Total tests: {}", spec.tests.len());
        
        // Group by section
        let mut sections = std::collections::HashMap::new();
        for test in &spec.tests {
            sections.entry(&test.section).or_insert_with(Vec::new).push(test);
        }
        
        println!();
        println!("{}", "Sections:".cyan().bold());
        for (i, (section, tests)) in sections.iter().enumerate() {
            println!("{}. {} ({} tests)", i + 1, section, tests.len());
        }
        
        println!();
        if self.confirm("View details for a specific test case?")? {
            println!("Enter example number:");
            let example_num = self.get_user_choice()?;
            
            if let Some(test_case) = spec.find_by_example(example_num) {
                println!();
                println!("{} Example {}:", "Test Case".cyan().bold(), test_case.example);
                println!("{}: {}", "Section".blue(), test_case.section);
                println!();
                println!("{}", "Markdown:".blue().bold());
                println!("{}", test_case.markdown);
                println!();
                println!("{}", "Expected HTML:".blue().bold());
                println!("{}", test_case.html);
                
                if self.confirm("Test this example now?")? {
                    let result = self.runner.run_test_case(test_case);
                    match result {
                        TestResult::Passed => {
                            println!("{}", "✓ Test passed!".green().bold());
                        }
                        TestResult::Failed { diff, .. } => {
                            println!("{}", "✗ Test failed!".red().bold());
                            println!();
                            println!("{}", diff);
                        }
                        TestResult::Error { message } => {
                            println!("{}: {}", "Error".red().bold(), message);
                        }
                        TestResult::NoBaseline { .. } => {
                            // This shouldn't happen for loaded test cases
                            println!("{}", "No baseline (unexpected)".yellow());
                        }
                    }
                }
            } else {
                println!("{}", "Example not found.".red());
            }
        }
        
        Ok(())
    }
    
    /// Get a multi-line input from the user (until they enter "---")
    fn get_multiline_input(&self) -> Result<String> {
        let mut lines = Vec::new();
        
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            
            let trimmed = line.trim();
            if trimmed == "---" {
                break;
            }
            
            lines.push(line);
        }
        
        Ok(lines.join(""))
    }
    
    /// Get a single-line string input
    fn get_string_input(&self) -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
    
    /// Ask for user confirmation (y/n)
    fn confirm(&self, prompt: &str) -> Result<bool> {
        print!("{} (y/n): ", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::RunnerConfig;
    
    #[test]
    fn test_interactive_manager_creation() {
        let runner = TestRunner::new(RunnerConfig::default());
        let manager = InteractiveManager::new(runner.clone());
        
        // Verify the manager was created and contains the runner
        // We can't easily test the interactive parts without mocking stdin/stdout,
        // but we can test that the internal runner is accessible and functional
        let test_markdown = "# Test Header";
        let result = manager.runner.process_markdown(test_markdown);
        
        assert!(result.is_ok(), "Manager's runner should process markdown successfully");
        let html = result.unwrap();
        assert!(html.contains("Test Header"), "HTML should contain the test header text");
        
        // Test that the manager can be created without panicking
        // and verify basic internal state
        assert!(!html.is_empty(), "Generated HTML should not be empty");
    }
}