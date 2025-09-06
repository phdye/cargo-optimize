//! Comprehensive Test Runner for cargo-optimize
//!
//! This module provides the core test execution framework that orchestrates
//! all test categories and provides unified reporting.

use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};


/// Available test categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// Unit tests for individual components
    Unit,
    /// Integration tests for complete workflows
    Integration,
    /// Property-based tests for invariant verification
    PropertyBased,
    /// Fuzz tests for robustness
    Fuzz,
    /// Performance benchmarks
    Performance,
    /// Golden master tests for regression prevention
    GoldenMaster,
    /// Regression tests
    Regression,
    /// Stress tests for extreme conditions
    Stress,
    /// Boundary value tests
    BoundaryValue,
    /// CI/CD specific tests
    CiCd,
}

impl TestCategory {
    /// Get all available test categories
    pub fn all() -> Vec<TestCategory> {
        vec![
            TestCategory::Unit,
            TestCategory::Integration,
            TestCategory::PropertyBased,
            TestCategory::Fuzz,
            TestCategory::Performance,
            TestCategory::GoldenMaster,
            TestCategory::Regression,
            TestCategory::Stress,
            TestCategory::BoundaryValue,
            TestCategory::CiCd,
        ]
    }

    /// Get description of this test category
    pub fn description(&self) -> &'static str {
        match self {
            TestCategory::Unit => "Unit tests for individual components",
            TestCategory::Integration => "Integration tests for complete workflows",
            TestCategory::PropertyBased => "Property-based tests for invariant verification",
            TestCategory::Fuzz => "Fuzz tests for robustness with random inputs",
            TestCategory::Performance => "Performance benchmarks and timing tests",
            TestCategory::GoldenMaster => "Golden master tests for regression prevention",
            TestCategory::Regression => "Regression tests for known issues",
            TestCategory::Stress => "Stress tests for extreme conditions",
            TestCategory::BoundaryValue => "Boundary value tests for edge cases",
            TestCategory::CiCd => "CI/CD integration and compatibility tests",
        }
    }

    /// Get test pattern for this category
    pub fn test_pattern(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit_test*",
            TestCategory::Integration => "integration_test*",
            TestCategory::PropertyBased => "property_*",
            TestCategory::Fuzz => "fuzz_*",
            TestCategory::Performance => "benchmark_*",
            TestCategory::GoldenMaster => "golden_master_*",
            TestCategory::Regression => "regression_*",
            TestCategory::Stress => "stress_*",
            TestCategory::BoundaryValue => "boundary_*",
            TestCategory::CiCd => "ci_cd_*",
        }
    }
}

/// Result of a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub category: TestCategory,
    pub passed: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub output: String,
}

impl TestResult {
    pub fn new(name: String, category: TestCategory) -> Self {
        Self {
            name,
            category,
            passed: false,
            duration: Duration::default(),
            error_message: None,
            output: String::new(),
        }
    }

    pub fn passed(mut self, duration: Duration) -> Self {
        self.passed = true;
        self.duration = duration;
        self
    }

    pub fn failed(mut self, duration: Duration, error: String) -> Self {
        self.passed = false;
        self.duration = duration;
        self.error_message = Some(error);
        self
    }


}

/// Complete test suite results
#[derive(Debug)]
pub struct TestSuite {
    pub results: Vec<TestResult>,
    pub total_duration: Duration,
    pub categories_run: Vec<TestCategory>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            total_duration: Duration::default(),
            categories_run: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: TestResult) {
        if !self.categories_run.contains(&result.category) {
            self.categories_run.push(result.category);
        }
        self.results.push(result);
    }

    pub fn summary(&self) -> TestSummary {
        let total_tests = self.results.len();
        let total_passed = self.results.iter().filter(|r| r.passed).count();
        let total_failed = total_tests - total_passed;

        let mut by_category = HashMap::new();
        for result in &self.results {
            let entry = by_category.entry(result.category).or_insert((0, 0));
            if result.passed {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }

        TestSummary {
            total_tests,
            total_passed,
            total_failed,
            total_duration: self.total_duration,
            by_category,
        }
    }

    pub fn print_detailed_report(&self) {
        let summary = self.summary();

        println!("\n🧪 Test Summary");
        println!("{}", "=".repeat(50));
        println!("Total Tests: {}", summary.total_tests);
        println!("Passed: {}", summary.total_passed);
        println!("Failed: {}", summary.total_failed);
        println!("Duration: {:.2?}", summary.total_duration);
        println!();

        // By category
        println!("📊 Results by Category");
        println!("{}", "-".repeat(50));
        for (category, (passed, failed)) in &summary.by_category {
            let total = passed + failed;
            let pass_rate = if total > 0 {
                (*passed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            println!(
                "{:?}: {} passed, {} failed ({:.1}%)",
                category, passed, failed, pass_rate
            );
        }
        println!();

        // Failed tests details
        let failed_tests: Vec<_> = self.results.iter().filter(|r| !r.passed).collect();

        if !failed_tests.is_empty() {
            println!("❌ Failed Tests");
            println!("{}", "-".repeat(50));
            for test in failed_tests {
                println!("• {}", test.name);
                if let Some(error) = &test.error_message {
                    println!("  Error: {}", error);
                }
                println!("  Duration: {:.2?}", test.duration);
                if !test.output.is_empty() {
                    println!("  Output: {}", test.output);
                }
                println!();
            }
        }

        // Performance summary
        let avg_duration = if summary.total_tests > 0 {
            summary.total_duration / summary.total_tests as u32
        } else {
            Duration::default()
        };

        println!("⚡ Performance");
        println!("{}", "-".repeat(50));
        println!("Average test time: {:.2?}", avg_duration);

        let slowest = self.results.iter().max_by_key(|r| r.duration);
        if let Some(slow) = slowest {
            println!("Slowest test: {} ({:.2?})", slow.name, slow.duration);
        }

        let fastest = self.results.iter().min_by_key(|r| r.duration);
        if let Some(fast) = fastest {
            println!("Fastest test: {} ({:.2?})", fast.name, fast.duration);
        }
    }
}

/// Summary of test results
#[derive(Debug)]
pub struct TestSummary {
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_duration: Duration,
    pub by_category: HashMap<TestCategory, (usize, usize)>, // (passed, failed)
}

/// Main test runner
#[derive(Debug)]
pub struct TestRunner {
    categories: Vec<TestCategory>,
    parallel: bool,
    verbose: bool,
    fail_fast: bool,
    timeout: Duration,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            categories: TestCategory::all(),
            parallel: true,
            verbose: false,
            fail_fast: false,
            timeout: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn categories(mut self, categories: Vec<TestCategory>) -> Self {
        self.categories = categories;
        self
    }

    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run all configured tests
    pub fn run(self) -> Result<TestSuite, String> {
        let start_time = Instant::now();
        let mut suite = TestSuite::new();

        if self.verbose {
            println!("🚀 Running test categories: {:?}", self.categories);
            println!("Parallel: {}, Fail fast: {}", self.parallel, self.fail_fast);
            println!();
        }

        // Collect all tests to run
        let mut all_tests = Vec::new();
        for category in &self.categories {
            let tests = self.discover_tests(*category)?;
            all_tests.extend(tests);
        }

        if self.verbose {
            println!("Discovered {} tests", all_tests.len());
        }

        // Run tests
        if self.parallel && all_tests.len() > 1 {
            self.run_parallel(all_tests, &mut suite)?;
        } else {
            self.run_sequential(all_tests, &mut suite)?;
        }

        suite.total_duration = start_time.elapsed();
        Ok(suite)
    }

    fn discover_tests(
        &self,
        category: TestCategory,
    ) -> Result<Vec<(String, TestCategory)>, String> {
        // In a real implementation, this would use reflection or test discovery
        // For now, we'll return mock tests based on category
        let mut tests = Vec::new();

        match category {
            TestCategory::Unit => {
                tests.extend(vec![
                    ("test_config_default_values".to_string(), category),
                    ("test_config_builder_pattern".to_string(), category),
                    ("test_detector_cpu_cores".to_string(), category),
                    ("test_detector_memory_info".to_string(), category),
                    ("test_analyzer_project_stats".to_string(), category),
                    ("test_optimizer_feature_selection".to_string(), category),
                ]);
            }
            TestCategory::Integration => {
                tests.extend(vec![
                    ("test_end_to_end_optimization".to_string(), category),
                    ("test_real_project_analysis".to_string(), category),
                    ("test_linker_configuration".to_string(), category),
                ]);
            }
            TestCategory::PropertyBased => {
                tests.extend(vec![
                    (
                        "property_config_serialization_roundtrip".to_string(),
                        category,
                    ),
                    (
                        "property_hardware_detection_consistency".to_string(),
                        category,
                    ),
                ]);
            }
            TestCategory::Fuzz => {
                tests.extend(vec![
                    ("fuzz_config_parsing".to_string(), category),
                    ("fuzz_file_operations".to_string(), category),
                ]);
            }
            TestCategory::Performance => {
                tests.extend(vec![
                    ("benchmark_config_creation".to_string(), category),
                    ("benchmark_hardware_detection".to_string(), category),
                    ("benchmark_project_analysis".to_string(), category),
                ]);
            }
            TestCategory::GoldenMaster => {
                tests.extend(vec![
                    ("golden_master_default_config".to_string(), category),
                    ("golden_master_optimization_features".to_string(), category),
                ]);
            }
            TestCategory::Regression => {
                tests.extend(vec![("regression_issue_001".to_string(), category)]);
            }
            TestCategory::Stress => {
                tests.extend(vec![
                    ("stress_large_project".to_string(), category),
                    ("stress_concurrent_access".to_string(), category),
                ]);
            }
            TestCategory::BoundaryValue => {
                tests.extend(vec![
                    ("boundary_empty_config".to_string(), category),
                    ("boundary_max_parallel_jobs".to_string(), category),
                ]);
            }
            TestCategory::CiCd => {
                tests.extend(vec![
                    ("ci_cd_github_actions".to_string(), category),
                    ("ci_cd_gitlab_ci".to_string(), category),
                    ("ci_cd_jenkins".to_string(), category),
                ]);
            }
        }

        Ok(tests)
    }

    fn run_sequential(
        &self,
        tests: Vec<(String, TestCategory)>,
        suite: &mut TestSuite,
    ) -> Result<(), String> {
        for (test_name, category) in tests {
            if self.verbose {
                println!("Running {}...", test_name);
            }

            let result = self.run_single_test(test_name.clone(), category)?;

            if self.verbose {
                if result.passed {
                    println!("  ✅ {} ({:.2?})", test_name, result.duration);
                } else {
                    println!("  ❌ {} ({:.2?})", test_name, result.duration);
                    if let Some(error) = &result.error_message {
                        println!("     Error: {}", error);
                    }
                }
            }

            let failed = !result.passed;
            suite.add_result(result);

            if failed && self.fail_fast {
                return Err(format!(
                    "Test {} failed, stopping due to fail-fast",
                    test_name
                ));
            }
        }

        Ok(())
    }

    fn run_parallel(
        &self,
        tests: Vec<(String, TestCategory)>,
        suite: &mut TestSuite,
    ) -> Result<(), String> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();

        for (test_name, category) in tests {
            let results_clone = Arc::clone(&results);
            let verbose = self.verbose;
            let _timeout = self.timeout;

            let handle = thread::spawn(move || {
                let start_time = Instant::now();

                // Simulate test execution
                let result = if test_name.contains("fail") {
                    TestResult::new(test_name.clone(), category)
                        .failed(start_time.elapsed(), "Simulated test failure".to_string())
                } else {
                    thread::sleep(Duration::from_millis(50 + (test_name.len() % 100) as u64));
                    TestResult::new(test_name.clone(), category).passed(start_time.elapsed())
                };

                if verbose {
                    if result.passed {
                        println!("  ✅ {} ({:.2?})", test_name, result.duration);
                    } else {
                        println!("  ❌ {} ({:.2?})", test_name, result.duration);
                    }
                }

                results_clone.lock().unwrap().push(result);
            });

            handles.push(handle);
        }

        // Wait for all tests to complete
        for handle in handles {
            handle.join().map_err(|_| "Thread panicked")?;
        }

        // Add all results to suite
        let results = results.lock().unwrap();
        for result in results.iter() {
            suite.add_result(result.clone());
        }

        // Check for failures if fail_fast is enabled
        if self.fail_fast {
            for result in results.iter() {
                if !result.passed {
                    return Err(format!("Test {} failed", result.name));
                }
            }
        }

        Ok(())
    }

    fn run_single_test(
        &self,
        test_name: String,
        category: TestCategory,
    ) -> Result<TestResult, String> {
        let start_time = Instant::now();

        // For demonstration, we'll simulate test execution
        // In a real implementation, this would actually run the test function
        let result = match test_name.as_str() {
            name if name.contains("fail") => TestResult::new(test_name, category)
                .failed(start_time.elapsed(), "Simulated test failure".to_string()),
            name if name.contains("slow") => {
                thread::sleep(Duration::from_millis(200));
                TestResult::new(test_name, category).passed(start_time.elapsed())
            }
            _ => {
                // Simulate some work
                thread::sleep(Duration::from_millis(10 + (test_name.len() % 50) as u64));
                TestResult::new(test_name, category).passed(start_time.elapsed())
            }
        };

        Ok(result)
    }
}

/// Report format options
#[derive(Debug, Clone)]
pub enum ReportFormat {
    /// Human-readable text format
    Text,
    /// JSON format for machine processing
    Json,
    /// JUnit XML format for CI systems
    Junit,
    /// HTML format for rich reporting
    Html,
}

/// Validate test environment before running tests
pub fn validate_test_environment() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Check Rust toolchain
    if std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_err()
    {
        errors.push("Rust compiler not found".to_string());
    }

    // Check Cargo
    if std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .is_err()
    {
        errors.push("Cargo not found".to_string());
    }

    // Check for required dependencies
    let cargo_toml_path = std::path::Path::new("Cargo.toml");
    if !cargo_toml_path.exists() {
        errors.push("Cargo.toml not found in current directory".to_string());
    }

    // Check write permissions for test artifacts
    let temp_dir = std::env::temp_dir();
    if std::fs::File::create(temp_dir.join("cargo_optimize_test_write")).is_err() {
        errors.push("No write permission in temporary directory".to_string());
    } else {
        let _ = std::fs::remove_file(temp_dir.join("cargo_optimize_test_write"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Generate test report in specified format
pub fn generate_test_report(suite: &TestSuite, format: ReportFormat) -> Result<String, String> {
    match format {
        ReportFormat::Text => generate_text_report(suite),
        ReportFormat::Json => generate_json_report(suite),
        ReportFormat::Junit => generate_junit_report(suite),
        ReportFormat::Html => generate_html_report(suite),
    }
}

fn generate_text_report(suite: &TestSuite) -> Result<String, String> {
    let summary = suite.summary();

    let mut report = String::new();
    report.push_str(&format!("Test Results Summary\n"));
    report.push_str(&format!("====================\n"));
    report.push_str(&format!("Total Tests: {}\n", summary.total_tests));
    report.push_str(&format!("Passed: {}\n", summary.total_passed));
    report.push_str(&format!("Failed: {}\n", summary.total_failed));
    report.push_str(&format!("Duration: {:.2?}\n\n", summary.total_duration));

    report.push_str("Results by Category:\n");
    for (category, (passed, failed)) in &summary.by_category {
        report.push_str(&format!(
            "{:?}: {} passed, {} failed\n",
            category, passed, failed
        ));
    }

    Ok(report)
}

fn generate_json_report(suite: &TestSuite) -> Result<String, String> {
    let summary = suite.summary();

    let mut json = format!(
        r#"{{
  "summary": {{
    "total_tests": {},
    "total_passed": {},
    "total_failed": {},
    "duration_ms": {}
  }},
  "results": ["#,
        summary.total_tests,
        summary.total_passed,
        summary.total_failed,
        summary.total_duration.as_millis()
    );

    for (i, result) in suite.results.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"
    {{
      "name": "{}",
      "category": "{:?}",
      "passed": {},
      "duration_ms": {}
    }}"#,
            result.name,
            result.category,
            result.passed,
            result.duration.as_millis()
        ));
    }

    json.push_str("\n  ]\n}");
    Ok(json)
}

fn generate_junit_report(suite: &TestSuite) -> Result<String, String> {
    let summary = suite.summary();

    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="cargo-optimize" tests="{}" failures="{}" time="{:.3}">
"#,
        summary.total_tests,
        summary.total_failed,
        summary.total_duration.as_secs_f64()
    );

    for result in &suite.results {
        xml.push_str(&format!(
            r#"  <testcase name="{}" classname="{:?}" time="{:.3}""#,
            result.name,
            result.category,
            result.duration.as_secs_f64()
        ));

        if result.passed {
            xml.push_str(" />\n");
        } else {
            xml.push_str(">\n");
            if let Some(error) = &result.error_message {
                xml.push_str(&format!(
                    r#"    <failure message="{}" />"#,
                    html_escape(error)
                ));
            }
            xml.push_str("\n  </testcase>\n");
        }
    }

    xml.push_str("</testsuite>\n");
    Ok(xml)
}

fn generate_html_report(suite: &TestSuite) -> Result<String, String> {
    let summary = suite.summary();

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>cargo-optimize Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>cargo-optimize Test Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Tests: {}</p>
        <p class="passed">Passed: {}</p>
        <p class="failed">Failed: {}</p>
        <p>Duration: {:.2?}</p>
    </div>
    
    <h2>Test Results</h2>
    <table>
        <tr>
            <th>Test Name</th>
            <th>Category</th>
            <th>Status</th>
            <th>Duration</th>
        </tr>"#,
        summary.total_tests, summary.total_passed, summary.total_failed, summary.total_duration
    );

    let mut table_rows = String::new();
    for result in &suite.results {
        let status_class = if result.passed { "passed" } else { "failed" };
        let status_text = if result.passed { "PASS" } else { "FAIL" };

        table_rows.push_str(&format!(
            r#"
        <tr>
            <td>{}</td>
            <td>{:?}</td>
            <td class="{}">{}</td>
            <td>{:.2?}</td>
        </tr>"#,
            html_escape(&result.name),
            result.category,
            status_class,
            status_text,
            result.duration
        ));
    }

    Ok(format!(
        "{}{}\n    </table>\n</body>\n</html>",
        html, table_rows
    ))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_all() {
        let categories = TestCategory::all();
        assert!(!categories.is_empty());
        assert!(categories.contains(&TestCategory::Unit));
        assert!(categories.contains(&TestCategory::Integration));
    }

    #[test]
    fn test_category_descriptions() {
        for category in TestCategory::all() {
            assert!(!category.description().is_empty());
            assert!(!category.test_pattern().is_empty());
        }
    }

    #[test]
    fn test_test_result_creation() {
        let result = TestResult::new("test_example".to_string(), TestCategory::Unit)
            .passed(Duration::from_millis(100));

        assert_eq!(result.name, "test_example");
        assert_eq!(result.category, TestCategory::Unit);
        assert!(result.passed);
        assert_eq!(result.duration, Duration::from_millis(100));
    }

    #[test]
    fn test_test_suite_summary() {
        let mut suite = TestSuite::new();

        suite.add_result(
            TestResult::new("test1".to_string(), TestCategory::Unit)
                .passed(Duration::from_millis(50)),
        );

        suite.add_result(
            TestResult::new("test2".to_string(), TestCategory::Unit)
                .failed(Duration::from_millis(100), "Error".to_string()),
        );

        let summary = suite.summary();
        assert_eq!(summary.total_tests, 2);
        assert_eq!(summary.total_passed, 1);
        assert_eq!(summary.total_failed, 1);
    }

    #[test]
    fn test_runner_configuration() {
        let runner = TestRunner::new()
            .categories(vec![TestCategory::Unit])
            .parallel(false)
            .verbose(true)
            .fail_fast(true);

        assert_eq!(runner.categories, vec![TestCategory::Unit]);
        assert!(!runner.parallel);
        assert!(runner.verbose);
        assert!(runner.fail_fast);
    }

    #[test]
    fn test_report_generation() {
        let mut suite = TestSuite::new();
        suite.add_result(
            TestResult::new("test1".to_string(), TestCategory::Unit)
                .passed(Duration::from_millis(50)),
        );

        let text_report = generate_text_report(&suite).unwrap();
        assert!(text_report.contains("Total Tests: 1"));
        assert!(text_report.contains("Passed: 1"));

        let json_report = generate_json_report(&suite).unwrap();
        assert!(json_report.contains("\"total_tests\": 1"));

        let junit_report = generate_junit_report(&suite).unwrap();
        assert!(junit_report.contains("tests=\"1\""));

        let html_report = generate_html_report(&suite).unwrap();
        assert!(html_report.contains("<html>"));
        assert!(html_report.contains("Total Tests: 1"));
    }
}
