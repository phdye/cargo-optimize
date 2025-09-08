#[test]
fn debug_figment_loading() {
    use cargo_optimize::config::*;
    use std::fs;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    
    // Write config
    let config_content = r#"
[global]
optimization_level = "aggressive"
auto_detect_hardware = false
"#;
    fs::write(&config_path, config_content).expect("Failed to write config");
    
    println!("Config path: {:?}", config_path);
    println!("Config exists: {}", config_path.exists());
    println!("Config content:\n{}", fs::read_to_string(&config_path).unwrap());
    
    // No need to change directory - use base_dir instead
    println!("Temp dir: {:?}", temp_dir.path());
    
    // Set env var
    std::env::set_var("TEST_DEBUG_GLOBAL__AUTO_DETECT_HARDWARE", "false");
    
    // Create manager with base directory
    let manager = ConfigManager::new_with_base_dir(temp_dir.path(), "TEST_DEBUG_")
        .expect("Failed to create manager");
    
    let config = manager.config();
    println!("Loaded optimization level: {:?}", config.global.optimization_level);
    println!("Loaded auto_detect_hardware: {}", config.global.auto_detect_hardware);
    
    // Clean up
    std::env::remove_var("TEST_DEBUG_GLOBAL__AUTO_DETECT_HARDWARE");
    
    assert_eq!(config.global.optimization_level, OptimizationLevel::Aggressive);
}
