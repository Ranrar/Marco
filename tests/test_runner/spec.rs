//! JSON test specification structures
//! 
//! Defines the structures for parsing and managing test cases from JSON spec files
//! like commonmark.json and marco.json.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A single test case from the JSON specification
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TestCase {
    /// The markdown input to test
    pub markdown: String,
    
    /// The expected HTML output
    pub html: String,
    
    /// Example number from the specification
    pub example: u32,
    
    /// Starting line number in the specification document
    pub start_line: u32,
    
    /// Ending line number in the specification document
    pub end_line: u32,
    
    /// Section name this test belongs to
    pub section: String,
}

/// Collection of test cases loaded from a JSON specification file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestSpec {
    /// All test cases in this specification
    pub tests: Vec<TestCase>,
    
    /// Source file name (for tracking purposes)
    pub source: String,
}

impl TestSpec {
    /// Load a test specification from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        
        // Handle empty files gracefully
        if content.trim().is_empty() {
            return Ok(TestSpec {
                tests: vec![],
                source: path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            });
        }
        
        let tests: Vec<TestCase> = serde_json::from_str(&content)?;
        
        Ok(TestSpec {
            tests,
            source: path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        })
    }
    
    /// Save the test specification back to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.tests)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Find a test case by example number
    pub fn find_by_example(&self, example: u32) -> Option<&TestCase> {
        self.tests.iter().find(|test| test.example == example)
    }
    
    /// Add a new test case to the specification
    pub fn add_test(&mut self, test_case: TestCase) {
        self.tests.push(test_case);
    }
    
    /// Get the next available example number
    pub fn next_example_number(&self) -> u32 {
        self.tests.iter()
            .map(|test| test.example)
            .max()
            .unwrap_or(0) + 1
    }
}

/// Test result for a single test case
#[derive(Debug, Clone)]
pub enum TestResult {
    /// Test passed - actual output matches expected
    Passed,
    
    /// Test failed - actual output doesn't match expected
    Failed {
        expected: String,
        actual: String,
        diff: String,
    },
    
    /// Test case has no expected result - needs baseline
    NoBaseline {
        actual: String,
    },
    
    /// Test case couldn't be executed due to error
    Error {
        message: String,
    },
}

/// Summary of test run results
#[derive(Debug, Default)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub needs_baseline: usize,
    pub errors: usize,
}

impl TestSummary {
    /// Create a new empty test summary
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a test result in the summary
    pub fn record(&mut self, result: &TestResult) {
        self.total += 1;
        match result {
            TestResult::Passed => self.passed += 1,
            TestResult::Failed { .. } => self.failed += 1,
            TestResult::NoBaseline { .. } => self.needs_baseline += 1,
            TestResult::Error { .. } => self.errors += 1,
        }
    }
    
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_load_empty_spec() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file).unwrap();
        
        let spec = TestSpec::load_from_file(temp_file.path()).unwrap();
        assert_eq!(spec.tests.len(), 0);
        assert!(!spec.source.is_empty());
    }
    
    #[test]
    fn test_load_valid_spec() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[
            {{
                \"markdown\": \"# Testing\",
                \"html\": \"<h1>Testing</h1>\",
                \"example\": 1,
                \"start_line\": 10,
                \"end_line\": 15,
                \"section\": \"Headers\"
            }}
        ]").unwrap();
        
        let spec = TestSpec::load_from_file(temp_file.path()).unwrap();
        assert_eq!(spec.tests.len(), 1);
        assert_eq!(spec.tests[0].markdown, "# Testing");
        assert_eq!(spec.tests[0].html, "<h1>Testing</h1>");
        assert_eq!(spec.tests[0].example, 1);
        assert_eq!(spec.tests[0].section, "Headers");
    }
    
    #[test]
    fn test_find_by_example() {
        let spec = TestSpec {
            tests: vec![
                TestCase {
                    markdown: "# Test 1".to_string(),
                    html: "<h1>Test 1</h1>".to_string(),
                    example: 1,
                    start_line: 10,
                    end_line: 15,
                    section: "Headers".to_string(),
                },
                TestCase {
                    markdown: "# Test 2".to_string(),
                    html: "<h1>Test 2</h1>".to_string(),
                    example: 2,
                    start_line: 20,
                    end_line: 25,
                    section: "Headers".to_string(),
                },
            ],
            source: "test.json".to_string(),
        };
        
        let test = spec.find_by_example(1).unwrap();
        assert_eq!(test.markdown, "# Test 1");
        
        let test = spec.find_by_example(999);
        assert!(test.is_none());
    }
    
    #[test]
    fn test_next_example_number() {
        let mut spec = TestSpec {
            tests: vec![],
            source: "test.json".to_string(),
        };
        
        assert_eq!(spec.next_example_number(), 1);
        
        spec.add_test(TestCase {
            markdown: "# Test".to_string(),
            html: "<h1>Test</h1>".to_string(),
            example: 5,
            start_line: 10,
            end_line: 15,
            section: "Headers".to_string(),
        });
        
        assert_eq!(spec.next_example_number(), 6);
    }
    
    #[test]
    fn test_summary() {
        let mut summary = TestSummary::new();
        
        summary.record(&TestResult::Passed);
        summary.record(&TestResult::Failed {
            expected: "foo".to_string(),
            actual: "bar".to_string(),
            diff: "diff".to_string(),
        });
        summary.record(&TestResult::NoBaseline {
            actual: "baz".to_string(),
        });
        
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.needs_baseline, 1);
        // Check success rate with reasonable tolerance for floating point precision
        assert!((summary.success_rate() - 33.333333333333336).abs() < 0.000001);
    }
}