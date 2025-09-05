//! Comprehensive integration tests for cargo-optimize

use cargo_optimize::{auto_configure, optimize_with_config, Config, OptimizationLevel};

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::env;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::fs;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}

use std::path::{Path, PathBuf};

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}


use tempfile::TempDir;

fn create_test_metadata() -> cargo_metadata::Metadata {
    use cargo_metadata::{Package, PackageId, Version};
    cargo_metadata::Metadata {
        packages: vec![],
        workspace_members: vec![],
        resolve: None,
        root: std::env::current_dir().unwrap_or_default().into(),
        metadata: None,
        version: 1,
        workspace_root: std::env::current_dir().unwrap_or_default().into(),
        target_directory: std::env::current_dir().unwrap_or_default().join("target").into(),
    }
}


/// Helper to create a test project
fn create_test_project(root: &Path, name: &str) -> PathBuf {
    let project_dir = root.join(name);
    fs::create_dir_all(&project_dir).unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        name
    );

    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let main_rs = r#"
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#;

    fs::write(src_dir.join("main.rs"), main_rs).unwrap();

    project_dir
}

#[test]
fn test_end_to_end_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "test_project");

    // Save original environment
    let original_rustflags = env::var("RUSTFLAGS").ok();
    let original_cargo_build_jobs = env::var("CARGO_BUILD_JOBS").ok();

    // Change to project directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Run auto-configuration
    auto_configure();

    // Check that optimization marker is set
    assert_eq!(
        env::var("CARGO_OPTIMIZE_ACTIVE").ok(),
        Some("1".to_string())
    );

    // Restore environment
    env::set_current_dir(original_dir).unwrap();

    if let Some(val) = original_rustflags {
        env::set_var("RUSTFLAGS", val);
    } else {
        env::remove_var("RUSTFLAGS");
    }

    if let Some(val) = original_cargo_build_jobs {
        env::set_var("CARGO_BUILD_JOBS", val);
    } else {
        env::remove_var("CARGO_BUILD_JOBS");
    }

    env::remove_var("CARGO_OPTIMIZE_ACTIVE");
}

#[test]
fn test_real_project_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "analysis_project");

    // Add some dependencies
    let cargo_toml = r#"[package]
name = "analysis_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
criterion = "0.5"
"#;

    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Add more source files
    let src_dir = project_dir.join("src");

    let lib_rs = r#"
pub mod utils;
pub mod core;

pub fn process_data(input: &str) -> String {
    input.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process() {
        assert_eq!(process_data("hello"), "HELLO");
    }
}
"#;

    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap();
    fs::write(src_dir.join("utils.rs"), "// Utils module\n").unwrap();
    fs::write(src_dir.join("core.rs"), "// Core module\n").unwrap();

    // Create tests directory
    let tests_dir = project_dir.join("tests");
    fs::create_dir_all(&tests_dir).unwrap();

    let integration_test = r#"
#[test]
fn test_integration() {
    assert!(true);
}
"#;

    fs::write(tests_dir.join("integration.rs"), integration_test).unwrap();

    // Change to project directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Run optimization with analysis
    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Balanced)
        .set_auto_detect(true)
        .dry_run(); // Use dry_run to avoid side effects

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    // Restore directory
    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_linker_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "linker_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.optimize_linker = true;
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_workspace_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_dir = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace_dir).unwrap();

    // Create workspace Cargo.toml
    let workspace_toml = r#"
[workspace]
members = ["crate_a", "crate_b"]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = "1.0"
"#;

    fs::write(workspace_dir.join("Cargo.toml"), workspace_toml).unwrap();

    // Create member crates
    for name in &["crate_a", "crate_b"] {
        let crate_dir = workspace_dir.join(name);
        fs::create_dir_all(&crate_dir.join("src")).unwrap();

        let crate_toml = format!(
            r#"[package]
name = "{}"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
"#,
            name
        );

        fs::write(crate_dir.join("Cargo.toml"), crate_toml).unwrap();

        let lib_rs = format!("// {} library", name);
        fs::write(crate_dir.join("src/lib.rs"), lib_rs).unwrap();
    }

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&workspace_dir).unwrap();

    let mut config = Config::new();
    config
        .set_optimization_level(OptimizationLevel::Balanced)
        .dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_incremental_compilation_setup() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "incremental_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Save original
    let original_incremental = env::var("CARGO_INCREMENTAL").ok();

    let mut config = Config::new();
    config.incremental = true;
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    // Restore
    env::set_current_dir(original_dir).unwrap();
    if let Some(val) = original_incremental {
        env::set_var("CARGO_INCREMENTAL", val);
    } else {
        env::remove_var("CARGO_INCREMENTAL");
    }
}

#[test]
fn test_cache_configuration_integration() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "cache_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.enable_cache = true;
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parallel_jobs_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "parallel_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Save original
    let original_jobs = env::var("CARGO_BUILD_JOBS").ok();

    let mut config = Config::new();
    config.set_parallel_jobs(4);
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    // Restore
    env::set_current_dir(original_dir).unwrap();
    if let Some(val) = original_jobs {
        env::set_var("CARGO_BUILD_JOBS", val);
    } else {
        env::remove_var("CARGO_BUILD_JOBS");
    }
}

#[test]
fn test_profile_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "profile_project");

    // Create Cargo.toml with existing profiles
    let cargo_toml = r#"[package]
name = "profile_project"
version = "0.1.0"
edition = "2021"

[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
"#;

    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.set_optimization_level(OptimizationLevel::Balanced);
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_all_optimization_levels() {
    let levels = vec![
        OptimizationLevel::Conservative,
        OptimizationLevel::Balanced,
        OptimizationLevel::Aggressive,
        OptimizationLevel::Custom,
    ];

    for level in levels {
        let temp_dir = TempDir::new().unwrap();
        let project_dir =
            create_test_project(temp_dir.path(), &format!("level_{:?}_project", level));

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&project_dir).unwrap();

        let mut config = Config::new();
        config.set_optimization_level(level);
        config.dry_run();

        let result = optimize_with_config(config);
        assert!(result.is_ok(), "Failed for level {:?}", level);

        env::set_current_dir(original_dir).unwrap();
    }
}

#[test]
fn test_build_script_integration() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "build_script_project");

    // Create build.rs
    let build_rs = r#"
fn main() {
    // Simulate build script that would call cargo_optimize
    println!("cargo:rerun-if-changed=build.rs");
}
"#;

    fs::write(project_dir.join("build.rs"), build_rs).unwrap();

    // Update Cargo.toml
    let cargo_toml = r#"[package]
name = "build_script_project"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]

[build-dependencies]
"#;

    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_large_dependency_tree() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "deps_project");

    // Create project with many dependencies
    let cargo_toml = r#"[package]
name = "deps_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["rt", "macros"] }
reqwest = { version = "0.12", features = ["json"] }
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"
"#;

    fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.set_optimization_level(OptimizationLevel::Aggressive);
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_ci_environment_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "ci_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Simulate CI environment
    let original_ci = env::var("CI").ok();
    env::set_var("CI", "true");

    let mut config = Config::new();
    config.set_auto_detect(true);
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    // Restore
    env::set_current_dir(original_dir).unwrap();
    if let Some(val) = original_ci {
        env::set_var("CI", val);
    } else {
        env::remove_var("CI");
    }
}

#[test]
fn test_custom_flags_integration() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "flags_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // Save originals
    let original_rustflags = env::var("RUSTFLAGS").ok();
    let original_cargo_flags = env::var("CARGO_BUILD_FLAGS").ok();

    let mut config = Config::new();
    config.extra_cargo_flags = vec!["--verbose".to_string()];
    config.extra_rustc_flags = vec!["-C".to_string(), "opt-level=2".to_string()];
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    // Restore
    env::set_current_dir(original_dir).unwrap();
    if let Some(val) = original_rustflags {
        env::set_var("RUSTFLAGS", val);
    } else {
        env::remove_var("RUSTFLAGS");
    }
    if let Some(val) = original_cargo_flags {
        env::set_var("CARGO_BUILD_FLAGS", val);
    } else {
        env::remove_var("CARGO_BUILD_FLAGS");
    }
}

#[test]
fn test_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "verbose_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.verbose();
    config.dry_run();

    let result = optimize_with_config(config);
    assert!(result.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

// Test optimization persistence
#[test]
fn test_optimization_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(temp_dir.path(), "persist_project");

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    // First optimization
    let mut config = Config::new();
    config.set_optimization_level(OptimizationLevel::Balanced);
    config.dry_run();

    let result1 = optimize_with_config(config.clone());
    assert!(result1.is_ok());

    // Second optimization (should be idempotent)
    let result2 = optimize_with_config(config);
    assert!(result2.is_ok());

    env::set_current_dir(original_dir).unwrap();
}

// Test error recovery
#[test]
fn test_missing_cargo_toml_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("no_cargo");
    fs::create_dir_all(&project_dir).unwrap();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&project_dir).unwrap();

    let mut config = Config::new();
    config.dry_run();

    let result = optimize_with_config(config);
    // Should fail gracefully
    assert!(result.is_err());

    env::set_current_dir(original_dir).unwrap();
}
