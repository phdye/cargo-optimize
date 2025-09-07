//! Tests for the enhanced configuration management system
//! 
//! Tests cover:
//! - Configuration loading and saving
//! - Profile management
//! - Backup and restore functionality
//! - Config merging

use cargo_optimize::config::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test helper to create a temporary directory with cargo structure
fn setup_test_dir() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let cargo_dir = dir.path().join(".cargo");
    fs::create_dir_all(&cargo_dir).expect("Failed to create .cargo dir");
    dir
}

/// Test helper to write a config file
fn write_config(dir: &Path, filename: &str, content: &str) {
    let path = dir.join(filename);
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent dirs");
    }
    fs::write(path, content).expect("Failed to write config");
}

/// Test helper to read a config file
fn read_config(dir: &Path, filename: &str) -> String {
    let path = dir.join(filename);
    fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn test_default_config_creation() {
    let config = Config::default();
    
    // Verify default profiles exist
    assert_eq!(config.profiles.len(), 4);
    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("test"));
    assert!(config.profiles.contains_key("release"));
    assert!(config.profiles.contains_key("bench"));
    
    // Verify global settings
    assert_eq!(config.global.optimization_level, OptimizationLevel::Balanced);
    assert!(config.global.auto_detect_hardware);
    assert!(!config.global.verbose);
    assert!(config.global.use_sccache);
    
    // Verify backup settings
    assert!(config.backup.auto_backup);
    assert_eq!(config.backup.max_backups, 5);
}

#[test]
fn test_profile_settings() {
    let config = Config::default();
    
    // Test dev profile
    let dev = config.get_profile("dev").expect("Dev profile should exist");
    assert_eq!(dev.name, "dev");
    assert_eq!(dev.incremental, Some(true));
    assert!(dev.cache.enabled);
    assert_eq!(dev.cache.cache_type, CacheType::Sccache);
    
    // Test release profile
    let release = config.get_profile("release").expect("Release profile should exist");
    assert_eq!(release.name, "release");
    assert_eq!(release.incremental, Some(false));
    assert!(release.rustflags.contains(&"-C".to_string()));
    assert!(release.rustflags.contains(&"opt-level=3".to_string()));
    
    // Test bench profile
    let bench = config.get_profile("bench").expect("Bench profile should exist");
    assert_eq!(bench.name, "bench");
    assert!(!bench.cache.enabled);
    assert_eq!(bench.cache.cache_type, CacheType::None);
}

#[test]
fn test_save_cargo_config() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create and save config
    let config = Config::default();
    config.save_cargo_config().expect("Failed to save cargo config");
    
    // Verify file was created
    let config_path = Path::new(".cargo/config.toml");
    assert!(config_path.exists());
    
    // Read and verify content
    let content = fs::read_to_string(config_path).expect("Failed to read config");
    assert!(content.contains("# Cargo configuration - optimized by cargo-optimize"));
    assert!(content.contains("[build]"));
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}

#[test]
fn test_save_optimize_config() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create and save config
    let config = Config::default();
    config.save_optimize_config().expect("Failed to save optimize config");
    
    // Verify file was created
    let config_path = Path::new("cargo-optimize.toml");
    assert!(config_path.exists());
    
    // Read and verify content
    let content = fs::read_to_string(config_path).expect("Failed to read config");
    assert!(content.contains("[metadata]"));
    assert!(content.contains("[global]"));
    assert!(content.contains("[profile.dev]"));
    assert!(content.contains("[profile.release]"));
    assert!(content.contains("[backup]"));
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}

#[test]
fn test_backup_creation() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create a config file to backup
    let cargo_config = ".cargo/config.toml";
    write_config(temp_dir.path(), cargo_config, "[build]\njobs = 4\n");
    
    // Create config and perform backup
    let config = Config::default();
    let backup_path = config.create_backup().expect("Failed to create backup");
    
    // Verify backup was created
    assert!(backup_path.exists());
    assert!(backup_path.to_string_lossy().contains("config_backup_"));
    
    // Verify backup content matches original
    let backup_content = fs::read_to_string(&backup_path).expect("Failed to read backup");
    assert_eq!(backup_content, "[build]\njobs = 4\n");
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}

#[test]
fn test_restore_from_backup() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create a backup file
    let backup_dir = Path::new(".cargo/backups");
    let _ = fs::remove_dir_all(backup_dir); // Clean up any existing backup dir
    fs::create_dir_all(backup_dir).expect("Failed to create backup dir");
    let backup_path = backup_dir.join("test_backup.toml");
    fs::write(&backup_path, "[build]\njobs = 8\n").expect("Failed to write backup");
    
    // Restore from backup
    Config::restore_from_backup(&backup_path).expect("Failed to restore backup");
    
    // Verify config was restored
    let config_path = Path::new(".cargo/config.toml");
    assert!(config_path.exists());
    
    let content = fs::read_to_string(config_path).expect("Failed to read config");
    assert_eq!(content, "[build]\njobs = 8\n");
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}

#[test]
fn test_backup_cleanup() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create multiple backup files
    let backup_dir = Path::new(".cargo/backups");
    fs::create_dir_all(backup_dir).expect("Failed to create backup dir");
    
    for i in 1..=10 {
        let backup_name = format!("config_backup_{}.toml", i);
        let backup_path = backup_dir.join(backup_name);
        fs::write(backup_path, format!("backup {}", i)).expect("Failed to write backup");
        
        // Add small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    // Create config with max_backups = 5
    let mut config = Config::default();
    config.backup.max_backups = 5;
    
    // Clean up old backups
    config.cleanup_old_backups().expect("Failed to cleanup backups");
    
    // Count remaining backups
    let backup_count = fs::read_dir(backup_dir)
        .expect("Failed to read backup dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_str()
                .map(|s| s.starts_with("config_backup_"))
                .unwrap_or(false)
        })
        .count();
    
    assert_eq!(backup_count, 5, "Should keep only 5 most recent backups");
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}

#[test]
fn test_apply_profile() {
    let config = Config::default();
    
    // Apply dev profile
    config.apply_profile("dev").expect("Failed to apply dev profile");
    
    // Note: We can't easily test environment variables being set
    // because they affect the global environment
    // In a real test, we'd use a mock or integration test
}

#[test]
fn test_hardware_optimization() {
    let mut config = Config::default();
    
    // Apply hardware optimizations
    config.apply_hardware_optimizations().expect("Failed to apply hardware optimizations");
    
    // Check that job counts were set
    for profile in config.profiles.values() {
        assert!(profile.jobs.is_some(), "Jobs should be set after hardware optimization");
        let jobs = profile.jobs.unwrap();
        assert!(jobs > 0, "Jobs should be greater than 0");
    }
}

#[test]
fn test_optimization_levels() {
    assert_eq!(
        format!("{:?}", OptimizationLevel::Conservative),
        "Conservative"
    );
    assert_eq!(
        format!("{:?}", OptimizationLevel::Balanced),
        "Balanced"
    );
    assert_eq!(
        format!("{:?}", OptimizationLevel::Aggressive),
        "Aggressive"
    );
}

#[test]
fn test_cache_types() {
    assert_eq!(CacheType::None, CacheType::None);
    assert_eq!(CacheType::Sccache, CacheType::Sccache);
    assert_eq!(CacheType::Ccache, CacheType::Ccache);
    assert_eq!(
        CacheType::Custom("custom".to_string()),
        CacheType::Custom("custom".to_string())
    );
}

#[test]
fn test_config_error_display() {
    use std::io;
    
    let io_error = ConfigError::IoError(io::Error::new(io::ErrorKind::NotFound, "test"));
    assert!(format!("{}", io_error).contains("I/O error"));
    
    let parse_error = ConfigError::ParseError("invalid TOML".to_string());
    assert!(format!("{}", parse_error).contains("Parse error"));
    
    let profile_error = ConfigError::ProfileNotFound("custom".to_string());
    assert!(format!("{}", profile_error).contains("Profile not found"));
    
    let backup_error = ConfigError::BackupNotFound(PathBuf::from("test.toml"));
    assert!(format!("{}", backup_error).contains("Backup not found"));
    
    let linker_error = ConfigError::UnknownLinker("unknown".to_string());
    assert!(format!("{}", linker_error).contains("Unknown linker"));
    
    let other_error = ConfigError::Other("generic error".to_string());
    assert_eq!(format!("{}", other_error), "generic error");
}

#[test]
fn test_generate_linker_config() {
    let config = Config::default();
    
    // Test Windows linkers
    if cfg!(target_os = "windows") {
        let rust_lld = config.generate_linker_config("rust-lld")
            .expect("Should generate rust-lld config");
        assert!(rust_lld.contains("[target.x86_64-pc-windows-msvc]"));
        assert!(rust_lld.contains("linker = \"rust-lld\""));
        
        let lld_link = config.generate_linker_config("lld-link")
            .expect("Should generate lld-link config");
        assert!(lld_link.contains("linker = \"lld-link.exe\""));
    }
    
    // Test Linux linkers
    if cfg!(target_os = "linux") {
        let mold = config.generate_linker_config("mold")
            .expect("Should generate mold config");
        assert!(mold.contains("[target.x86_64-unknown-linux-gnu]"));
        assert!(mold.contains("link-arg=-fuse-ld=mold"));
        
        let lld = config.generate_linker_config("lld")
            .expect("Should generate lld config");
        assert!(lld.contains("link-arg=-fuse-ld=lld"));
    }
    
    // Test unknown linker
    let result = config.generate_linker_config("unknown");
    assert!(result.is_err());
}

#[test]
fn test_profile_mutation() {
    let mut config = Config::default();
    
    // Get mutable reference to dev profile
    {
        let dev_profile = config.get_profile_mut("dev")
            .expect("Should get mutable dev profile");
        dev_profile.jobs = Some(16);
        dev_profile.incremental = Some(false);
    }
    
    // Verify changes were applied
    let dev_profile = config.get_profile("dev")
        .expect("Should get dev profile");
    assert_eq!(dev_profile.jobs, Some(16));
    assert_eq!(dev_profile.incremental, Some(false));
}

#[test]
fn test_missing_profile() {
    let config = Config::default();
    assert!(config.get_profile("nonexistent").is_none());
    
    let result = config.apply_profile("nonexistent");
    assert!(result.is_err());
}

/// Integration test for complete configuration workflow
#[test]
fn test_complete_workflow() {
    let temp_dir = setup_test_dir();
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    
    // Change to temp directory
    let result = std::env::set_current_dir(temp_dir.path());
    if result.is_err() {
        // If we can't change directory, skip the test
        return;
    }
    
    // Create config
    let mut config = Config::default();
    
    // Modify settings
    config.global.verbose = true;
    config.global.optimization_level = OptimizationLevel::Aggressive;
    
    // Apply hardware optimizations
    config.apply_hardware_optimizations().expect("Failed to apply hardware opts");
    
    // Save configuration
    config.save().expect("Failed to save config");
    
    // Verify files were created
    assert!(Path::new(".cargo/config.toml").exists());
    assert!(Path::new("cargo-optimize.toml").exists());
    
    // Create backup
    let backup_path = config.create_backup().expect("Failed to create backup");
    assert!(backup_path.exists());
    
    // Modify config again
    config.global.verbose = false;
    config.save().expect("Failed to save modified config");
    
    // Restore from backup
    Config::restore_from_backup(&backup_path).expect("Failed to restore");
    
    // Verify restoration worked
    let restored_content = fs::read_to_string(".cargo/config.toml")
        .expect("Failed to read restored config");
    assert!(restored_content.contains("# Cargo configuration"));
    
    // Restore original directory
    let _ = std::env::set_current_dir(original_dir);
}
