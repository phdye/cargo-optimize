#[test]
fn debug_config_loading() {
    use cargo_optimize::config::*;
    use std::fs;
    use tempfile::TempDir;
    
    // Create a temp directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    
    // Write a test config
    let config_content = r#"
[global]
optimization_level = "aggressive"
auto_detect_hardware = false

[profiles.dev]
jobs = 2
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config");
    
    // Change to that directory  
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    
    // Try to load the config
    println!("Current dir: {:?}", std::env::current_dir());
    println!("Config file exists: {}", config_path.exists());
    println!("Config content:\n{}", fs::read_to_string(&config_path).unwrap());
    
    // Set env var to ensure no hardware detection
    std::env::set_var("CARGO_OPTIMIZE_GLOBAL__AUTO_DETECT_HARDWARE", "false");
    
    // Create manager
    let manager = ConfigManager::with_profile("dev")
        .expect("Failed to create manager");
    
    let config = manager.config();
    
    // Print what we got
    println!("Optimization level: {:?}", config.global.optimization_level);
    println!("Auto detect hardware: {}", config.global.auto_detect_hardware);
    println!("Number of profiles: {}", config.profiles.len());
    
    for (name, profile) in &config.profiles {
        println!("Profile '{}': jobs = {:?}", name, profile.jobs);
    }
    
    // Clean up
    std::env::remove_var("CARGO_OPTIMIZE_GLOBAL__AUTO_DETECT_HARDWARE");
    
    // Do assertions
    assert_eq!(config.global.optimization_level, OptimizationLevel::Aggressive, 
        "Optimization level should be aggressive");
    assert!(!config.global.auto_detect_hardware, 
        "Hardware detection should be disabled");
    assert_eq!(config.profiles.get("dev").unwrap().jobs, Some(JobCount::Fixed(2)),
        "Dev profile should have jobs = 2");
}
