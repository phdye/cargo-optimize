//! Debug test to understand Figment behavior

use cargo_optimize::config::*;
use figment::providers::{Format, Toml};
use figment::Figment;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_figment_directly() {
    // Create a temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    
    // Write config
    let config_content = r#"
[global]
optimization_level = "aggressive"
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");
    
    // Change to that directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    // Try to load with Figment directly
    let figment = Figment::new()
        .merge(Toml::string(r#"
[global]
optimization_level = "conservative"
"#))
        .merge(Toml::file("cargo-optimize.toml"));
    
    // Extract config
    #[derive(Debug, serde::Deserialize)]
    struct TestConfig {
        global: TestGlobal,
    }
    
    #[derive(Debug, serde::Deserialize)]
    struct TestGlobal {
        optimization_level: String,
    }
    
    let config: TestConfig = figment.extract().expect("Failed to extract");
    println!("Optimization level from Figment: {}", config.global.optimization_level);
    
    // Should be "aggressive" from the file, not "balanced" from the string
    assert_eq!(config.global.optimization_level, "aggressive");
}

#[test]
fn test_current_directory_affects_figment() {
    // Create two temp directories
    let temp_dir1 = TempDir::new().expect("Failed to create temp dir 1");
    let temp_dir2 = TempDir::new().expect("Failed to create temp dir 2");
    
    // Write different configs
    fs::write(
        temp_dir1.path().join("cargo-optimize.toml"),
        r#"[global]
optimization_level = "conservative""#
    ).expect("Failed to write config 1");
    
    fs::write(
        temp_dir2.path().join("cargo-optimize.toml"),
        r#"[global]
optimization_level = "aggressive""#
    ).expect("Failed to write config 2");
    
    // Load from dir1 - no need to change directory
    let manager1 = ConfigManager::new_with_base_dir(temp_dir1.path(), "FIGMENT_TEST_1_")
        .expect("Failed to create manager 1");
    assert_eq!(manager1.config().global.optimization_level, OptimizationLevel::Conservative);
    
    // Load from dir2 - no need to change directory
    let manager2 = ConfigManager::new_with_base_dir(temp_dir2.path(), "FIGMENT_TEST_2_")
        .expect("Failed to create manager 2");
    assert_eq!(manager2.config().global.optimization_level, OptimizationLevel::Aggressive);
}
