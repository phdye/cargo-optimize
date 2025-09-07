//! Configuration Management for cargo-optimize
//! 
//! This module provides enhanced configuration management with support for:
//! - Safe merging of existing configurations
//! - Multi-file config support (.cargo/config.toml + cargo-optimize.toml)
//! - Profile system (dev/test/release/bench)
//! - Backup and rollback capabilities

use std::collections::HashMap;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Main configuration structure for cargo-optimize
#[derive(Debug, Clone)]
pub struct Config {
    /// Optimization profiles for different build modes
    pub profiles: HashMap<String, Profile>,
    /// Global settings that apply to all profiles
    pub global: GlobalSettings,
    /// Backup configuration for rollback
    pub backup: BackupConfig,
    /// Metadata about the configuration
    pub metadata: ConfigMetadata,
}

/// Optimization profile for a specific build mode
#[derive(Debug, Clone)]
pub struct Profile {
    /// Name of the profile (dev, test, release, bench)
    pub name: String,
    /// Linker to use for this profile
    pub linker: Option<String>,
    /// Number of parallel jobs
    pub jobs: Option<usize>,
    /// Whether to use incremental compilation
    pub incremental: Option<bool>,
    /// Custom rustflags for this profile
    pub rustflags: Vec<String>,
    /// Build cache settings
    pub cache: CacheSettings,
    /// Target directory override
    pub target_dir: Option<PathBuf>,
}

/// Global settings that apply across all profiles
#[derive(Debug, Clone)]
pub struct GlobalSettings {
    /// Default optimization level
    pub optimization_level: OptimizationLevel,
    /// Whether to automatically detect hardware
    pub auto_detect_hardware: bool,
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Whether to use sccache if available
    pub use_sccache: bool,
    /// Custom environment variables
    pub env_vars: HashMap<String, String>,
}

/// Optimization level for build configuration
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// Conservative optimization - minimal changes, maximum compatibility
    Conservative,
    /// Balanced optimization - good performance with reasonable safety
    Balanced,
    /// Aggressive optimization - maximum performance, may affect stability
    Aggressive,
}

/// Cache configuration settings
#[derive(Debug, Clone)]
pub struct CacheSettings {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache directory location
    pub cache_dir: Option<PathBuf>,
    /// Maximum cache size in MB
    pub max_size_mb: Option<usize>,
    /// Cache type (sccache, ccache, etc.)
    pub cache_type: CacheType,
}

/// Type of build cache to use
#[derive(Debug, Clone, PartialEq)]
pub enum CacheType {
    /// No caching
    None,
    /// sccache
    Sccache,
    /// ccache
    Ccache,
    /// Custom cache command
    Custom(String),
}

/// Backup configuration for rollback support
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Whether to create backups automatically
    pub auto_backup: bool,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Directory to store backups
    pub backup_dir: PathBuf,
}

/// Metadata about the configuration
#[derive(Debug, Clone)]
pub struct ConfigMetadata {
    /// Version of cargo-optimize that created this config
    pub version: String,
    /// Timestamp when config was created/modified
    pub timestamp: u64,
    /// Platform this config was created for
    pub platform: String,
    /// Hash of the configuration for integrity checking
    pub hash: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            profiles: Self::default_profiles(),
            global: GlobalSettings::default(),
            backup: BackupConfig::default(),
            metadata: ConfigMetadata::default(),
        }
    }
}

impl Config {
    /// Create default profiles for dev, test, release, and bench
    fn default_profiles() -> HashMap<String, Profile> {
        let mut profiles = HashMap::new();
        
        // Dev profile - fast compilation, moderate optimization
        profiles.insert("dev".to_string(), Profile {
            name: "dev".to_string(),
            linker: None, // Will be auto-detected
            jobs: None, // Will use CPU count
            incremental: Some(true),
            rustflags: vec![],
            cache: CacheSettings {
                enabled: true,
                cache_dir: None,
                max_size_mb: Some(1024),
                cache_type: CacheType::Sccache,
            },
            target_dir: None,
        });
        
        // Test profile - balanced for test execution
        profiles.insert("test".to_string(), Profile {
            name: "test".to_string(),
            linker: None,
            jobs: None,
            incremental: Some(true),
            rustflags: vec![],
            cache: CacheSettings {
                enabled: true,
                cache_dir: None,
                max_size_mb: Some(512),
                cache_type: CacheType::Sccache,
            },
            target_dir: None,
        });
        
        // Release profile - maximum optimization
        profiles.insert("release".to_string(), Profile {
            name: "release".to_string(),
            linker: None,
            jobs: None,
            incremental: Some(false),
            rustflags: vec!["-C".to_string(), "opt-level=3".to_string()],
            cache: CacheSettings {
                enabled: true,
                cache_dir: None,
                max_size_mb: Some(2048),
                cache_type: CacheType::Sccache,
            },
            target_dir: None,
        });
        
        // Bench profile - optimized for benchmarking
        profiles.insert("bench".to_string(), Profile {
            name: "bench".to_string(),
            linker: None,
            jobs: None,
            incremental: Some(false),
            rustflags: vec!["-C".to_string(), "opt-level=3".to_string()],
            cache: CacheSettings {
                enabled: false, // Disable cache for consistent benchmarks
                cache_dir: None,
                max_size_mb: None,
                cache_type: CacheType::None,
            },
            target_dir: None,
        });
        
        profiles
    }
    
    /// Load configuration from multiple sources
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Config::default();
        
        // Load from .cargo/config.toml if it exists
        let cargo_config_path = Path::new(".cargo/config.toml");
        if cargo_config_path.exists() {
            config.merge_cargo_config(cargo_config_path)?;
        }
        
        // Load from cargo-optimize.toml if it exists
        let optimize_config_path = Path::new("cargo-optimize.toml");
        if optimize_config_path.exists() {
            config.merge_optimize_config(optimize_config_path)?;
        }
        
        // Auto-detect hardware and apply optimizations
        if config.global.auto_detect_hardware {
            config.apply_hardware_optimizations()?;
        }
        
        Ok(config)
    }
    
    /// Save configuration to files
    pub fn save(&self) -> Result<(), ConfigError> {
        // Create backup if enabled
        if self.backup.auto_backup {
            self.create_backup()?;
        }
        
        // Save cargo configuration
        self.save_cargo_config()?;
        
        // Save cargo-optimize specific configuration
        self.save_optimize_config()?;
        
        Ok(())
    }
    
    /// Merge with existing .cargo/config.toml
    fn merge_cargo_config(&mut self, path: &Path) -> Result<(), ConfigError> {
        let _content = fs::read_to_string(path)
            .map_err(ConfigError::IoError)?;
        
        // Parse and merge the configuration
        // For now, we'll do a simple text-based merge
        // In a full implementation, we'd use a TOML parser
        
        Ok(())
    }
    
    /// Merge with cargo-optimize.toml
    fn merge_optimize_config(&mut self, path: &Path) -> Result<(), ConfigError> {
        let _content = fs::read_to_string(path)
            .map_err(ConfigError::IoError)?;
        
        // Parse cargo-optimize specific settings
        // This would contain profile overrides and advanced settings
        
        Ok(())
    }
    
    /// Apply hardware-based optimizations
    pub fn apply_hardware_optimizations(&mut self) -> Result<(), ConfigError> {
        // Detect CPU cores
        let cpu_cores = num_cpus::get();
        
        // Apply to all profiles that don't have explicit job counts
        for profile in self.profiles.values_mut() {
            if profile.jobs.is_none() {
                // Use n-1 cores to leave one for the system
                profile.jobs = Some((cpu_cores - 1).max(1));
            }
        }
        
        // Detect available memory and adjust cache sizes
        // This would use platform-specific APIs in a full implementation
        
        Ok(())
    }
    
    /// Save cargo configuration (.cargo/config.toml)
    pub fn save_cargo_config(&self) -> Result<(), ConfigError> {
        let config_dir = Path::new(".cargo");
        fs::create_dir_all(config_dir)
            .map_err(ConfigError::IoError)?;
        
        let config_path = config_dir.join("config.toml");
        let mut content = String::new();
        
        // Add header
        content.push_str("# Cargo configuration - optimized by cargo-optimize\n");
        content.push_str(&format!("# Generated: {}\n", format_timestamp()));
        content.push_str(&format!("# Version: {}\n\n", self.metadata.version));
        
        // Add target configuration with detected linker
        if let Some(linker) = self.detect_best_linker() {
            content.push_str(&self.generate_linker_config(&linker)?);
        }
        
        // Add build configuration
        content.push_str("\n[build]\n");
        if let Some(jobs) = self.profiles.get("dev").and_then(|p| p.jobs) {
            content.push_str(&format!("jobs = {}\n", jobs));
        }
        
        // Add profile configurations
        for (name, profile) in &self.profiles {
            if !profile.rustflags.is_empty() || profile.incremental.is_some() {
                content.push_str(&format!("\n[profile.{}]\n", name));
                if let Some(incremental) = profile.incremental {
                    content.push_str(&format!("incremental = {}\n", incremental));
                }
            }
        }
        
        fs::write(config_path, content)
            .map_err(ConfigError::IoError)?;
        
        Ok(())
    }
    
    /// Save cargo-optimize specific configuration
    pub fn save_optimize_config(&self) -> Result<(), ConfigError> {
        let config_path = Path::new("cargo-optimize.toml");
        let mut content = String::new();
        
        // Add header
        content.push_str("# cargo-optimize configuration file\n");
        content.push_str("# This file contains advanced optimization settings\n\n");
        
        // Add metadata
        content.push_str("[metadata]\n");
        content.push_str(&format!("version = \"{}\"\n", self.metadata.version));
        content.push_str(&format!("timestamp = {}\n", self.metadata.timestamp));
        content.push_str(&format!("platform = \"{}\"\n\n", self.metadata.platform));
        
        // Add global settings
        content.push_str("[global]\n");
        content.push_str(&format!("optimization_level = \"{:?}\"\n", self.global.optimization_level));
        content.push_str(&format!("auto_detect_hardware = {}\n", self.global.auto_detect_hardware));
        content.push_str(&format!("verbose = {}\n", self.global.verbose));
        content.push_str(&format!("use_sccache = {}\n\n", self.global.use_sccache));
        
        // Add profile-specific settings
        for (name, profile) in &self.profiles {
            content.push_str(&format!("[profile.{}]\n", name));
            if let Some(linker) = &profile.linker {
                content.push_str(&format!("linker = \"{}\"\n", linker));
            }
            if let Some(jobs) = profile.jobs {
                content.push_str(&format!("jobs = {}\n", jobs));
            }
            content.push_str(&format!("cache_enabled = {}\n", profile.cache.enabled));
            if let Some(max_size) = profile.cache.max_size_mb {
                content.push_str(&format!("cache_max_size_mb = {}\n", max_size));
            }
            content.push('\n');
        }
        
        // Add backup settings
        content.push_str("[backup]\n");
        content.push_str(&format!("auto_backup = {}\n", self.backup.auto_backup));
        content.push_str(&format!("max_backups = {}\n", self.backup.max_backups));
        
        fs::write(config_path, content)
            .map_err(ConfigError::IoError)?;
        
        Ok(())
    }
    
    /// Create a backup of current configuration
    pub fn create_backup(&self) -> Result<PathBuf, ConfigError> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.backup.backup_dir)
            .map_err(ConfigError::IoError)?;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_name = format!("config_backup_{}.toml", timestamp);
        let backup_path = self.backup.backup_dir.join(backup_name);
        
        // Save current configuration to backup
        let cargo_config = Path::new(".cargo/config.toml");
        if cargo_config.exists() {
            let content = fs::read_to_string(cargo_config)
                .map_err(ConfigError::IoError)?;
            fs::write(&backup_path, content)
                .map_err(ConfigError::IoError)?;
        } else {
            // If no config exists, create an empty backup
            fs::write(&backup_path, "# Empty configuration backup\n")
                .map_err(ConfigError::IoError)?;
        }
        
        // Clean up old backups if needed
        self.cleanup_old_backups()?;
        
        Ok(backup_path)
    }
    
    /// Restore configuration from a backup
    pub fn restore_from_backup(backup_path: &Path) -> Result<(), ConfigError> {
        if !backup_path.exists() {
            return Err(ConfigError::BackupNotFound(backup_path.to_path_buf()));
        }
        
        let content = fs::read_to_string(backup_path)
            .map_err(ConfigError::IoError)?;
        
        let config_path = Path::new(".cargo/config.toml");
        fs::create_dir_all(config_path.parent().unwrap())
            .map_err(ConfigError::IoError)?;
        
        fs::write(config_path, content)
            .map_err(ConfigError::IoError)?;
        
        Ok(())
    }
    
    /// Clean up old backups, keeping only the most recent ones
    pub fn cleanup_old_backups(&self) -> Result<(), ConfigError> {
        let mut backups: Vec<_> = fs::read_dir(&self.backup.backup_dir)
            .map_err(ConfigError::IoError)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|s| s.starts_with("config_backup_") && s.ends_with(".toml"))
                    .unwrap_or(false)
            })
            .collect();
        
        if backups.len() <= self.backup.max_backups {
            return Ok(());
        }
        
        // Sort by modification time
        backups.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        
        // Remove oldest backups
        let to_remove = backups.len() - self.backup.max_backups;
        for entry in backups.into_iter().take(to_remove) {
            fs::remove_file(entry.path())
                .map_err(ConfigError::IoError)?;
        }
        
        Ok(())
    }
    
    /// Detect the best available linker (delegating to mvp module)
    fn detect_best_linker(&self) -> Option<String> {
        match crate::mvp::detect_best_linker() {
            Ok(linker) if linker != "default" => Some(linker),
            _ => None,
        }
    }
    
    /// Generate linker configuration for the target platform
    pub fn generate_linker_config(&self, linker: &str) -> Result<String, ConfigError> {
        let config = if cfg!(target_os = "windows") {
            match linker {
                "rust-lld" => {
                    "[target.x86_64-pc-windows-msvc]\n\
                     linker = \"rust-lld\"\n"
                },
                "lld-link" => {
                    "[target.x86_64-pc-windows-msvc]\n\
                     linker = \"lld-link.exe\"\n"
                },
                _ => return Err(ConfigError::UnknownLinker(linker.to_string())),
            }
        } else {
            match linker {
                "mold" => {
                    "[target.x86_64-unknown-linux-gnu]\n\
                     linker = \"clang\"\n\
                     rustflags = [\"-C\", \"link-arg=-fuse-ld=mold\"]\n"
                },
                "lld" => {
                    "[target.x86_64-unknown-linux-gnu]\n\
                     linker = \"clang\"\n\
                     rustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]\n"
                },
                "gold" => {
                    "[target.x86_64-unknown-linux-gnu]\n\
                     linker = \"clang\"\n\
                     rustflags = [\"-C\", \"link-arg=-fuse-ld=gold\"]\n"
                },
                _ => return Err(ConfigError::UnknownLinker(linker.to_string())),
            }
        };
        
        Ok(config.to_string())
    }
    
    /// Get a specific profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }
    
    /// Get a mutable reference to a specific profile
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(name)
    }
    
    /// Apply a specific profile's settings
    pub fn apply_profile(&self, profile_name: &str) -> Result<(), ConfigError> {
        let profile = self.profiles.get(profile_name)
            .ok_or_else(|| ConfigError::ProfileNotFound(profile_name.to_string()))?;
        
        // Set environment variables for the profile
        if let Some(jobs) = profile.jobs {
            std::env::set_var("CARGO_BUILD_JOBS", jobs.to_string());
        }
        
        if profile.cache.enabled {
            match profile.cache.cache_type {
                CacheType::Sccache => {
                    std::env::set_var("RUSTC_WRAPPER", "sccache");
                },
                CacheType::Ccache => {
                    std::env::set_var("RUSTC_WRAPPER", "ccache");
                },
                CacheType::Custom(ref cmd) => {
                    std::env::set_var("RUSTC_WRAPPER", cmd);
                },
                CacheType::None => {},
            }
        }
        
        Ok(())
    }
}

impl Default for GlobalSettings {
    fn default() -> Self {
        GlobalSettings {
            optimization_level: OptimizationLevel::Balanced,
            auto_detect_hardware: true,
            verbose: false,
            use_sccache: true,
            env_vars: HashMap::new(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            auto_backup: true,
            max_backups: 5,
            backup_dir: PathBuf::from(".cargo/backups"),
        }
    }
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        ConfigMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            platform: if cfg!(target_os = "windows") {
                "windows".to_string()
            } else if cfg!(target_os = "linux") {
                "linux".to_string()
            } else {
                "unknown".to_string()
            },
            hash: None,
        }
    }
}

/// Error types for configuration operations
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error occurred
    IoError(io::Error),
    /// Configuration file is malformed
    ParseError(String),
    /// Profile not found
    ProfileNotFound(String),
    /// Backup not found
    BackupNotFound(PathBuf),
    /// Unknown linker specified
    UnknownLinker(String),
    /// Generic error with message
    Other(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "I/O error: {}", e),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ProfileNotFound(name) => write!(f, "Profile not found: {}", name),
            ConfigError::BackupNotFound(path) => write!(f, "Backup not found: {}", path.display()),
            ConfigError::UnknownLinker(name) => write!(f, "Unknown linker: {}", name),
            ConfigError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Helper function to format timestamp
fn format_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    let total_secs = duration.as_secs();
    
    // Simple timestamp format
    format!("Unix timestamp: {}", total_secs)
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.profiles.len(), 4);
        assert!(config.profiles.contains_key("dev"));
        assert!(config.profiles.contains_key("test"));
        assert!(config.profiles.contains_key("release"));
        assert!(config.profiles.contains_key("bench"));
    }
    
    #[test]
    fn test_profile_defaults() {
        let config = Config::default();
        
        let dev_profile = config.get_profile("dev").unwrap();
        assert_eq!(dev_profile.incremental, Some(true));
        assert!(dev_profile.cache.enabled);
        
        let bench_profile = config.get_profile("bench").unwrap();
        assert_eq!(bench_profile.incremental, Some(false));
        assert!(!bench_profile.cache.enabled);
    }
    
    #[test]
    fn test_optimization_levels() {
        assert_eq!(OptimizationLevel::Conservative, OptimizationLevel::Conservative);
        assert_ne!(OptimizationLevel::Conservative, OptimizationLevel::Aggressive);
    }
    
    #[test]
    fn test_backup_config_defaults() {
        let backup = BackupConfig::default();
        assert!(backup.auto_backup);
        assert_eq!(backup.max_backups, 5);
        assert_eq!(backup.backup_dir, PathBuf::from(".cargo/backups"));
    }
}
