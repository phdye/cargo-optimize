//! Integration tests for cargo-optimize

use cargo_optimize::{Config, OptimizationLevel};
use std::env;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.optimization_level, OptimizationLevel::Balanced);
    assert!(config.auto_detect_hardware);
    assert!(config.analyze_project);
    assert!(config.optimize_linker);
    assert!(config.enable_cache);
}

#[test]
fn test_config_builder() {
    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(16)
        .verbose()
        .dry_run();

    assert_eq!(config.optimization_level, OptimizationLevel::Aggressive);
    assert_eq!(config.parallel_jobs, Some(16));
    assert!(config.verbose);
    assert!(config.dry_run);
}

#[test]
fn test_optimization_level_features() {
    use cargo_optimize::config::OptimizationFeature;

    // Conservative should enable only safe features
    let conservative = OptimizationLevel::Conservative;
    assert!(conservative.should_enable(OptimizationFeature::FastLinker));
    assert!(conservative.should_enable(OptimizationFeature::Sccache));
    assert!(!conservative.should_enable(OptimizationFeature::NativeCpu));

    // Aggressive should enable everything
    let aggressive = OptimizationLevel::Aggressive;
    assert!(aggressive.should_enable(OptimizationFeature::FastLinker));
    assert!(aggressive.should_enable(OptimizationFeature::NativeCpu));
    assert!(aggressive.should_enable(OptimizationFeature::ThinLto));
}

#[test]
fn test_is_optimized() {
    // Initially should not be optimized
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");
    assert!(!cargo_optimize::is_optimized());

    // Set the marker
    env::set_var("CARGO_OPTIMIZE_ACTIVE", "1");
    assert!(cargo_optimize::is_optimized());

    // Clean up
    env::remove_var("CARGO_OPTIMIZE_ACTIVE");
}

#[test]
fn test_version() {
    let version = cargo_optimize::version();
    assert!(!version.is_empty());
    assert!(version.starts_with("0."));
}

#[test]
#[ignore] // This test requires a real Rust project
fn test_auto_configure() {
    // Create a temporary project
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create a minimal Cargo.toml
    std::fs::write(
        project_path.join("Cargo.toml"),
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )
    .unwrap();

    // Create src/main.rs
    std::fs::create_dir(project_path.join("src")).unwrap();
    std::fs::write(
        project_path.join("src").join("main.rs"),
        "fn main() { println!(\"Hello, world!\"); }",
    )
    .unwrap();

    // Change to the project directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(project_path).unwrap();

    // Run auto_configure
    cargo_optimize::auto_configure();

    // Check that optimization marker is set
    assert!(cargo_optimize::is_optimized());

    // Restore original directory
    env::set_current_dir(original_dir).unwrap();
}
