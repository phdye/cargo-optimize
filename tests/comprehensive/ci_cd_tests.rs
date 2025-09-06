//! CI/CD Integration Tests for cargo-optimize
//!
//! These tests verify that cargo-optimize works correctly in various
//! CI/CD environments and automated build systems.

use cargo_optimize::{auto_configure, optimize_with_config, Config, OptimizationLevel};
use std::env;
use std::fs;
use std::time::{Duration, Instant};
use tempfile::TempDir;


// Helper to simulate different CI environments
fn simulate_ci_environment(ci_type: &str) {
    // Clear existing CI variables
    let ci_vars = [
        "CI",
        "CONTINUOUS_INTEGRATION",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "JENKINS_URL",
        "CIRCLECI",
        "TRAVIS",
        "TF_BUILD",
        "BUILDKITE",
    ];

    for var in &ci_vars {
        env::remove_var(var);
    }

    // Set specific CI environment
    match ci_type {
        "github" => {
            env::set_var("CI", "true");
            env::set_var("GITHUB_ACTIONS", "true");
            env::set_var("GITHUB_WORKFLOW", "test");
            env::set_var("GITHUB_RUN_ID", "123456");
            env::set_var("GITHUB_SHA", "abcdef1234567890");
        }
        "gitlab" => {
            env::set_var("CI", "true");
            env::set_var("GITLAB_CI", "true");
            env::set_var("CI_JOB_ID", "12345");
            env::set_var("CI_PIPELINE_ID", "67890");
            env::set_var("CI_COMMIT_SHA", "abcdef1234567890");
        }
        "jenkins" => {
            env::set_var("JENKINS_URL", "http://jenkins.example.com");
            env::set_var("BUILD_NUMBER", "42");
            env::set_var("JOB_NAME", "cargo-optimize-test");
            env::set_var("WORKSPACE", "/var/jenkins/workspace");
        }
        "circleci" => {
            env::set_var("CI", "true");
            env::set_var("CIRCLECI", "true");
            env::set_var("CIRCLE_BUILD_NUM", "123");
            env::set_var("CIRCLE_PROJECT_REPONAME", "cargo-optimize");
        }
        "travis" => {
            env::set_var("CI", "true");
            env::set_var("TRAVIS", "true");
            env::set_var("TRAVIS_BUILD_NUMBER", "42");
            env::set_var("TRAVIS_JOB_NUMBER", "42.1");
        }
        "azure" => {
            env::set_var("TF_BUILD", "True");
            env::set_var("BUILD_BUILDNUMBER", "20230101.1");
            env::set_var("SYSTEM_TEAMPROJECT", "cargo-optimize");
        }
        _ => {
            env::set_var("CI", "true");
        }
    }
}

fn cleanup_ci_environment() {
    let ci_vars = [
        "CI",
        "CONTINUOUS_INTEGRATION",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "JENKINS_URL",
        "CIRCLECI",
        "TRAVIS",
        "TF_BUILD",
        "BUILDKITE",
        "GITHUB_WORKFLOW",
        "GITHUB_RUN_ID",
        "GITHUB_SHA",
        "CI_JOB_ID",
        "CI_PIPELINE_ID",
        "CI_COMMIT_SHA",
        "BUILD_NUMBER",
        "JOB_NAME",
        "WORKSPACE",
        "CIRCLE_BUILD_NUM",
        "CIRCLE_PROJECT_REPONAME",
        "TRAVIS_BUILD_NUMBER",
        "TRAVIS_JOB_NUMBER",
        "BUILD_BUILDNUMBER",
        "SYSTEM_TEAMPROJECT",
    ];

    for var in &ci_vars {
        env::remove_var(var);
    }
}

fn create_ci_test_project(name: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create a realistic project for CI testing
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["rt-multi-thread"] }}

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "ci_bench"
harness = false
"#,
        name
    );

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    // Create source files
    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();

    let main_rs = r#"//! CI test project

#[tokio::main]
async fn main() {
    println!("CI test project running");
    
    // Simulate some work
    for i in 0..100 {
        tokio::task::yield_now().await;
        let _ = i * i;
    }
    
    println!("Work completed");
}
"#;

    fs::write(src_dir.join("main.rs"), main_rs).unwrap();

    let lib_rs = r#"//! CI test library

/// Calculate factorial for CI testing
pub fn factorial(n: u32) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n as u64 * factorial(n - 1),
    }
}

/// Sum of squares for performance testing
pub fn sum_of_squares(limit: u32) -> u64 {
    (1..=limit).map(|x| (x as u64) * (x as u64)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
    }
    
    #[test]
    fn test_sum_of_squares() {
        assert_eq!(sum_of_squares(3), 14); // 1 + 4 + 9
    }
}
"#;

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();

    // Create benchmark
    let benches_dir = project_path.join("benches");
    fs::create_dir(&benches_dir).unwrap();

    let bench_rs = format!(
        r#"//! CI benchmarks

use criterion::{{criterion_group, criterion_main, Criterion}};
use {}::{{factorial, sum_of_squares}};

fn bench_factorial(c: &mut Criterion) {{
    c.bench_function("factorial 10", |b| b.iter(|| factorial(10)));
}}

fn bench_sum_of_squares(c: &mut Criterion) {{
    c.bench_function("sum of squares 100", |b| b.iter(|| sum_of_squares(100)));
}}

criterion_group!(benches, bench_factorial, bench_sum_of_squares);
criterion_main!(benches);
"#,
        name.replace("-", "_")
    );

    fs::write(benches_dir.join("ci_bench.rs"), bench_rs).unwrap();

    temp_dir
}

#[test]
fn test_github_actions_environment() {
    let temp_dir = create_ci_test_project("github-test");

    // Save original environment
    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    // Setup GitHub Actions environment
    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test optimization in GitHub Actions
    let start_time = Instant::now();
    auto_configure();
    let duration = start_time.elapsed();

    // Verify optimization completed
    assert!(
        cargo_optimize::is_optimized(),
        "Should be optimized in GitHub Actions"
    );

    // Should complete reasonably quickly in CI
    assert!(
        duration < Duration::from_secs(30),
        "GitHub Actions optimization took too long: {:?}",
        duration
    );

    // Check CI-specific optimizations
    let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
    assert!(!rustflags.is_empty(), "RUSTFLAGS should be set in CI");

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_gitlab_ci_environment() {
    let temp_dir = create_ci_test_project("gitlab-test");

    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    simulate_ci_environment("gitlab");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test optimization in GitLab CI
    auto_configure();

    assert!(
        cargo_optimize::is_optimized(),
        "Should be optimized in GitLab CI"
    );

    // GitLab CI should have parallel job settings
    let build_jobs = env::var("CARGO_BUILD_JOBS").unwrap_or_default();
    assert!(
        !build_jobs.is_empty(),
        "Build jobs should be configured in GitLab CI"
    );

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_jenkins_environment() {
    let temp_dir = create_ci_test_project("jenkins-test");

    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    simulate_ci_environment("jenkins");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test optimization in Jenkins
    auto_configure();

    assert!(
        cargo_optimize::is_optimized(),
        "Should be optimized in Jenkins"
    );

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_ci_specific_configurations() {
    let ci_environments = vec!["github", "gitlab", "jenkins", "circleci", "travis", "azure"];

    for ci_env in ci_environments {
        let temp_dir = create_ci_test_project(&format!("ci-{}", ci_env));

        let original_dir = env::current_dir().unwrap();
        simulate_ci_environment(ci_env);

        env::set_current_dir(temp_dir.path()).unwrap();
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");

        // Test with CI-optimized config
        let mut config = Config::default();
        config.set_optimization_level(OptimizationLevel::Balanced);
        // CI environments typically want fewer parallel jobs
        config.set_parallel_jobs(2);
        config.verbose(); // More logging in CI

        let result = optimize_with_config(config);

        assert!(
            result.is_ok(),
            "Optimization should succeed in {} CI",
            ci_env
        );
        assert!(
            cargo_optimize::is_optimized(),
            "Should be optimized in {} CI",
            ci_env
        );

        env::set_current_dir(original_dir).unwrap();
        cleanup_ci_environment();
    }
}

#[test]
fn test_ci_parallel_build_limits() {
    let temp_dir = create_ci_test_project("parallel-test");

    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test that CI doesn't use too many parallel jobs
    auto_configure();

    let build_jobs = env::var("CARGO_BUILD_JOBS").unwrap_or_default();
    if !build_jobs.is_empty() {
        let jobs: usize = build_jobs.parse().unwrap_or(1);
        // CI should limit parallel jobs to reasonable numbers
        assert!(jobs <= 8, "CI should limit parallel jobs, got {}", jobs);
        assert!(jobs >= 1, "CI should use at least 1 job");
    }

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_ci_cache_configuration() {
    let temp_dir = create_ci_test_project("cache-test");

    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test cache configuration in CI
    let mut config = Config::default();
    config.enable_cache = true;
    config.dry_run(); // Use dry run to avoid actually configuring cache

    let result = optimize_with_config(config);
    assert!(result.is_ok(), "Cache configuration should work in CI");

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_ci_timeout_handling() {
    let temp_dir = create_ci_test_project("timeout-test");

    let original_dir = env::current_dir().unwrap();
    let original_active = env::var("CARGO_OPTIMIZE_ACTIVE").ok();

    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test that optimization completes within reasonable time for CI
    let start_time = Instant::now();
    auto_configure();
    let duration = start_time.elapsed();

    // CI optimization should complete quickly
    assert!(
        duration < Duration::from_secs(60),
        "CI optimization should complete within 60 seconds, took {:?}",
        duration
    );

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
    if let Some(val) = original_active {
        env::set_var("CARGO_OPTIMIZE_ACTIVE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    }
}

#[test]
fn test_ci_with_matrix_builds() {
    // Simulate matrix build scenarios
    let test_matrix = vec![
        ("ubuntu-latest", "stable"),
        ("ubuntu-latest", "beta"),
        ("ubuntu-latest", "nightly"),
        ("windows-latest", "stable"),
        ("macos-latest", "stable"),
    ];

    for (os, toolchain) in test_matrix {
        let temp_dir = create_ci_test_project(&format!("matrix-{}-{}", os, toolchain));

        let original_dir = env::current_dir().unwrap();
        simulate_ci_environment("github");

        // Simulate matrix environment variables
        env::set_var("MATRIX_OS", os);
        env::set_var("MATRIX_RUST", toolchain);

        env::set_current_dir(temp_dir.path()).unwrap();
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");

        // Test optimization in matrix build
        let mut config = Config::default();

        // Adjust config based on matrix parameters
        match toolchain {
            "nightly" => config.set_optimization_level(OptimizationLevel::Aggressive),
            "beta" => config.set_optimization_level(OptimizationLevel::Balanced),
            _ => config.set_optimization_level(OptimizationLevel::Conservative),
        };

        let result = optimize_with_config(config);
        assert!(
            result.is_ok(),
            "Matrix build {}/{} should succeed",
            os,
            toolchain
        );

        env::set_current_dir(original_dir).unwrap();
        cleanup_ci_environment();
        env::remove_var("MATRIX_OS");
        env::remove_var("MATRIX_RUST");
    }
}

#[test]
fn test_ci_failure_recovery() {
    let temp_dir = create_ci_test_project("failure-test");

    let original_dir = env::current_dir().unwrap();
    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test with problematic configuration that might fail
    let mut config = Config::default();
    config.custom_linker = Some(std::path::PathBuf::from("/nonexistent/linker"));
    config.dry_run(); // Use dry run to avoid actual failure

    // Should handle gracefully without stopping the build
    let result = optimize_with_config(config);

    // The optimization might fail, but it shouldn't panic or crash the build
    match result {
        Ok(_) => {
            // Success is fine
        }
        Err(_) => {
            // Graceful failure is also acceptable in CI
            println!("Optimization failed gracefully in CI");
        }
    }

    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
}

#[test]
fn test_ci_build_artifacts() {
    let temp_dir = create_ci_test_project("artifacts-test");

    let original_dir = env::current_dir().unwrap();
    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Run optimization
    auto_configure();

    // Check that optimization doesn't interfere with build artifacts
    assert!(temp_dir.path().join("Cargo.toml").exists());
    assert!(temp_dir.path().join("src").exists());

    // If .cargo directory was created, it should be valid
    let cargo_dir = temp_dir.path().join(".cargo");
    if cargo_dir.exists() {
        let config_file = cargo_dir.join("config.toml");
        if config_file.exists() {
            let content = fs::read_to_string(&config_file).unwrap();
            assert!(!content.is_empty(), "Cargo config should not be empty");
        }
    }

    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
}

#[test]
fn test_ci_environment_variable_handling() {
    let temp_dir = create_ci_test_project("env-test");

    let original_dir = env::current_dir().unwrap();

    // Test various CI environment scenarios
    let ci_scenarios = vec![
        (
            "github",
            vec![
                ("GITHUB_ACTIONS", "true"),
                ("GITHUB_WORKSPACE", temp_dir.path().to_str().unwrap()),
                ("RUNNER_TEMP", "/tmp"),
            ],
        ),
        (
            "gitlab",
            vec![
                ("GITLAB_CI", "true"),
                ("CI_PROJECT_DIR", temp_dir.path().to_str().unwrap()),
                ("CI_BUILDS_DIR", "/builds"),
            ],
        ),
    ];

    for (ci_name, env_vars) in ci_scenarios {
        simulate_ci_environment(ci_name);

        // Set additional CI-specific variables
        for (key, value) in env_vars {
            env::set_var(key, value);
        }

        env::set_current_dir(temp_dir.path()).unwrap();
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");

        // Should handle CI environment variables correctly
        auto_configure();

        assert!(
            cargo_optimize::is_optimized(),
            "Should be optimized in {} CI environment",
            ci_name
        );

        cleanup_ci_environment();
    }

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_docker_container_builds() {
    let temp_dir = create_ci_test_project("docker-test");

    let original_dir = env::current_dir().unwrap();

    // Simulate Docker container environment
    env::set_var("CI", "true");
    // Docker environments often have these characteristics
    env::set_var("CONTAINER", "docker");
    env::set_var("USER", "root");
    env::set_var("HOME", "/root");

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Test optimization in container environment
    let mut config = Config::default();
    config.set_parallel_jobs(2); // Containers often have limited resources

    let result = optimize_with_config(config);
    assert!(result.is_ok(), "Should work in Docker container");

    env::set_current_dir(original_dir).unwrap();

    // Cleanup container environment variables
    env::remove_var("CI");
    env::remove_var("CONTAINER");
    env::remove_var("USER");
    env::remove_var("HOME");
}

#[test]
fn test_ci_with_custom_target_dir() {
    let temp_dir = create_ci_test_project("custom-target-test");

    let original_dir = env::current_dir().unwrap();
    let original_target_dir = env::var("CARGO_TARGET_DIR").ok();

    simulate_ci_environment("github");

    // Many CI systems use custom target directories
    let custom_target = temp_dir.path().join("custom-target");
    fs::create_dir(&custom_target).unwrap();
    env::set_var("CARGO_TARGET_DIR", custom_target.to_str().unwrap());

    env::set_current_dir(temp_dir.path()).unwrap();
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");

    // Should handle custom target directory correctly
    auto_configure();

    assert!(
        cargo_optimize::is_optimized(),
        "Should work with custom target dir"
    );

    // Restore environment
    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();

    if let Some(val) = original_target_dir {
        env::set_var("CARGO_TARGET_DIR", val);
    } else {
        env::remove_var("CARGO_TARGET_DIR");
    }
}

// Performance test specifically for CI environments
#[test]
fn test_ci_performance_characteristics() {
    let temp_dir = create_ci_test_project("perf-test");

    let original_dir = env::current_dir().unwrap();
    simulate_ci_environment("github");

    env::set_current_dir(temp_dir.path()).unwrap();

    const ITERATIONS: usize = 5;
    let mut times = Vec::new();

    for _ in 0..ITERATIONS {
        env::remove_var("CARGO_OPTIMIZE_ACTIVE");

        let start = Instant::now();
        auto_configure();
        let duration = start.elapsed();

        times.push(duration);
    }

    let avg_time: Duration = times.iter().sum::<Duration>() / ITERATIONS as u32;
    let max_time = times.iter().max().unwrap();

    // CI performance should be consistent and fast
    assert!(
        avg_time < Duration::from_secs(10),
        "CI average time should be under 10s, got {:?}",
        avg_time
    );
    assert!(
        *max_time < Duration::from_secs(20),
        "CI max time should be under 20s, got {:?}",
        max_time
    );

    env::set_current_dir(original_dir).unwrap();
    cleanup_ci_environment();
}
