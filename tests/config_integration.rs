//! Simplified integration tests for configuration management
//! 
//! These tests verify core functionality without relying on complex directory operations

use cargo_optimize::config::*;

#[test]
fn test_config_basics() {
    // Test default configuration creation
    let config = Config::default();
    
    // Verify profiles exist
    assert_eq!(config.profiles.len(), 4);
    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("test"));
    assert!(config.profiles.contains_key("release"));
    assert!(config.profiles.contains_key("bench"));
    
    // Verify global settings
    assert_eq!(config.global.optimization_level, OptimizationLevel::Balanced);
    assert!(config.global.auto_detect_hardware);
    
    // Verify backup settings
    assert!(config.backup.auto_backup);
    assert_eq!(config.backup.max_backups, 5);
}

#[test]
fn test_profile_configuration() {
    let config = Config::default();
    
    // Test dev profile
    let dev = config.get_profile("dev").expect("Dev profile should exist");
    assert_eq!(dev.name, "dev");
    assert_eq!(dev.incremental, Some(true));
    assert!(dev.cache.enabled);
    
    // Test release profile
    let release = config.get_profile("release").expect("Release profile should exist");
    assert_eq!(release.incremental, Some(false));
    assert!(!release.rustflags.is_empty());
    
    // Test bench profile
    let bench = config.get_profile("bench").expect("Bench profile should exist");
    assert!(!bench.cache.enabled);
}

#[test]
fn test_profile_mutations() {
    let mut config = Config::default();
    
    // Modify dev profile
    if let Some(dev) = config.get_profile_mut("dev") {
        dev.jobs = Some(JobCount::Fixed(8));
        dev.incremental = Some(false);
    }
    
    // Verify changes
    let dev = config.get_profile("dev").expect("Dev profile should exist");
    assert_eq!(dev.jobs, Some(JobCount::Fixed(8)));
    assert_eq!(dev.incremental, Some(false));
}

#[test]
fn test_hardware_optimization_basic() {
    let mut config = Config::default();
    
    // Apply hardware optimizations
    config.apply_hardware_optimizations().expect("Hardware optimization should succeed");
    
    // Verify job counts were set
    for profile in config.profiles.values() {
        assert!(profile.jobs.is_some());
        let job_count = profile.jobs.as_ref().unwrap();
        assert!(job_count > &0usize);
    }
}

#[test]
fn test_linker_config_generation() {
    let config = Config::default();
    
    // Test generating linker configs for current platform
    if cfg!(target_os = "windows") {
        // Test Windows linkers
        let rust_lld = config.generate_linker_config("rust-lld");
        assert!(rust_lld.is_ok());
        let content = rust_lld.unwrap();
        assert!(content.contains("[target.x86_64-pc-windows-msvc]"));
        assert!(content.contains("rust-lld"));
        
        // Test unknown linker
        let unknown = config.generate_linker_config("unknown");
        assert!(unknown.is_err());
    }
    
    if cfg!(target_os = "linux") {
        // Test Linux linkers
        let mold = config.generate_linker_config("mold");
        assert!(mold.is_ok());
        let content = mold.unwrap();
        assert!(content.contains("[target.x86_64-unknown-linux-gnu]"));
        assert!(content.contains("mold"));
    }
}

#[test]
fn test_config_error_handling() {
    use std::path::PathBuf;
    
    // Test error display
    let io_error = ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(format!("{}", io_error).contains("I/O error"));
    
    let profile_error = ConfigError::ProfileNotFound("custom".to_string());
    assert!(format!("{}", profile_error).contains("Profile not found"));
    
    let backup_error = ConfigError::BackupNotFound(PathBuf::from("test.toml"));
    assert!(format!("{}", backup_error).contains("Backup not found"));
}

#[test]
fn test_cache_settings() {
    let config = Config::default();
    
    // Test cache types
    assert_eq!(CacheType::None, CacheType::None);
    assert_eq!(CacheType::Sccache, CacheType::Sccache);
    
    // Test dev profile cache
    let dev = config.get_profile("dev").unwrap();
    assert!(dev.cache.enabled);
    assert_eq!(dev.cache.cache_type, CacheType::Sccache);
    assert_eq!(dev.cache.max_size, Some(CacheSize::Megabytes(1024)));
    
    // Test bench profile cache (disabled)
    let bench = config.get_profile("bench").unwrap();
    assert!(!bench.cache.enabled);
    assert_eq!(bench.cache.cache_type, CacheType::None);
}

#[test]
fn test_optimization_levels() {
    // Test enum equality
    assert_eq!(OptimizationLevel::Conservative, OptimizationLevel::Conservative);
    assert_eq!(OptimizationLevel::Balanced, OptimizationLevel::Balanced);
    assert_eq!(OptimizationLevel::Aggressive, OptimizationLevel::Aggressive);
    
    // Test debug output
    assert_eq!(format!("{:?}", OptimizationLevel::Conservative), "Conservative");
    assert_eq!(format!("{:?}", OptimizationLevel::Balanced), "Balanced");
    assert_eq!(format!("{:?}", OptimizationLevel::Aggressive), "Aggressive");
}

#[test]
fn test_global_settings() {
    let config = Config::default();
    
    // Test defaults
    assert_eq!(config.global.optimization_level, OptimizationLevel::Balanced);
    assert!(config.global.auto_detect_hardware);
    assert!(!config.global.verbose);
    assert!(config.global.use_sccache);
    assert!(config.global.env_vars.is_empty());
}

#[test]
fn test_backup_configuration() {
    let config = Config::default();
    
    // Test defaults
    assert!(config.backup.auto_backup);
    assert_eq!(config.backup.max_backups, 5);
    assert_eq!(config.backup.backup_dir.to_string_lossy(), ".cargo/backups");
}

#[test]
fn test_metadata() {
    let config = Config::default();
    
    // Test metadata fields
    assert_eq!(config.metadata.version, env!("CARGO_PKG_VERSION"));
    assert!(config.metadata.timestamp > 0);
    
    if cfg!(target_os = "windows") {
        assert_eq!(config.metadata.platform, "windows");
    } else if cfg!(target_os = "linux") {
        assert_eq!(config.metadata.platform, "linux");
    }
}
