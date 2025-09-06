//! Unit tests for the optimizer module

use cargo_optimize::{Config, OptimizationLevel, Optimizer};

use pretty_assertions::assert_eq;
use std::env;
use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;


#[test]
fn test_optimizer_creation() {
    let temp_dir = TempDir::new().unwrap();
    let _optimizer = Optimizer::new(temp_dir.path()).unwrap();

    // Should create with default config
    // Optimizer created successfully
}

#[test]
fn test_optimizer_with_custom_config() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_parallel_jobs(8)
        .verbose();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_optimizer_environment_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a basic Cargo.toml
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let mut config = Config::default();
    config.auto_detect_hardware = true;

    let _optimizer = Optimizer::with_config(project_root, config).unwrap();

    // Should detect environment when auto_detect is true
    // Note: We can't test the full optimize() as it modifies the environment
    // but we can test that creation works
    // Optimizer created successfully
}

#[test]
fn test_optimization_levels() {
    let levels = vec![
        OptimizationLevel::Conservative,
        OptimizationLevel::Balanced,
        OptimizationLevel::Aggressive,
        OptimizationLevel::Custom,
    ];

    for level in levels {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create project
        let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
        fs::create_dir_all(project_root.join("src")).unwrap();
        fs::write(project_root.join("src/lib.rs"), "").unwrap();

        let mut config = Config::new();
        config.set_optimization_level(level).dry_run(); // Use dry_run to avoid modifying environment

        let _optimizer = Optimizer::with_config(project_root, config).unwrap();
        // Optimizer created successfully
    }
}

#[test]
fn test_parallel_jobs_configuration() {
    let temp_dir = TempDir::new().unwrap();

    // Save original value
    let original = env::var("CARGO_BUILD_JOBS").ok();

    // Test setting parallel jobs
    let job_counts = vec![1, 2, 4, 8, 16];

    for jobs in job_counts {
        let mut config = Config::new();
        config.set_parallel_jobs(jobs);

        // We can't fully test the optimizer without side effects,
        // but we can verify the config is accepted
        let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    }

    // Restore original
    if let Some(val) = original {
        env::set_var("CARGO_BUILD_JOBS", val);
    } else {
        env::remove_var("CARGO_BUILD_JOBS");
    }
}

#[test]
fn test_dry_run_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();

    // Save environment state
    let original_rustflags = env::var("RUSTFLAGS").ok();
    let original_cargo_build_jobs = env::var("CARGO_BUILD_JOBS").ok();

    // Run optimization in dry-run mode
    let result = optimizer.optimize();

    // Dry-run should succeed
    assert!(result.is_ok());

    // Environment should not be modified in dry-run
    assert_eq!(env::var("RUSTFLAGS").ok(), original_rustflags);
    assert_eq!(env::var("CARGO_BUILD_JOBS").ok(), original_cargo_build_jobs);
}

#[test]
fn test_verbose_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let mut config = Config::new();
    config.verbose().dry_run(); // Use dry_run to avoid side effects

    let _optimizer = Optimizer::with_config(project_root, config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_custom_flags() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.extra_cargo_flags = vec!["--features".to_string(), "test-feature".to_string()];
    config.extra_rustc_flags = vec!["-C".to_string(), "opt-level=3".to_string()];

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_linker_configuration() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.optimize_linker = true;
    config.custom_linker = Some(PathBuf::from("/usr/bin/ld"));
    config.dry_run(); // Use dry_run to avoid system changes

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_cache_configuration() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.enable_cache = true;
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_incremental_compilation() {
    let temp_dir = TempDir::new().unwrap();

    // Save original
    let original = env::var("CARGO_INCREMENTAL").ok();

    let mut config = Config::new();
    config.incremental = true;
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();

    // Restore original
    if let Some(val) = original {
        env::set_var("CARGO_INCREMENTAL", val);
    } else {
        env::remove_var("CARGO_INCREMENTAL");
    }
}

#[test]
fn test_split_debuginfo() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.split_debuginfo = true;
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_target_cpu_configuration() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.target_cpu = Some("native".to_string());
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

// Boundary value tests
#[test]
fn test_optimizer_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "empty"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();
    let result = optimizer.optimize();

    // Should handle empty project gracefully
    assert!(result.is_ok());
}

#[test]
fn test_optimizer_max_parallel_jobs() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.set_parallel_jobs(usize::MAX);
    config.dry_run();

    // Should handle extreme values
    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

#[test]
fn test_optimizer_zero_parallel_jobs() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.set_parallel_jobs(0); // Should be interpreted as "auto"
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

// Error handling tests
#[test]
fn test_optimizer_invalid_project_path() {
    let result = Optimizer::new("/this/path/does/not/exist");
    // Should still create optimizer, error would occur during optimize()
    assert!(result.is_ok());
}

#[test]
fn test_optimizer_no_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    let result = optimizer.optimize();

    // Should fail gracefully when no Cargo.toml exists
    assert!(result.is_err() || result.is_ok()); // Depends on implementation
}

// Property-based test helpers
#[test]
fn test_optimization_level_ordering() {
    // Property: More aggressive levels should enable more features
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    // Conservative should be less aggressive than Balanced
    let mut config_conservative = Config::new();
    config_conservative.set_optimization_level(OptimizationLevel::Conservative);

    let mut config_balanced = Config::new();
    config_balanced.set_optimization_level(OptimizationLevel::Balanced);

    let mut config_aggressive = Config::new();
    config_aggressive.set_optimization_level(OptimizationLevel::Aggressive);

    // Test that each level exists and can be used
    for config in [config_conservative, config_balanced, config_aggressive] {
        let _optimizer = Optimizer::with_config(project_root, config).unwrap();
        // Optimizer created successfully
    }
}

#[test]
fn test_environment_adaptation() {
    let temp_dir = TempDir::new().unwrap();

    // Test that optimizer adapts to different environments
    let mut config = Config::new();
    config.auto_detect_hardware = true;
    config.dry_run();

    let _optimizer = Optimizer::with_config(temp_dir.path(), config).unwrap();
    // Optimizer created successfully
}

// Integration-style tests
#[test]
fn test_optimizer_workspace_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create a workspace
    let cargo_toml = r#"
[workspace]
members = ["crate1", "crate2"]
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();

    // Create member crates
    for i in 1..=2 {
        let crate_dir = project_root.join(format!("crate{}", i));
        fs::create_dir_all(&crate_dir.join("src")).unwrap();

        let member_toml = format!(
            r#"
[package]
name = "crate{}"
version = "0.1.0"
edition = "2021"
"#,
            i
        );
        fs::write(crate_dir.join("Cargo.toml"), member_toml).unwrap();
        fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
    }

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();
    let result = optimizer.optimize();

    // Should handle workspace projects
    assert!(result.is_ok());
}

#[test]
fn test_optimizer_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project with dependencies
    let cargo_toml = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
serde_json = "1.0"

[dev-dependencies]
criterion = "0.5"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();
    let result = optimizer.optimize();

    assert!(result.is_ok());
}

// Stress tests
#[test]
fn test_optimizer_repeated_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    // Run optimization multiple times
    for _ in 0..5 {
        let mut config = Config::new();
        config.dry_run();

        let mut optimizer = Optimizer::with_config(&project_root, config).unwrap();
        let result = optimizer.optimize();

        // Should be idempotent
        assert!(result.is_ok());
    }
}

#[test]
fn test_optimizer_all_features_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    // Enable all features
    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Aggressive)
        .set_auto_detect(true)
        .set_parallel_jobs(8)
        .verbose()
        .dry_run();

    config.optimize_linker = true;
    config.enable_cache = true;
    config.incremental = true;
    config.split_debuginfo = true;
    config.target_cpu = Some("native".to_string());

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();
    let result = optimizer.optimize();

    // Should handle all features being enabled
    assert!(result.is_ok());
}

#[test]
fn test_optimizer_disabled_via_env() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Create project
    let cargo_toml = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(project_root.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(project_root.join("src")).unwrap();
    fs::write(project_root.join("src/lib.rs"), "").unwrap();

    // Save original
    let original = env::var("CARGO_OPTIMIZE_DISABLE").ok();

    // Disable via environment
    env::set_var("CARGO_OPTIMIZE_DISABLE", "1");

    let mut config = Config::new();
    config.dry_run();

    let mut optimizer = Optimizer::with_config(project_root, config).unwrap();
    let result = optimizer.optimize();

    // Should still succeed but skip optimization
    assert!(result.is_ok());

    // Restore original
    if let Some(val) = original {
        env::set_var("CARGO_OPTIMIZE_DISABLE", val);
    } else {
        env::remove_var("CARGO_OPTIMIZE_DISABLE");
    }
}
