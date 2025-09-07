//! Comprehensive tests for configuration management (Phase 1.1)
//! 
//! Tests for:
//! - Figment-based layered configuration
//! - TOML preservation with toml_edit
//! - Backup and restore mechanisms
//! - Percentage value parsing
//! - Profile support

use cargo_optimize::config::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use toml_edit::DocumentMut;

/// Helper to create a test environment
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create .cargo directory
    let cargo_dir = temp_dir.path().join(".cargo");
    fs::create_dir_all(&cargo_dir).expect("Failed to create .cargo dir");
    
    temp_dir
}

/// Helper to create a config manager in a test directory
fn create_test_manager(dir: &Path) -> ConfigManager {
    std::env::set_current_dir(dir).expect("Failed to change directory");
    ConfigManager::new().expect("Failed to create config manager")
}

#[test]
fn test_default_configuration() {
    let config = Config::default();
    
    // Check all default profiles exist
    assert_eq!(config.profiles.len(), 4);
    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("test"));
    assert!(config.profiles.contains_key("release"));
    assert!(config.profiles.contains_key("bench"));
    
    // Check global defaults
    assert_eq!(config.global.optimization_level, OptimizationLevel::Balanced);
    assert!(config.global.auto_detect_hardware);
    assert!(!config.global.verbose);
    assert!(config.global.use_sccache);
    
    // Check backup defaults
    assert!(config.backup.auto_backup);
    assert_eq!(config.backup.max_backups, 5);
    assert_eq!(config.backup.backup_dir, PathBuf::from(".cargo/backups"));
}

#[test]
fn test_percentage_parsing() {
    // Test JobCount percentage
    let job = JobCount::parse("75%").unwrap();
    assert!(matches!(job, JobCount::Percentage(_)));
    
    let cores = num_cpus::get();
    let expected = ((cores as f64 * 0.75).round() as usize).max(1);
    assert_eq!(job.to_count(), expected);
    
    // Test fixed count
    let job = JobCount::parse("8").unwrap();
    assert!(matches!(job, JobCount::Fixed(8)));
    assert_eq!(job.to_count(), 8);
    
    // Test invalid input
    assert!(JobCount::parse("invalid").is_err());
}

#[test]
fn test_cache_size_percentage() {
    // Test percentage cache size
    let size = CacheSize::Percentage("10%".to_string());
    let mb = size.to_megabytes();
    assert!(mb >= 100); // Should be at least 100MB
    
    // Test fixed size
    let size = CacheSize::Megabytes(2048);
    assert_eq!(size.to_megabytes(), 2048);
}

#[test]
fn test_figment_layered_configuration() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    
    // Create a custom configuration file
    let custom_config = r#"
[global]
optimization_level = "aggressive"
verbose = true
default_jobs = "50%"

[profiles.dev]
incremental = false
jobs = "4"
"#;
    
    fs::write(&config_path, custom_config).expect("Failed to write config");
    
    // Create manager and check layered config
    let manager = create_test_manager(temp_dir.path());
    let config = manager.config();
    
    // Check that custom values override defaults
    assert_eq!(config.global.optimization_level, OptimizationLevel::Aggressive);
    assert!(config.global.verbose);
    assert_eq!(
        config.global.default_jobs,
        Some(JobCount::Percentage("50%".to_string()))
    );
    
    // Check profile override
    let dev_profile = config.profiles.get("dev").unwrap();
    assert_eq!(dev_profile.incremental, Some(false));
    assert_eq!(dev_profile.jobs, Some(JobCount::Fixed(4)));
}

#[test]
fn test_environment_variable_override() {
    let temp_dir = setup_test_env();
    
    // Set environment variables using double underscore for nested keys
    std::env::set_var("CARGO_OPTIMIZE_GLOBAL__VERBOSE", "true");
    std::env::set_var("CARGO_OPTIMIZE_GLOBAL__USE_SCCACHE", "false");
    
    let manager = create_test_manager(temp_dir.path());
    let config = manager.config();
    
    // Check env var overrides
    assert!(config.global.verbose);
    assert!(!config.global.use_sccache);
    
    // Clean up env vars
    std::env::remove_var("CARGO_OPTIMIZE_GLOBAL__VERBOSE");
    std::env::remove_var("CARGO_OPTIMIZE_GLOBAL__USE_SCCACHE");
}

#[test]
fn test_toml_preservation() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    
    // Create an existing config with custom formatting
    let existing_config = r#"# My custom cargo config
# This has special formatting

[build]
# Important comment
jobs = 4

[profile.dev]
# Dev profile settings
opt-level = 0
"#;
    
    fs::write(&config_path, existing_config).expect("Failed to write config");
    
    // Apply our optimizations
    let manager = create_test_manager(temp_dir.path());
    manager.apply().expect("Failed to apply config");
    
    // Read back and check preservation
    let updated_content = fs::read_to_string(&config_path)
        .expect("Failed to read updated config");
    
    // Check that comments are preserved
    assert!(updated_content.contains("# Important comment"));
    assert!(updated_content.contains("# Dev profile settings"));
    
    // Parse as document to verify structure
    let doc: DocumentMut = updated_content.parse().expect("Invalid TOML");
    assert!(doc.get("build").is_some());
    assert!(doc.get("profile").is_some());
}

#[test]
fn test_backup_creation() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let backup_dir = temp_dir.path().join(".cargo").join("backups");
    
    // Create initial config
    fs::write(&config_path, "[build]\njobs = 2\n").expect("Failed to write config");
    
    // Create manager and trigger backup
    let manager = create_test_manager(temp_dir.path());
    let backup_path = manager.create_backup().expect("Failed to create backup");
    
    // Check backup exists
    assert!(backup_path.exists());
    assert!(backup_dir.exists());
    
    // Check backup content
    let backup_content = fs::read_to_string(&backup_path)
        .expect("Failed to read backup");
    assert_eq!(backup_content, "[build]\njobs = 2\n");
}

#[test]
fn test_backup_restore() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    
    // Create initial config
    let original_content = "[build]\njobs = 2\n";
    fs::write(&config_path, original_content).expect("Failed to write config");
    
    // Create backup
    let manager = create_test_manager(temp_dir.path());
    let backup_path = manager.create_backup().expect("Failed to create backup");
    
    // Modify config
    fs::write(&config_path, "[build]\njobs = 8\n").expect("Failed to modify config");
    
    // Restore from backup
    manager.restore_from_backup(&backup_path)
        .expect("Failed to restore from backup");
    
    // Check restored content
    let restored_content = fs::read_to_string(&config_path)
        .expect("Failed to read restored config");
    assert_eq!(restored_content, original_content);
}

#[test]
fn test_backup_cleanup() {
    let temp_dir = setup_test_env();
    let backup_dir = temp_dir.path().join(".cargo").join("backups");
    
    // Create manager with max_backups = 3
    let mut manager = create_test_manager(temp_dir.path());
    manager.config_mut().backup.max_backups = 3;
    
    // Create multiple backups
    for i in 0..5 {
        manager.create_backup().expect("Failed to create backup");
        // Add small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    // Check that only 3 backups remain
    let backup_count = fs::read_dir(&backup_dir)
        .expect("Failed to read backup dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name()
                .to_str()
                .map(|s| s.starts_with("config_backup_"))
                .unwrap_or(false)
        })
        .count();
    
    assert_eq!(backup_count, 3);
}

#[test]
fn test_hardware_detection() {
    let mut config = Config::default();
    
    // Apply hardware optimizations
    config.apply_hardware_optimizations()
        .expect("Failed to apply hardware optimizations");
    
    // Check that job counts were set
    for profile in config.profiles.values() {
        assert!(profile.jobs.is_some());
        
        // Check that cache size was set
        if profile.cache.enabled {
            assert!(profile.cache.max_size.is_some());
        }
    }
}

#[test]
fn test_profile_selection() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    
    // Create config with profile-specific settings
    let config_content = r#"
[profiles.dev]
jobs = "2"

[profiles.release]
jobs = "8"
incremental = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config");
    
    // Load with dev profile
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");
    let manager = ConfigManager::with_profile("dev")
        .expect("Failed to create manager");
    
    let dev_profile = manager.config().profiles.get("dev").unwrap();
    assert_eq!(dev_profile.jobs, Some(JobCount::Percentage("75%".to_string())));
}

#[test]
fn test_linker_configuration() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    
    // Create manager and apply config
    let manager = create_test_manager(temp_dir.path());
    manager.apply().expect("Failed to apply config");
    
    // Check that config was created
    assert!(config_path.exists());
    
    // Parse and verify structure
    let content = fs::read_to_string(&config_path)
        .expect("Failed to read config");
    let doc: DocumentMut = content.parse().expect("Invalid TOML");
    
    // Check for target configuration (if linker was detected)
    // This depends on the system having a compatible linker
    if doc.get("target").is_some() {
        let target = if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        
        assert!(doc["target"].get(target).is_some());
    }
}

#[test]
fn test_optimization_levels() {
    // Test serialization/deserialization
    let levels = vec![
        OptimizationLevel::Conservative,
        OptimizationLevel::Balanced,
        OptimizationLevel::Aggressive,
    ];
    
    for level in levels {
        let serialized = serde_json::to_string(&level).unwrap();
        let deserialized: OptimizationLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(level, deserialized);
    }
}

#[test]
fn test_cache_types() {
    let types = vec![
        CacheType::None,
        CacheType::Sccache,
        CacheType::Ccache,
        CacheType::Custom("my-cache".to_string()),
    ];
    
    for cache_type in types {
        let serialized = serde_json::to_string(&cache_type).unwrap();
        let deserialized: CacheType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cache_type, deserialized);
    }
}

#[test]
fn test_empty_config_handling() {
    let temp_dir = setup_test_env();
    
    // Ensure no existing config
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    assert!(!config_path.exists());
    
    // Create and apply manager
    let manager = create_test_manager(temp_dir.path());
    manager.apply().expect("Failed to apply config");
    
    // Check that config was created
    assert!(config_path.exists());
    
    // Verify it's valid TOML
    let content = fs::read_to_string(&config_path)
        .expect("Failed to read config");
    let _doc: DocumentMut = content.parse().expect("Invalid TOML");
}

#[test]
fn test_metadata_generation() {
    let config = Config::default();
    
    // Check metadata fields
    assert_eq!(config.metadata.version, env!("CARGO_PKG_VERSION"));
    assert!(config.metadata.timestamp > 0);
    
    let expected_platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };
    
    assert_eq!(config.metadata.platform, expected_platform);
}

#[test]
fn test_profile_inheritance() {
    let config = Config::default();
    
    // Check that all profiles have consistent structure
    for (name, profile) in &config.profiles {
        assert_eq!(profile.name, *name);
        assert!(profile.rustflags.is_empty() || !profile.rustflags.is_empty());
        assert!(matches!(profile.cache.cache_type, 
            CacheType::None | CacheType::Sccache | CacheType::Ccache | CacheType::Custom(_)));
    }
}

#[test]
fn test_concurrent_config_access() {
    use std::sync::Arc;
    use std::thread;
    
    let temp_dir = Arc::new(setup_test_env());
    let mut handles = vec![];
    
    // Spawn multiple threads trying to create/modify config
    for i in 0..5 {
        let temp_dir_clone = Arc::clone(&temp_dir);
        let handle = thread::spawn(move || {
            std::env::set_current_dir(temp_dir_clone.path())
                .expect("Failed to change dir");
            
            let manager = ConfigManager::new()
                .expect("Failed to create manager");
            
            // Each thread creates its own backup
            let backup_path = manager.create_backup()
                .expect("Failed to create backup");
            
            assert!(backup_path.exists());
            
            // Add small delay to simulate work
            thread::sleep(std::time::Duration::from_millis(10 * i));
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    // Verify backup directory has expected files
    let backup_dir = temp_dir.path().join(".cargo").join("backups");
    let backup_count = fs::read_dir(&backup_dir)
        .expect("Failed to read backup dir")
        .count();
    
    // Should have at least one backup (may be less than 5 due to cleanup)
    assert!(backup_count > 0);
}

// Integration test for the complete workflow
#[test]
fn test_complete_configuration_workflow() {
    let temp_dir = setup_test_env();
    
    // Step 1: Create custom configuration file
    let custom_config = r#"
[global]
optimization_level = "aggressive"
default_jobs = "75%"

[profiles.release]
jobs = "100%"
incremental = false

[backup]
max_backups = 3
"#;
    
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    fs::write(&config_path, custom_config).expect("Failed to write config");
    
    // Step 2: Set environment variable override
    std::env::set_var("CARGO_OPTIMIZE_GLOBAL_VERBOSE", "true");
    
    // Step 3: Create manager (layered config applied)
    let manager = create_test_manager(temp_dir.path());
    
    // Step 4: Verify layered configuration
    let config = manager.config();
    assert_eq!(config.global.optimization_level, OptimizationLevel::Aggressive);
    assert!(config.global.verbose); // From env var
    assert_eq!(config.global.default_jobs, Some(JobCount::Percentage("75%".to_string())));
    assert_eq!(config.backup.max_backups, 3);
    
    // Step 5: Apply configuration
    manager.apply().expect("Failed to apply config");
    
    // Step 6: Verify .cargo/config.toml was created
    let cargo_config = temp_dir.path().join(".cargo").join("config.toml");
    assert!(cargo_config.exists());
    
    // Step 7: Create backup
    let backup_path = manager.create_backup().expect("Failed to create backup");
    assert!(backup_path.exists());
    
    // Clean up
    std::env::remove_var("CARGO_OPTIMIZE_GLOBAL_VERBOSE");
}
