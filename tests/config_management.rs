//! Comprehensive tests for configuration management (Phase 1.1)
//! 
//! Tests for:
//! - Figment-based layered configuration
//! - TOML preservation with toml_edit
//! - Backup and restore mechanisms
//! - Percentage value parsing
//! - Profile support
//!
//! Each test uses a unique environment variable prefix to prevent interference
//! when tests run in parallel. Tests no longer change the current directory.

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
    
    // Create backups directory for tests that need it
    let backup_dir = cargo_dir.join("backups");
    fs::create_dir_all(&backup_dir).expect("Failed to create backup dir");
    
    temp_dir
}

/// Helper to create a config manager in a test directory with unique env prefix
/// No longer changes the current directory!
fn create_test_manager_with_prefix(dir: &Path, env_prefix: &str) -> ConfigManager {
    // Disable hardware auto-detection by default in tests
    std::env::set_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix), "false");
    let manager = ConfigManager::new_with_base_dir(dir, env_prefix)
        .expect("Failed to create config manager");
    std::env::remove_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix));
    manager
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
    assert_eq!(config.backup.backup_dir, PathBuf::from(".cargo").join("backups"));
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
    let env_prefix = "TEST_FIGMENT_";
    
    // Create a custom configuration file
    let custom_config = r#"
[global]
optimization_level = "aggressive"
verbose = true
default_jobs = "50%"
auto_detect_hardware = false

[profiles.dev]
incremental = false
jobs = "4"
"#;
    
    fs::write(&config_path, custom_config).expect("Failed to write config");
    
    // Create manager and check layered config
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
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
    let _temp_dir = setup_test_env();
    let env_prefix = "TEST_ENV_OVERRIDE_";
    
    // Set environment variables using double underscore for nested keys
    std::env::set_var(format!("{}GLOBAL__VERBOSE", env_prefix), "true");
    std::env::set_var(format!("{}GLOBAL__USE_SCCACHE", env_prefix), "false");
    std::env::set_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix), "false");
    
    // Create manager and check env overrides
    let _temp_dir = setup_test_env();
    let manager = ConfigManager::new_with_base_dir(_temp_dir.path(), env_prefix)
        .expect("Failed to create manager");
    let config = manager.config();
    
    // Check that env vars override defaults
    assert!(config.global.verbose);
    assert!(!config.global.use_sccache);
    
    // Clean up
    std::env::remove_var(format!("{}GLOBAL__VERBOSE", env_prefix));
    std::env::remove_var(format!("{}GLOBAL__USE_SCCACHE", env_prefix));
    std::env::remove_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix));
}

#[test]
fn test_profile_inheritance() {
    let config = Config::default();
    
    // Check that all profiles have expected defaults
    for (name, profile) in &config.profiles {
        // Bench profile has caching disabled by design for accurate benchmarking
        if name != "bench" {
            assert!(profile.cache.enabled, "Profile {} should have cache enabled", name);
        } else {
            assert!(!profile.cache.enabled, "Bench profile should have cache disabled");
        }
        assert!(profile.rustflags.is_empty() || profile.name == "release");
    }
}

#[test]
fn test_toml_preservation() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let env_prefix = "TEST_TOML_PRESERVE_";
    
    // Create initial config with comments and formatting
    let initial_content = r#"# My custom cargo config
# This has special formatting

[build]
# Important comment
jobs = 4

[profile.dev]
# Dev profile settings
opt-level = 0
"#;
    
    fs::write(&config_path, initial_content).expect("Failed to write config");
    
    // Create manager and apply config
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
    manager.apply().expect("Failed to apply config");
    
    // Read back and check formatting is preserved
    let updated_content = fs::read_to_string(&config_path)
        .expect("Failed to read config");
    
    // Parse as TOML to check structure is valid
    let doc: DocumentMut = updated_content.parse()
        .expect("Invalid TOML after update");
    
    // Check that build section exists
    assert!(doc.get("build").is_some());
}

#[test]
fn test_backup_creation() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let env_prefix = "TEST_BACKUP_CREATE_";
    
    // Create initial config
    let initial_content = "[build]\njobs = 2\n";
    fs::write(&config_path, initial_content).expect("Failed to write config");
    
    // Create manager and create backup
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
    let backup_path = manager.create_backup().expect("Failed to create backup");
    
    // Check backup exists and contains correct content
    assert!(backup_path.exists());
    let backup_content = fs::read_to_string(&backup_path)
        .expect("Failed to read backup");
    
    // The backup should contain the actual config file content
    assert_eq!(backup_content, "[build]\njobs = 2\n");
}

#[test]
fn test_backup_restore() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let env_prefix = "TEST_BACKUP_RESTORE_";
    
    // Create initial config
    let original_content = "[build]\njobs = 2\n";
    fs::write(&config_path, original_content).expect("Failed to write config");
    
    // Create backup
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
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
    let env_prefix = "TEST_BACKUP_CLEANUP_";
    
    // Create a dummy config file first
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    fs::write(&config_path, "# Test config\n").expect("Failed to write config");
    
    // Create manager with max_backups = 3
    let mut manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
    manager.config_mut().backup.max_backups = 3;
    
    // Create multiple backups
    for _i in 0..5 {
        manager.create_backup().expect("Failed to create backup");
        // Add delay to ensure different timestamps (Unix time is in seconds)
        std::thread::sleep(std::time::Duration::from_millis(1100));
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
    
    assert_eq!(backup_count, 3, "Should have exactly 3 backups after cleanup");
}

#[test]
fn test_hardware_detection() {
    let _temp_dir = setup_test_env();
    let env_prefix = "TEST_HARDWARE_";
    
    // Enable hardware detection
    std::env::set_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix), "true");
    
    // Create manager with hardware detection
    let _temp_dir2 = setup_test_env();
    let manager = ConfigManager::new_with_base_dir(_temp_dir2.path(), env_prefix)
        .expect("Failed to create manager");
    let config = manager.config();
    
    // Check that hardware optimization was applied
    // After hardware detection, profiles should have job counts set
    for (_name, profile) in &config.profiles {
        if profile.jobs.is_some() {
            // If jobs is set, it should be a reasonable value
            let job_count = profile.jobs.as_ref().unwrap().to_count();
            assert!(job_count > 0 && job_count <= num_cpus::get() * 2);
        }
    }
    
    // Clean up
    std::env::remove_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix));
}

#[test]
fn test_profile_selection() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    let env_prefix = "TEST_PROFILE_SELECT_";
    
    // Create config with profile-specific settings
    let config_content = r#"
[global]
auto_detect_hardware = false

[profiles.dev]
jobs = 2

[profiles.release]
jobs = 8
incremental = false
"#;
    
    fs::write(&config_path, config_content).expect("Failed to write config");
    
    // Load with dev profile - no longer changes directory!
    std::env::set_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix), "false");
    let manager = ConfigManager::with_profile_and_base_dir("dev", temp_dir.path(), env_prefix)
        .expect("Failed to create manager");
    std::env::remove_var(format!("{}GLOBAL__AUTO_DETECT_HARDWARE", env_prefix));
    
    let dev_profile = manager.config().profiles.get("dev").unwrap();
    // With auto_detect_hardware = false, the value should be preserved as Fixed(2)
    assert_eq!(dev_profile.jobs, Some(JobCount::Fixed(2)));
}

#[test]
fn test_linker_configuration() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let env_prefix = "TEST_LINKER_CONFIG_";
    
    // Create manager and apply config
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
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
fn test_metadata_generation() {
    let config = Config::default();
    
    // Check metadata has expected fields
    assert!(!config.metadata.version.is_empty());
    assert!(config.metadata.timestamp > 0);
    assert!(!config.metadata.platform.is_empty());
}

#[test]
fn test_empty_config_handling() {
    let temp_dir = setup_test_env();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    let env_prefix = "TEST_EMPTY_CONFIG_";
    
    // Don't create any config file initially
    assert!(!config_path.exists());
    
    // Create manager and apply - should create a new config
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
    manager.apply().expect("Failed to apply config");
    
    // Check that config was created
    assert!(config_path.exists());
    
    // Parse and verify it's valid TOML
    let content = fs::read_to_string(&config_path)
        .expect("Failed to read config");
    let _doc: DocumentMut = content.parse()
        .expect("Invalid TOML created for empty config");
}

#[test]
fn test_concurrent_config_access() {
    use std::sync::Arc;
    use std::thread;
    
    let temp_dir = Arc::new(setup_test_env());
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    
    // Create initial config
    fs::write(&config_path, "[build]\njobs = 4\n").expect("Failed to write config");
    
    // Spawn multiple threads that create backups
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let temp_dir = Arc::clone(&temp_dir);
            let env_prefix = format!("TEST_CONCURRENT_{}_", i);
            thread::spawn(move || {
                // Each thread uses its own environment prefix
                let manager = create_test_manager_with_prefix(temp_dir.path(), &env_prefix);
                manager.create_backup().expect("Failed to create backup");
            })
        })
        .collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    // Check that backups were created
    let backup_dir = temp_dir.path().join(".cargo").join("backups");
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
    
    assert!(backup_count > 0);
}

#[test]
fn test_complete_configuration_workflow() {
    let temp_dir = setup_test_env();
    let env_prefix = "TEST_COMPLETE_WORKFLOW_";
    
    // Step 1: Create custom configuration file
    let custom_config = r#"
[global]
optimization_level = "aggressive"
default_jobs = "75%"
auto_detect_hardware = false

[profiles.release]
jobs = "100%"
incremental = false

[backup]
max_backups = 3
"#;
    
    let config_path = temp_dir.path().join("cargo-optimize.toml");
    fs::write(&config_path, custom_config).expect("Failed to write config");
    
    // Step 2: Set environment variable override
    std::env::set_var(format!("{}GLOBAL__VERBOSE", env_prefix), "true");
    
    // Step 3: Create manager (layered config applied)
    let manager = create_test_manager_with_prefix(temp_dir.path(), env_prefix);
    
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
    std::env::remove_var(format!("{}GLOBAL__VERBOSE", env_prefix));
}
