//! Main test configuration and entry point for cargo-optimize comprehensive test suite
//!
//! This module provides the main entry points for running the comprehensive test suite
//! and coordinates different test categories.

use std::env;
use std::process;
use std::time::Instant;

mod comprehensive {
    pub mod boundary_tests;
    pub mod ci_cd_tests;
    pub mod fuzz_tests;
    pub mod golden_master_tests;
    pub mod integration_tests_comprehensive;
    pub mod performance_tests;
    pub mod property_based_tests;
    pub mod regression_tests;
    pub mod stress_tests;
    pub mod test_runner_comprehensive;
    pub mod unit_tests_analyzer;
    pub mod unit_tests_config;
    pub mod unit_tests_detector;
    pub mod unit_tests_optimizer;
}

use comprehensive::test_runner_comprehensive::{
    generate_test_report, validate_test_environment, ReportFormat, TestCategory, TestRunner,
};

/// Main entry point for running all tests
fn main() {
    println!("🚀 Cargo-Optimize Comprehensive Test Suite");
    println!("{}", "=".repeat(60));

    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let mut categories = TestCategory::all();
    let mut parallel = true;
    let mut verbose = false;
    let mut fail_fast = false;
    let mut report_format = ReportFormat::Text;
    let mut validate_env = true;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--categories" => {
                if i + 1 < args.len() {
                    categories = parse_categories(&args[i + 1]);
                    i += 1;
                }
            }
            "--no-parallel" => {
                parallel = false;
            }
            "--verbose" | "-v" => {
                verbose = true;
            }
            "--fail-fast" => {
                fail_fast = true;
            }
            "--report" => {
                if i + 1 < args.len() {
                    report_format = parse_report_format(&args[i + 1]);
                    i += 1;
                }
            }
            "--no-validate" => {
                validate_env = false;
            }
            "--list-categories" => {
                list_categories();
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                process::exit(1);
            }
        }
        i += 1;
    }

    // Validate test environment
    if validate_env {
        println!("🔍 Validating test environment...");
        if let Err(errors) = validate_test_environment() {
            eprintln!("❌ Test environment validation failed:");
            for error in errors {
                eprintln!("  • {}", error);
            }
            eprintln!("\nUse --no-validate to skip validation");
            process::exit(1);
        }
        println!("✅ Test environment validation passed");
    }

    // Run tests
    println!("\n🧪 Running tests...");
    let start_time = Instant::now();

    let runner = TestRunner::new()
        .categories(categories)
        .parallel(parallel)
        .verbose(verbose)
        .fail_fast(fail_fast);

    match runner.run() {
        Ok(suite) => {
            let duration = start_time.elapsed();
            println!("\n✅ Test suite completed in {:.2?}", duration);

            // Generate and display report
            suite.print_detailed_report();

            // Generate additional report formats if requested
            match generate_test_report(&suite, report_format.clone()) {
                Ok(report) => {
                    match report_format {
                        ReportFormat::Text => {
                            // Already printed above
                        }
                        ReportFormat::Json => {
                            println!("\n📄 JSON Report:");
                            println!("{}", report);
                        }
                        ReportFormat::Junit => {
                            println!("\n📄 JUnit Report:");
                            println!("{}", report);
                        }
                        ReportFormat::Html => {
                            println!("\n📄 HTML Report generated");
                            // In a real implementation, you'd write this to a file
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to generate report: {}", e);
                }
            }

            // Exit with appropriate code
            let summary = suite.summary();
            if summary.total_failed > 0 {
                eprintln!("\n❌ {} tests failed", summary.total_failed);
                process::exit(1);
            } else {
                println!("\n🎉 All {} tests passed!", summary.total_passed);
                process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("❌ Test suite failed: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("cargo-optimize comprehensive test suite");
    println!();
    println!("USAGE:");
    println!("    cargo test --bin test_main [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --categories <LIST>     Comma-separated list of test categories to run");
    println!("    --no-parallel          Run tests sequentially instead of in parallel");
    println!("    --verbose, -v           Enable verbose output");
    println!("    --fail-fast             Stop on first test failure");
    println!(
        "    --report <FORMAT>       Generate report in specified format (text|json|junit|html)"
    );
    println!("    --no-validate           Skip test environment validation");
    println!("    --list-categories       List all available test categories");
    println!("    --help, -h              Show this help message");
    println!();
    println!("CATEGORIES:");
    for category in TestCategory::all() {
        println!(
            "    {:<15} {}",
            format!("{:?}", category),
            category.description()
        );
    }
    println!();
    println!("EXAMPLES:");
    println!("    cargo test --bin test_main                           # Run all tests");
    println!(
        "    cargo test --bin test_main --categories unit,integration  # Run specific categories"
    );
    println!("    cargo test --bin test_main --verbose --fail-fast    # Run with verbose output, stop on failure");
    println!("    cargo test --bin test_main --report json            # Generate JSON report");
}

fn list_categories() {
    println!("Available test categories:");
    for category in TestCategory::all() {
        println!("  {:?}:", category);
        println!("    Description: {}", category.description());
        println!("    Pattern: {}", category.test_pattern());
        println!();
    }
}

fn parse_categories(input: &str) -> Vec<TestCategory> {
    let mut categories = Vec::new();

    for category_str in input.split(',') {
        let category_str = category_str.trim().to_lowercase();
        match category_str.as_str() {
            "unit" => categories.push(TestCategory::Unit),
            "integration" => categories.push(TestCategory::Integration),
            "property" | "property-based" => categories.push(TestCategory::PropertyBased),
            "fuzz" => categories.push(TestCategory::Fuzz),
            "performance" | "perf" | "benchmark" => categories.push(TestCategory::Performance),
            "golden" | "golden-master" => categories.push(TestCategory::GoldenMaster),
            "regression" => categories.push(TestCategory::Regression),
            "stress" => categories.push(TestCategory::Stress),
            "boundary" => categories.push(TestCategory::BoundaryValue),
            _ => {
                eprintln!("Warning: Unknown category '{}'", category_str);
            }
        }
    }

    if categories.is_empty() {
        eprintln!("No valid categories specified, using all categories");
        TestCategory::all()
    } else {
        categories
    }
}

fn parse_report_format(input: &str) -> ReportFormat {
    match input.to_lowercase().as_str() {
        "text" | "txt" => ReportFormat::Text,
        "json" => ReportFormat::Json,
        "junit" | "xml" => ReportFormat::Junit,
        "html" | "htm" => ReportFormat::Html,
        _ => {
            eprintln!("Warning: Unknown report format '{}', using text", input);
            ReportFormat::Text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_parsing() {
        let categories = parse_categories("unit,integration,fuzz");
        assert_eq!(categories.len(), 3);
        assert!(categories.contains(&TestCategory::Unit));
        assert!(categories.contains(&TestCategory::Integration));
        assert!(categories.contains(&TestCategory::Fuzz));
    }

    #[test]
    fn test_report_format_parsing() {
        assert!(matches!(parse_report_format("json"), ReportFormat::Json));
        assert!(matches!(parse_report_format("junit"), ReportFormat::Junit));
        assert!(matches!(parse_report_format("html"), ReportFormat::Html));
        assert!(matches!(parse_report_format("invalid"), ReportFormat::Text));
    }
}

/// Configuration for the comprehensive test suite

impl Default for TestSuiteConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            verbose: false,
            fail_fast: false,
            timeout_seconds: 300, // 5 minutes
            validate_environment: true,
            categories: TestCategory::all(),
            report_format: ReportFormat::Text,
        }
    }
}

impl TestSuiteConfig {
    /// Create a configuration for CI environments
    pub fn for_ci() -> Self {
        Self {
            parallel: true,
            verbose: true,
            fail_fast: true,
            timeout_seconds: 600,        // 10 minutes for CI
            validate_environment: false, // Skip validation in CI
            categories: TestCategory::all(),
            report_format: ReportFormat::Junit, // JUnit for CI integration
        }
    }

    /// Create a configuration for development
    pub fn for_dev() -> Self {
        Self {
            parallel: true,
            verbose: false,
            fail_fast: false,
            timeout_seconds: 180, // 3 minutes for dev
            validate_environment: true,
            categories: vec![
                TestCategory::Unit,
                TestCategory::Integration,
                TestCategory::BoundaryValue,
            ],
            report_format: ReportFormat::Text,
        }
    }

    /// Create a configuration for quick testing
    pub fn quick() -> Self {
        Self {
            parallel: true,
            verbose: false,
            fail_fast: true,
            timeout_seconds: 60, // 1 minute
            validate_environment: false,
            categories: vec![TestCategory::Unit],
            report_format: ReportFormat::Text,
        }
    }

    /// Create a configuration for comprehensive testing
    pub fn comprehensive() -> Self {
        Self {
            parallel: false, // Sequential for thorough testing
            verbose: true,
            fail_fast: false,
            timeout_seconds: 1800, // 30 minutes
            validate_environment: true,
            categories: TestCategory::all(),
            report_format: ReportFormat::Html,
        }
    }
}

/// Run tests with a specific configuration
pub fn run_tests_with_config(config: TestSuiteConfig) -> Result<(), Box<dyn std::error::Error>> {
    if config.validate_environment {
        validate_test_environment()
            .map_err(|errors| format!("Environment validation failed: {}", errors.join(", ")))?;
    }

    let runner = TestRunner::new()
        .categories(config.categories)
        .parallel(config.parallel)
        .verbose(config.verbose)
        .fail_fast(config.fail_fast)
        .timeout(std::time::Duration::from_secs(config.timeout_seconds));

    let suite = runner.run()?;

    // Generate report
    let _report = generate_test_report(&suite, config.report_format)?;

    let summary = suite.summary();
    if summary.total_failed > 0 {
        return Err(format!("{} tests failed", summary.total_failed).into());
    }

    Ok(())
}

/// Macro for easily running specific test categories
#[macro_export]
macro_rules! run_category {
    ($category:expr) => {{
        use $crate::TestSuiteConfig;
        let mut config = TestSuiteConfig::default();
        config.categories = vec![$category];
        $crate::run_tests_with_config(config)
    }};
}

/// Macro for running tests in CI mode
#[macro_export]
macro_rules! run_ci_tests {
    () => {{
        use $crate::TestSuiteConfig;
        $crate::run_tests_with_config(TestSuiteConfig::for_ci())
    }};
}

/// Macro for running quick tests
#[macro_export]
macro_rules! run_quick_tests {
    () => {{
        use $crate::TestSuiteConfig;
        $crate::run_tests_with_config(TestSuiteConfig::quick())
    }};
}

// Export the main components for external use
// TestSuiteConfig is already defined in this module above
