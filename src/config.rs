//! Configuration Management for cargo-optimize - Phase 1.1 Implementation
//! 
//! This module provides sophisticated configuration management using:
//! - Figment for layered configuration (defaults -> file -> env vars)
//! - toml_edit for preserving user's TOML formatting
//! - Automatic backup and restore capabilities
//! - Profile support (dev/test/release/bench)
//! - Percentage value parsing for flexible configuration

use anyhow::{Context, Result};
use figment::providers::{Env, Format, Toml};
use figment::{Figment, Profile as FigmentProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{DocumentMut, Item, Table};
use tracing::{debug, info};

/// Main configuration structure for cargo-optimize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Optimization profiles for different build modes
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    
    /// Global settings that apply to all profiles
    #[serde(default)]
    pub global: GlobalSettings,
    
    /// Backup configuration for rollback
    #[serde(default)]
    pub backup: BackupConfig,
    
    /// Metadata about the configuration
    #[serde(default)]
    pub metadata: ConfigMetadata,
}

/// Optimization profile for a specific build mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Name of the profile (dev, test, release, bench)
    pub name: String,
    
    /// Linker to use for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linker: Option<String>,
    
    /// Number of parallel jobs (supports percentages like "75%")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<JobCount>,
    
    /// Whether to use incremental compilation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<bool>,
    
    /// Custom rustflags for this profile
    #[serde(default)]
    pub rustflags: Vec<String>,
    
    /// Build cache settings
    #[serde(default)]
    pub cache: CacheSettings,
    
    /// Target directory override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_dir: Option<PathBuf>,
}

/// Job count configuration with percentage support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JobCount {
    /// Fixed number of jobs
    Fixed(usize),
    /// Percentage of available cores (e.g., "75%")
    Percentage(String),
}

impl JobCount {
    /// Convert to actual job count based on available cores
    pub fn to_count(&self) -> usize {
        match self {
            JobCount::Fixed(n) => *n,
            JobCount::Percentage(p) => {
                let cores = num_cpus::get();
                // Handle both "75%" and "75" formats for compatibility
                let percentage_str = p.strip_suffix('%').unwrap_or(p);
                if let Ok(percentage) = percentage_str.parse::<f64>() {
                    let count = (cores as f64 * (percentage / 100.0)).round() as usize;
                    return count.max(1);
                }
                // Default to all cores if parsing fails
                cores
            }
        }
    }
    
    /// Parse from a string value
    pub fn parse(s: &str) -> Result<Self> {
        if s.ends_with('%') {
            Ok(JobCount::Percentage(s.to_string()))
        } else if let Ok(n) = s.parse::<usize>() {
            Ok(JobCount::Fixed(n))
        } else {
            anyhow::bail!("Invalid job count: {}", s)
        }
    }
}

/// Global settings that apply across all profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    
    /// Default job count for all profiles (supports percentages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_jobs: Option<JobCount>,
}

/// Optimization level for build configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    /// Conservative optimization - minimal changes, maximum compatibility
    Conservative,
    /// Balanced optimization - good performance with reasonable safety
    Balanced,
    /// Aggressive optimization - maximum performance, may affect stability
    Aggressive,
}

/// Cache configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Whether caching is enabled
    pub enabled: bool,
    
    /// Cache directory location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<PathBuf>,
    
    /// Maximum cache size (supports percentages like "10%")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<CacheSize>,
    
    /// Cache type (sccache, ccache, etc.)
    pub cache_type: CacheType,
}

/// Cache size configuration with percentage support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CacheSize {
    /// Fixed size in MB
    Megabytes(usize),
    /// Percentage of available disk space
    Percentage(String),
}

impl CacheSize {
    /// Convert to actual size in MB based on available disk space
    pub fn to_megabytes(&self) -> usize {
        match self {
            CacheSize::Megabytes(mb) => *mb,
            CacheSize::Percentage(p) => {
                // Use sysinfo to get disk space
                use sysinfo::System;
                let mut sys = System::new_all();
                sys.refresh_all();
                
                // For now, default to 10GB as sysinfo API has changed
                // TODO: Update to use new sysinfo disk API
                let total_space_mb = 10240;
                
                if let Some(percentage_str) = p.strip_suffix('%') {
                    if let Ok(percentage) = percentage_str.parse::<f64>() {
                        let size = (total_space_mb as f64 * (percentage / 100.0)).round() as usize;
                        return size.max(100); // Minimum 100MB
                    }
                }
                // Default to 1GB if parsing fails
                1024
            }
        }
    }
}

/// Type of build cache to use
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Whether to create backups automatically
    pub auto_backup: bool,
    
    /// Maximum number of backups to keep
    pub max_backups: usize,
    
    /// Directory to store backups
    pub backup_dir: PathBuf,
}

/// Metadata about the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Version of cargo-optimize that created this config
    pub version: String,
    
    /// Timestamp when config was created/modified
    pub timestamp: u64,
    
    /// Platform this config was created for
    pub platform: String,
    
    /// Hash of the configuration for integrity checking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Configuration manager using Figment for layered config
pub struct ConfigManager {
    figment: Figment,
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager with layered sources
    pub fn new() -> Result<Self> {
        // Build layered configuration with Figment
        let figment = Figment::new()
            // 1. Start with defaults
            .merge(Toml::string(&Self::default_config_toml()))
            // 2. Merge with cargo-optimize.toml if it exists
            .merge(Toml::file("cargo-optimize.toml").nested())
            // 3. Override with environment variables (CARGO_OPTIMIZE_*)
            .merge(Env::prefixed("CARGO_OPTIMIZE_").split("__"));
        
        // Extract the configuration
        let mut config: Config = figment.extract()
            .unwrap_or_else(|_| Config::default());
        
        // Auto-detect hardware if enabled
        if config.global.auto_detect_hardware {
            config.apply_hardware_optimizations()?;
        }
        
        Ok(ConfigManager {
            figment,
            config,
            config_path: PathBuf::from(".cargo/config.toml"),
        })
    }
    
    /// Load configuration from a specific profile
    pub fn with_profile(profile: &str) -> Result<Self> {
        let figment = Figment::new()
            .merge(Toml::string(&Self::default_config_toml()))
            .merge(Toml::file("cargo-optimize.toml").nested())
            .select(FigmentProfile::from(profile))
            .merge(Env::prefixed("CARGO_OPTIMIZE_").split("__"));
        
        let mut config: Config = figment.extract()
            .unwrap_or_else(|_| Config::default());
        
        if config.global.auto_detect_hardware {
            config.apply_hardware_optimizations()?;
        }
        
        Ok(ConfigManager {
            figment,
            config,
            config_path: PathBuf::from(".cargo/config.toml"),
        })
    }
    
    /// Get the default configuration as TOML string
    fn default_config_toml() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }
    
    /// Apply configuration to .cargo/config.toml while preserving formatting
    pub fn apply(&self) -> Result<()> {
        // Create backup if enabled
        if self.config.backup.auto_backup {
            self.create_backup()?;
        }
        
        // Load or create the document
        let mut doc = if self.config_path.exists() {
            // Read with retry for Windows file locking issues
            let content = self.read_config_with_retry()?;
            content.parse::<DocumentMut>()
                .unwrap_or_else(|_| DocumentMut::new())
        } else {
            // Ensure .cargo directory exists
            if let Some(parent) = self.config_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create .cargo directory")?;
            }
            DocumentMut::new()
        };
        
        // Apply our optimizations while preserving existing content
        self.apply_to_document(&mut doc)?;
        
        // Write back the modified document with retry for Windows
        self.write_config_with_retry(&doc.to_string())?;
        
        info!("Configuration applied successfully to {:?}", self.config_path);
        Ok(())
    }
    
    /// Apply optimizations to a TOML document while preserving formatting
    fn apply_to_document(&self, doc: &mut DocumentMut) -> Result<()> {
        // Add header comment if document is empty
        if doc.as_table().is_empty() {
            doc.decor_mut().set_prefix(
                "# Cargo configuration - optimized by cargo-optimize\n\
                 # This file has been automatically optimized for better build performance\n\n"
            );
        }
        
        // Apply linker configuration
        if let Some(linker) = self.detect_best_linker() {
            self.apply_linker_to_document(doc, &linker)?;
        }
        
        // Apply build configuration
        self.apply_build_config_to_document(doc)?;
        
        // Apply profile configurations
        self.apply_profiles_to_document(doc)?;
        
        Ok(())
    }
    
    /// Apply linker configuration to document
    fn apply_linker_to_document(&self, doc: &mut DocumentMut, linker: &str) -> Result<()> {
        let target = if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        
        // Ensure target table exists
        let _target_key = format!("target.{}", target);
        if !doc.contains_key("target") {
            doc["target"] = Item::Table(Table::new());
        }
        
        let target_table = doc["target"].as_table_mut()
            .context("Failed to access target table")?;
        
        if !target_table.contains_key(target) {
            target_table[target] = Item::Table(Table::new());
        }
        
        let platform_table = target_table[target].as_table_mut()
            .context("Failed to access platform table")?;
        
        // Set linker
        if cfg!(target_os = "windows") {
            platform_table["linker"] = toml_edit::value(linker);
        } else {
            // For Linux, use clang with appropriate flags
            platform_table["linker"] = toml_edit::value("clang");
            let rustflags = match linker {
                "mold" => vec!["-C", "link-arg=-fuse-ld=mold"],
                "lld" => vec!["-C", "link-arg=-fuse-ld=lld"],
                "gold" => vec!["-C", "link-arg=-fuse-ld=gold"],
                _ => vec![],
            };
            
            if !rustflags.is_empty() {
                let mut array = toml_edit::Array::new();
                for flag in rustflags {
                    array.push(flag);
                }
                platform_table["rustflags"] = toml_edit::value(array);
            }
        }
        
        debug!("Applied linker configuration: {}", linker);
        Ok(())
    }
    
    /// Apply build configuration to document
    fn apply_build_config_to_document(&self, doc: &mut DocumentMut) -> Result<()> {
        // Ensure build table exists
        if !doc.contains_key("build") {
            doc["build"] = Item::Table(Table::new());
        }
        
        let build_table = doc["build"].as_table_mut()
            .context("Failed to access build table")?;
        
        // Apply default job count if specified
        if let Some(jobs) = &self.config.global.default_jobs {
            build_table["jobs"] = toml_edit::value(jobs.to_count() as i64);
        }
        
        Ok(())
    }
    
    /// Apply profile configurations to document
    fn apply_profiles_to_document(&self, doc: &mut DocumentMut) -> Result<()> {
        for (name, profile) in &self.config.profiles {
            // Skip if no customizations for this profile
            if profile.rustflags.is_empty() && profile.incremental.is_none() {
                continue;
            }
            
            // Ensure profile table exists
            let _profile_key = format!("profile.{}", name);
            if !doc.contains_key("profile") {
                doc["profile"] = Item::Table(Table::new());
            }
            
            let profile_table = doc["profile"].as_table_mut()
                .context("Failed to access profile table")?;
            
            if !profile_table.contains_key(name) {
                profile_table[name] = Item::Table(Table::new());
            }
            
            let specific_profile = profile_table[name].as_table_mut()
                .context("Failed to access specific profile")?;
            
            // Apply incremental setting
            if let Some(incremental) = profile.incremental {
                specific_profile["incremental"] = toml_edit::value(incremental);
            }
        }
        
        Ok(())
    }
    
    /// Create a backup of the current configuration
    pub fn create_backup(&self) -> Result<PathBuf> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.config.backup.backup_dir)
            .context("Failed to create backup directory")?;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_name = format!("config_backup_{}.toml", timestamp);
        let backup_path = self.config.backup.backup_dir.join(backup_name);
        
        // Copy current config if it exists
        if self.config_path.exists() {
            fs::copy(&self.config_path, &backup_path)
                .context("Failed to create backup")?;
            info!("Created backup at {:?}", backup_path);
        } else {
            // Create empty backup as marker
            fs::write(&backup_path, "# No previous configuration\n")
                .context("Failed to create empty backup")?;
        }
        
        // Clean up old backups
        self.cleanup_old_backups()?;
        
        Ok(backup_path)
    }
    
    /// Restore configuration from a backup
    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        if !backup_path.exists() {
            anyhow::bail!("Backup file does not exist: {:?}", backup_path);
        }
        
        fs::copy(backup_path, &self.config_path)
            .context("Failed to restore from backup")?;
        
        info!("Restored configuration from {:?}", backup_path);
        Ok(())
    }
    
    /// Clean up old backups, keeping only the most recent ones
    fn cleanup_old_backups(&self) -> Result<()> {
        let mut backups: Vec<_> = fs::read_dir(&self.config.backup.backup_dir)
            .context("Failed to read backup directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name()
                    .to_str()
                    .map(|s| s.starts_with("config_backup_") && s.ends_with(".toml"))
                    .unwrap_or(false)
            })
            .collect();
        
        if backups.len() <= self.config.backup.max_backups {
            return Ok(());
        }
        
        // Sort by modification time
        backups.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        
        // Remove oldest backups
        let to_remove = backups.len() - self.config.backup.max_backups;
        for entry in backups.into_iter().take(to_remove) {
            fs::remove_file(entry.path())
                .context("Failed to remove old backup")?;
            debug!("Removed old backup: {:?}", entry.path());
        }
        
        Ok(())
    }
    
    /// Detect the best available linker
    fn detect_best_linker(&self) -> Option<String> {
        match crate::mvp::detect_best_linker() {
            Ok(linker) if linker != "default" => Some(linker),
            _ => None,
        }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
    
    /// Read config file with retry for Windows file locking
    fn read_config_with_retry(&self) -> Result<String> {
        let mut retries = 3;
        let mut last_error = None;
        
        while retries > 0 {
            match fs::read_to_string(&self.config_path) {
                Ok(content) => return Ok(content),
                Err(e) => {
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Failed to read config after retries: {:?}", last_error))
    }
    
    /// Write config file with retry for Windows file locking
    fn write_config_with_retry(&self, content: &str) -> Result<()> {
        let mut retries = 3;
        let mut last_error = None;
        
        while retries > 0 {
            match fs::write(&self.config_path, content) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Failed to write config after retries: {:?}", last_error))
    }
}

impl Config {
    /// Apply hardware-based optimizations
    pub fn apply_hardware_optimizations(&mut self) -> Result<()> {
        use sysinfo::System;
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let cpu_cores = num_cpus::get();
        let total_memory_mb = (sys.total_memory() / 1024) as usize;
        
        info!("Detected {} CPU cores, {}MB RAM", cpu_cores, total_memory_mb);
        
        // Apply to all profiles that don't have explicit job counts
        for profile in self.profiles.values_mut() {
            if profile.jobs.is_none() {
                // Use 75% of cores by default
                profile.jobs = Some(JobCount::Percentage("75%".to_string()));
            }
            
            // Adjust cache size based on available memory
            if profile.cache.max_size.is_none() {
                // Use 10% of RAM for cache, minimum 512MB
                let cache_size = (total_memory_mb / 10).max(512);
                profile.cache.max_size = Some(CacheSize::Megabytes(cache_size));
            }
        }
        
        Ok(())
    }
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
        
        // Dev profile - fast compilation
        profiles.insert("dev".to_string(), Profile {
            name: "dev".to_string(),
            linker: None,
            jobs: None,
            incremental: Some(true),
            rustflags: vec![],
            cache: CacheSettings {
                enabled: true,
                cache_dir: None,
                max_size: None,
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
                max_size: None,
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
            rustflags: vec![],
            cache: CacheSettings {
                enabled: true,
                cache_dir: None,
                max_size: None,
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
            rustflags: vec![],
            cache: CacheSettings {
                enabled: false,
                cache_dir: None,
                max_size: None,
                cache_type: CacheType::None,
            },
            target_dir: None,
        });
        
        profiles
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
            default_jobs: None,
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

impl Default for CacheSettings {
    fn default() -> Self {
        CacheSettings {
            enabled: true,
            cache_dir: None,
            max_size: None,
            cache_type: CacheType::Sccache,
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
            } else if cfg!(target_os = "macos") {
                "macos".to_string()
            } else {
                "unknown".to_string()
            },
            hash: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_job_count_parsing() {
        // Test percentage parsing
        let job = JobCount::parse("75%").unwrap();
        assert_eq!(job, JobCount::Percentage("75%".to_string()));
        
        // Test fixed number parsing
        let job = JobCount::parse("8").unwrap();
        assert_eq!(job, JobCount::Fixed(8));
        
        // Test conversion
        let cores = num_cpus::get();
        let job = JobCount::Percentage("50%".to_string());
        assert_eq!(job.to_count(), (cores / 2).max(1));
    }
    
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
    fn test_optimization_levels() {
        assert_eq!(
            OptimizationLevel::Conservative,
            OptimizationLevel::Conservative
        );
        assert_ne!(
            OptimizationLevel::Conservative,
            OptimizationLevel::Aggressive
        );
    }
    
    #[test]
    fn test_profile_defaults() {
        let config = Config::default();
        
        let dev = config.profiles.get("dev").unwrap();
        assert_eq!(dev.incremental, Some(true));
        assert!(dev.cache.enabled);
        
        let bench = config.profiles.get("bench").unwrap();
        assert_eq!(bench.incremental, Some(false));
        assert!(!bench.cache.enabled);
    }
}
