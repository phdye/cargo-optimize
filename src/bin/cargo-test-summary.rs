#!/usr/bin/env cargo

//! cargo-test-summary - A formatted test result summary for cargo nextest
//! 
//! This cargo subcommand runs nextest and formats the output
//! in a clean, organized summary format with accurate test counts.
//! 
//! Usage: cargo test-summary [nextest arguments]

use std::process::Command;
use std::collections::HashSet;
use regex::Regex;

#[derive(Debug, Default)]
struct TestResults {
    passed: Vec<TestResult>,
    failed: Vec<TestResult>,
    skipped: Vec<TestResult>,
    total_time: String,
}

#[derive(Debug, Clone)]
struct TestResult {
    status: String,
    time: String,
    binary: String,
    test_name: String,
}

impl TestResult {
    fn unique_id(&self) -> String {
        format!("{}::{}", self.binary, self.test_name)
    }
}

fn main() {
    // Skip the first arg if it's "test-summary" (cargo passes subcommand name)
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "test-summary")
        .collect();
    
    // Run nextest and capture both stdout and stderr
    let output = Command::new("cargo")
        .arg("nextest")
        .arg("run")
        .args(&args)
        .output()
        .expect("Failed to run cargo nextest");
    
    // Convert output to string - nextest outputs to stderr
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    // Process stderr first (where nextest output goes), then stdout
    let mut results = parse_nextest_output(&stderr_str);
    if results.passed.is_empty() && results.failed.is_empty() {
        // Try stdout if stderr had no results
        results = parse_nextest_output(&stdout_str);
    }
    
    // Format and print test summary output
    print_test_summary(&results);
    
    // Exit with the same code as nextest
    std::process::exit(output.status.code().unwrap_or(1));
}

fn parse_nextest_output(output: &str) -> TestResults {
    let mut results = TestResults::default();
    let mut seen_tests = HashSet::new();
    
    // Regex patterns for parsing nextest output
    let test_line_regex = Regex::new(
        r"^\s*(PASS|FAIL|SKIP)\s+\[\s*([0-9.]+)s\]\s+(\S+)(?:\s+(.+))?"
    ).unwrap();
    
    let skip_line_regex = Regex::new(
        r"^\s*SKIP\s+\[\s*\]\s+(\S+)(?:\s+(.+))?"
    ).unwrap();
    
    let summary_regex = Regex::new(
        r"Summary\s+\[\s*([0-9.]+)s\]\s+(\d+)\s+tests?\s+run:\s+(\d+)\s+passed"
    ).unwrap();
    
    // Process all lines
    for line in output.lines() {
        // Parse regular test results (PASS/FAIL with timing)
        if let Some(caps) = test_line_regex.captures(line) {
            if caps[1].to_string() == "SKIP" {
                continue; // Skip lines are handled separately
            }
            
            let test_name = caps.get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "test".to_string());
            
            let result = TestResult {
                status: caps[1].to_string(),
                time: caps[2].to_string(),
                binary: caps[3].to_string(),
                test_name,
            };
            
            // Only add if we haven't seen this test before (deduplication)
            let test_id = result.unique_id();
            if !seen_tests.contains(&test_id) {
                seen_tests.insert(test_id);
                match result.status.as_str() {
                    "PASS" => results.passed.push(result),
                    "FAIL" => results.failed.push(result),
                    _ => {}
                }
            }
        }
        
        // Parse SKIP lines (no timing)
        if let Some(caps) = skip_line_regex.captures(line) {
            let test_name = caps.get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "skipped test".to_string());
            
            let result = TestResult {
                status: "SKIP".to_string(),
                time: String::new(),
                binary: caps[1].to_string(),
                test_name,
            };
            
            // Only add if we haven't seen this test before
            let test_id = result.unique_id();
            if !seen_tests.contains(&test_id) {
                seen_tests.insert(test_id);
                results.skipped.push(result);
            }
        }
        
        // Parse summary line for total time
        if let Some(caps) = summary_regex.captures(line) {
            results.total_time = caps[1].to_string();
        }
    }
    
    // If no time was captured but we have results, estimate it
    if results.total_time.is_empty() && !results.passed.is_empty() {
        let total_ms: f64 = results.passed.iter()
            .filter_map(|t| t.time.parse::<f64>().ok())
            .sum();
        results.total_time = format!("{:.3}", total_ms);
    }
    
    results
}

fn print_test_summary(results: &TestResults) {
    println!("========================== test session results ==========================");
    
    // Print all test results
    for test in &results.passed {
        println!("        PASS [{:>9}s] {} {}", 
                 test.time, test.binary, test.test_name);
    }
    
    for test in &results.failed {
        println!("        FAIL [{:>9}s] {} {}", 
                 test.time, test.binary, test.test_name);
    }
    
    for test in &results.skipped {
        println!("        SKIP [          ] {} {}", 
                 test.binary, test.test_name);
    }
    
    // Build summary line with accurate counts
    let mut summary_parts = Vec::new();
    
    let passed_count = results.passed.len();
    let failed_count = results.failed.len();
    let skipped_count = results.skipped.len();
    
    if passed_count > 0 {
        summary_parts.push(format!("{} passed", passed_count));
    }
    
    if failed_count > 0 {
        summary_parts.push(format!("{} failed", failed_count));
    }
    
    if skipped_count > 0 {
        summary_parts.push(format!("{} skipped", skipped_count));
    }
    
    let summary = if summary_parts.is_empty() {
        "no tests run".to_string()
    } else {
        summary_parts.join(", ")
    };
    
    let time = if results.total_time.is_empty() {
        "0.000s".to_string()
    } else {
        format!("{}s", results.total_time)
    };
    
    println!("========================== {} in {} ==========================", summary, time);
}
