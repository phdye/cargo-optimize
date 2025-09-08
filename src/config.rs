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
use figment::{Figment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{DocumentMut, Item, Table};
use tracing::{debug, info};
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Profile not found
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    
    /// Backup not found
    #[error("Backup not found: {0}")]
    BackupNotFound(PathBuf),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Other error
    #[error("Configuration error: {0}")]
    Other(#[from] anyhow::Error),
}

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
#[derive(Debug, Clone, PartialEq)]
pub enum JobCount {
    /// Fixed number of jobs
    Fixed(usize),
    /// Percentage of available cores (e.g., "75%")
    Percentage(String),
}

// Custom Serialize implementation
impl Serialize for JobCount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JobCount::Fixed(n) => serializer.serialize_u64(*n as u64),
            JobCount::Percentage(p) => serializer.serialize_str(p),
        }
    }
}

// Custom Deserialize implementation
impl<'de> Deserialize<'de> for JobCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        
        struct JobCountVisitor;
        
        impl<'de> Visitor<'de> for JobCountVisitor {
            type Value = JobCount;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a number or a percentage string")
            }
            
            fn visit_u64<E>(self, value: u64) -> Result<JobCount, E>
            where
                E: de::Error,
            {
                Ok(JobCount::Fixed(value as usize))
            }
            
            fn visit_i64<E>(self, value: i64) -> Result<JobCount, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    Err(E::custom("job count must be positive"))
                } else {
                    Ok(JobCount::Fixed(value as usize))
                }
            }
            
            fn visit_str<E>(self, value: &str) -> Result<JobCount, E>
            where
                E: de::Error,
            {
                // Check if it's a percentage
                if value.ends_with('%') {
                    Ok(JobCount::Percentage(value.to_string()))
                } else if let Ok(n) = value.parse::<usize>() {
                    // Try to parse as a number
                    Ok(JobCount::Fixed(n))
                } else {
                    // If it's not a valid format, assume it's meant to be a percentage
                    Ok(JobCount::Percentage(value.to_string()))
                }
            }
        }
        
        deserializer.deserialize_any(JobCountVisitor)
    }
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
}

// Implement comparison traits for JobCount
impl PartialOrd for JobCount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.to_count().cmp(&other.to_count()))
    }
}

impl PartialEq<usize> for JobCount {
    fn eq(&self, other: &usize) -> bool {
        self.to_count() == *other
    }
}

impl PartialOrd<usize> for JobCount {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        Some(self.to_count().cmp(other))
    }
}

impl JobCount {
    /// Parse from a string value
    pub fn parse(s: &str) -> Result<Self> {
        // Check if it ends with % to determine if it's a percentage
        if s.ends_with('%') {
            Ok(JobCount::Percentage(s.to_string()))
        } else if let Ok(n) = s.parse::<usize>() {
            // If it's a valid number without %, treat it as fixed
            Ok(JobCount::Fixed(n))
        } else {
            // If it's not a number and doesn't end with %, it's an error
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
    _figment: Figment,
    config: Config,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager with layered sources
    pub fn new() -> Result<Self> {
        Self::new_with_env_prefix("CARGO_OPTIMIZE_")
    }
    
    /// Create a new configuration manager with custom environment prefix (for testing)
    pub fn new_with_env_prefix(env_prefix: &str) -> Result<Self> {
        // Use current directory as base
        let base_dir = std::env::current_dir()?;
        Self::new_with_base_dir(&base_dir, env_prefix)
    }
    
    /// Create a new configuration manager with a specific base directory
    /// This allows tests to work with isolated directories without changing the process's current directory
    pub fn new_with_base_dir(base_dir: &Path, env_prefix: &str) -> Result<Self> {
        // Construct absolute path to cargo-optimize.toml
        let config_file = base_dir.join("cargo-optimize.toml");
        
        // Build layered configuration with Figment
        let mut figment = Figment::new()
            // 1. Start with defaults
            .merge(Toml::string(&Self::default_config_toml()));
        
        // 2. Merge with cargo-optimize.toml if it exists
        if config_file.exists() {
            figment = figment.merge(Toml::file(&config_file));
        }
        
        // 3. Override with environment variables
        // Use double underscore for nested keys (e.g., PREFIX_GLOBAL__VERBOSE)
        figment = figment.merge(Env::prefixed(env_prefix).split("__"));
        
        // Extract the configuration
        let mut config: Config = figment.extract()
            .map_err(|e| anyhow::anyhow!("Failed to extract config: {}", e))?;
        
        // Auto-detect hardware if enabled
        if config.global.auto_detect_hardware {
            config.apply_hardware_optimizations()?;
        }
        
        // Use absolute path for .cargo/config.toml
        let config_path = base_dir.join(".cargo").join("config.toml");
        
        Ok(ConfigManager {
            _figment: figment,
            config,
            config_path,
        })
    }
    
    /// Load configuration from a specific profile
    pub fn with_profile(profile: &str) -> Result<Self> {
        Self::with_profile_and_env_prefix(profile, "CARGO_OPTIMIZE_")
    }
    
    /// Load configuration from a specific profile with custom environment prefix (for testing)
    pub fn with_profile_and_env_prefix(profile: &str, env_prefix: &str) -> Result<Self> {
        // Use current directory as base
        let base_dir = std::env::current_dir()?;
        Self::with_profile_and_base_dir(profile, &base_dir, env_prefix)
    }
    
    /// Load configuration from a specific profile with a specific base directory
    pub fn with_profile_and_base_dir(profile: &str, base_dir: &Path, env_prefix: &str) -> Result<Self> {
        // Construct absolute path to cargo-optimize.toml
        let config_file = base_dir.join("cargo-optimize.toml");
        
        // Build layered configuration with Figment
        let mut figment = Figment::new()
            // 1. Start with defaults
            .merge(Toml::string(&Self::default_config_toml()));
        
        // 2. Merge with cargo-optimize.toml if it exists
        if config_file.exists() {
            figment = figment.merge(Toml::file(&config_file));
        }
        
        // 3. Override with environment variables
        figment = figment.merge(Env::prefixed(env_prefix).split("__"));
        
        // Extract the configuration
        let mut config: Config = figment.extract()
            .map_err(|e| anyhow::anyhow!("Failed to extract config: {}", e))?;
        
        // Debug logging to understand what's being loaded
        debug!("Loading configuration with profile: {}", profile);
        debug!("Base directory: {:?}", base_dir);
        debug!("Config file path: {:?}, exists: {}", config_file, config_file.exists());
        debug!("Auto-detect hardware: {}", config.global.auto_detect_hardware);
        debug!("Optimization level: {:?}", config.global.optimization_level);
        
        // Verify the profile exists
        if !config.profiles.contains_key(profile) {
            // If the profile doesn't exist, create a default one
            let default_profile = Profile::default_for_name(profile.to_string());
            config.profiles.insert(profile.to_string(), default_profile);
        }
        
        // Auto-detect hardware if enabled
        if config.global.auto_detect_hardware {
            config.apply_hardware_optimizations()?;
        }
        
        // Use absolute path for .cargo/config.toml
        let config_path = base_dir.join(".cargo").join("config.toml");
        
        // Return the manager with the loaded configuration
        Ok(ConfigManager {
            _figment: figment,
            config,
            config_path,
        })
    }
    
    /// Get the default configuration as TOML string
    fn default_config_toml() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }
    
    /// Apply configuration to .cargo/config.toml while preserving formatting
    pub fn apply(&self) -> Result<()> {
        // Always ensure .cargo directory exists first
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
        }
        
        // Create backup if enabled AND if the config file exists
        if self.config.backup.auto_backup && self.config_path.exists() {
            self.create_backup()?;
        }
        
        // Load or create the document
        let mut doc = if self.config_path.exists() {
            // Read with retry for Windows file locking issues
            let content = self.read_config_with_retry()?;
            content.parse::<DocumentMut>()
                .unwrap_or_else(|_| DocumentMut::new())
        } else {
            DocumentMut::new()
        };
        
        // Apply our optimizations while preserving existing content
        self.apply_to_document(&mut doc)?;
        
        // Always write the config file, even if it's minimal
        // This ensures the file exists after apply() is called
        let content = doc.to_string();
        // If document is empty, at least add a comment
        let final_content = if content.trim().is_empty() {
            "# Cargo configuration managed by cargo-optimize\n".to_string()
        } else {
            content
        };
        
        // Write back the modified document with retry for Windows
        self.write_config_with_retry(&final_content)?;
        
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
        // Ensure backup directory exists - handle both absolute and relative paths
        let backup_dir = if self.config.backup.backup_dir.is_absolute() {
            self.config.backup.backup_dir.clone()
        } else {
            // If relative, it's relative to the config file's parent directory
            // This ensures backups are created in the right place even when
            // the current directory is different
            if let Some(parent) = self.config_path.parent() {
                // Go up one level from .cargo/config.toml to get the project root
                if let Some(project_root) = parent.parent() {
                    project_root.join(&self.config.backup.backup_dir)
                } else {
                    parent.join(&self.config.backup.backup_dir)
                }
            } else {
                // Fallback to current directory if we can't determine parent
                std::env::current_dir()?.join(&self.config.backup.backup_dir)
            }
        };
        
        // Normalize the path to use proper separators for the platform
        let backup_dir = PathBuf::from(backup_dir.to_string_lossy().replace('/', std::path::MAIN_SEPARATOR_STR));
        
        // Create all parent directories if they don't exist
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("Failed to create backup directory: {:?}", backup_dir))?;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_name = format!("config_backup_{}.toml", timestamp);
        let backup_path = backup_dir.join(&backup_name);
        
        // Debug: Check current directory and config path
        debug!("Current dir: {:?}", std::env::current_dir());
        debug!("Config path: {:?}, exists: {}", self.config_path, self.config_path.exists());
        debug!("Backup dir: {:?}, exists: {}", backup_dir, backup_dir.exists());
        
        // Copy current config if it exists
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)
                .context("Failed to read config file for backup")?;
            fs::write(&backup_path, &content)
                .with_context(|| format!("Failed to write backup file: {:?}", backup_path))?;
            info!("Created backup at {:?}", backup_path);
        } else {
            // Create empty backup as marker
            fs::write(&backup_path, "# No previous configuration\n")
                .with_context(|| format!("Failed to create empty backup: {:?}", backup_path))?;
            debug!("Config file does not exist at {:?}, created empty backup", self.config_path);
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
        
        // Read the backup content
        let backup_content = fs::read_to_string(backup_path)
            .context("Failed to read backup file")?;
        
        // Ensure the directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        // Write the backup content to the config file
        fs::write(&self.config_path, backup_content)
            .context("Failed to restore from backup")?;
        
        info!("Restored configuration from {:?}", backup_path);
        Ok(())
    }
    
    /// Clean up old backups, keeping only the most recent ones
    fn cleanup_old_backups(&self) -> Result<()> {
        // Handle both absolute and relative backup paths
        let backup_dir = if self.config.backup.backup_dir.is_absolute() {
            self.config.backup.backup_dir.clone()
        } else {
            // If relative, it's relative to the config file's parent directory
            if let Some(parent) = self.config_path.parent() {
                // Go up one level from .cargo/config.toml to get the project root
                if let Some(project_root) = parent.parent() {
                    project_root.join(&self.config.backup.backup_dir)
                } else {
                    parent.join(&self.config.backup.backup_dir)
                }
            } else {
                // Fallback to current directory if we can't determine parent
                std::env::current_dir()?.join(&self.config.backup.backup_dir)
            }
        };
        
        // Normalize the path to use proper separators for the platform
        let backup_dir = PathBuf::from(backup_dir.to_string_lossy().replace('/', std::path::MAIN_SEPARATOR_STR));
        
        // Return early if backup dir doesn't exist
        if !backup_dir.exists() {
            return Ok(());
        }
        
        let mut backups: Vec<_> = fs::read_dir(&backup_dir)
            .with_context(|| format!("Failed to read backup directory: {:?}", backup_dir))?
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
    
    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }
    
    /// Get a mutable profile by name
    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(name)
    }
    
    /// Generate linker configuration for Cargo
    pub fn generate_linker_config(&self, linker: &str) -> Result<String, ConfigError> {
        // Validate linker is supported
        let supported_linkers = if cfg!(target_os = "windows") {
            vec!["rust-lld", "lld-link.exe", "lld", "link.exe"]
        } else if cfg!(target_os = "linux") {
            vec!["mold", "lld", "gold", "ld"]
        } else if cfg!(target_os = "macos") {
            vec!["zld", "lld", "ld64"]
        } else {
            vec!["lld"]
        };
        
        if !supported_linkers.contains(&linker) {
            return Err(ConfigError::Other(anyhow::anyhow!(
                "Unsupported linker '{}' for current platform", linker
            )));
        }
        
        let mut config = String::new();
        
        // Add target-specific configuration
        if cfg!(target_os = "windows") {
            config.push_str("[target.x86_64-pc-windows-msvc]\n");
            config.push_str(&format!("linker = \"{}\"\n", linker));
            
            // Add rustflags for specific linkers
            match linker {
                "rust-lld" => {
                    config.push_str("rustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]\n");
                }
                "lld-link.exe" => {
                    config.push_str("rustflags = [\"-C\", \"linker=lld-link.exe\"]\n");
                }
                _ => {}
            }
        } else if cfg!(target_os = "linux") {
            config.push_str("[target.x86_64-unknown-linux-gnu]\n");
            config.push_str("linker = \"clang\"\n");
            
            // Add rustflags for specific linkers
            match linker {
                "mold" => {
                    config.push_str("rustflags = [\"-C\", \"link-arg=-fuse-ld=mold\"]\n");
                }
                "lld" => {
                    config.push_str("rustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]\n");
                }
                "gold" => {
                    config.push_str("rustflags = [\"-C\", \"link-arg=-fuse-ld=gold\"]\n");
                }
                _ => {}
            }
        } else if cfg!(target_os = "macos") {
            config.push_str("[target.x86_64-apple-darwin]\n");
            config.push_str(&format!("linker = \"{}\"\n", linker));
            
            if linker == "zld" {
                config.push_str("rustflags = [\"-C\", \"link-arg=-fuse-ld=zld\"]\n");
            }
        }
        
        Ok(config)
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

impl Profile {
    /// Create a default profile for a given name
    pub fn default_for_name(name: String) -> Self {
        Profile {
            name: name.clone(),
            linker: None,
            jobs: None,
            incremental: if name == "dev" || name == "test" {
                Some(true)
            } else {
                Some(false)
            },
            rustflags: Vec::new(),
            cache: CacheSettings::default(),
            target_dir: None,
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
                max_size: Some(CacheSize::Megabytes(1024)),
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
            rustflags: vec![
                "-C".to_string(),
                "opt-level=3".to_string(),
                "-C".to_string(), 
                "lto=true".to_string(),
            ],
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
            // Use proper path construction to avoid mixed separators
            backup_dir: PathBuf::from(".cargo").join("backups"),
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
