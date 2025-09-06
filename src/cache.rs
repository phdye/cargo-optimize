//! Build cache configuration and management

use crate::{Error, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};
use which::which;

/// Build cache system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheSystem {
    /// No caching
    None,
    /// sccache (Rust-specific, recommended)
    Sccache,
    /// ccache (general purpose)
    Ccache,
    /// Custom cache system
    Custom(String),
}

impl CacheSystem {
    /// Detect available cache systems
    pub fn detect_available() -> Vec<Self> {
        let mut available = vec![Self::None];

        // Check for sccache
        if which("sccache").is_ok() {
            available.push(Self::Sccache);
            debug!("Found sccache");
        }

        // Check for ccache
        if which("ccache").is_ok() {
            available.push(Self::Ccache);
            debug!("Found ccache");
        }

        info!("Available cache systems: {:?}", available);
        available
    }

    /// Get the best available cache system
    pub fn best_available() -> Self {
        let available = Self::detect_available();

        // Prefer sccache > ccache > none
        if available.contains(&Self::Sccache) {
            Self::Sccache
        } else if available.contains(&Self::Ccache) {
            Self::Ccache
        } else {
            Self::None
        }
    }

    /// Check if this cache system is installed
    pub fn is_installed(&self) -> bool {
        match self {
            Self::None => true,
            Self::Sccache => which("sccache").is_ok(),
            Self::Ccache => which("ccache").is_ok(),
            Self::Custom(cmd) => which(cmd).is_ok(),
        }
    }

    /// Get installation instructions
    pub fn install_instructions(&self) -> &str {
        match self {
            Self::None => "No cache system",
            Self::Sccache => {
                if cfg!(target_os = "linux") {
                    "Install with: cargo install sccache --locked"
                } else if cfg!(target_os = "macos") {
                    "Install with: brew install sccache or cargo install sccache --locked"
                } else if cfg!(target_os = "windows") {
                    "Install with: scoop install sccache or cargo install sccache --locked"
                } else {
                    "Install with: cargo install sccache --locked"
                }
            }
            Self::Ccache => {
                if cfg!(target_os = "linux") {
                    "Install with: sudo apt-get install ccache (Debian/Ubuntu) or sudo dnf install ccache (Fedora)"
                } else if cfg!(target_os = "macos") {
                    "Install with: brew install ccache"
                } else if cfg!(target_os = "windows") {
                    "Install with: scoop install ccache"
                } else {
                    "Check your package manager for 'ccache'"
                }
            }
            Self::Custom(_) => "Custom cache system - ensure it's in your PATH",
        }
    }

    /// Get the wrapper command for this cache system
    pub fn wrapper_command(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Sccache => Some("sccache".to_string()),
            Self::Ccache => Some("ccache".to_string()),
            Self::Custom(cmd) => Some(cmd.clone()),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache system to use
    pub system: CacheSystem,
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
    /// Maximum cache size in GB
    pub max_size_gb: Option<u64>,
    /// Enable distributed caching
    pub distributed: bool,
    /// Cache statistics
    pub show_stats: bool,
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new() -> Self {
        Self {
            system: CacheSystem::best_available(),
            cache_dir: None,
            max_size_gb: Some(10), // 10GB default
            distributed: false,
            show_stats: true,
        }
    }

    /// Create a configuration with no caching
    pub fn none() -> Self {
        Self {
            system: CacheSystem::None,
            cache_dir: None,
            max_size_gb: None,
            distributed: false,
            show_stats: false,
        }
    }

    /// Auto-detect best configuration
    pub fn auto_detect() -> Result<Self> {
        let mut config = Self::new();

        // Set cache directory based on platform
        config.cache_dir = Some(get_cache_dir()?);

        // Adjust size based on available disk space
        if let Ok(available_space) = get_available_disk_space() {
            // Use up to 10% of available space, max 50GB
            let max_cache = (available_space / 10).min(50 * 1024 * 1024 * 1024);
            config.max_size_gb = Some(max_cache / (1024 * 1024 * 1024));
        }

        Ok(config)
    }

    /// Apply this configuration
    pub fn apply(&self) -> Result<()> {
        info!("Applying cache configuration: {:?}", self);

        match &self.system {
            CacheSystem::Sccache => self.configure_sccache()?,
            CacheSystem::Ccache => self.configure_ccache()?,
            CacheSystem::None => {
                debug!("No cache system configured");
            }
            CacheSystem::Custom(_) => {
                warn!("Custom cache system - manual configuration required");
            }
        }

        Ok(())
    }

    /// Configure sccache
    fn configure_sccache(&self) -> Result<()> {
        // Set RUSTC_WRAPPER
        env::set_var("RUSTC_WRAPPER", "sccache");

        // Set cache directory
        if let Some(dir) = &self.cache_dir {
            env::set_var("SCCACHE_DIR", dir);
            std::fs::create_dir_all(dir)?;
        }

        // Set max cache size
        if let Some(size_gb) = self.max_size_gb {
            env::set_var("SCCACHE_CACHE_SIZE", format!("{}G", size_gb));
        }

        // Start sccache server
        Command::new("sccache").arg("--start-server").output().ok(); // Ignore if already running

        if self.show_stats {
            // Show initial stats
            if let Ok(output) = Command::new("sccache").arg("-s").output() {
                debug!(
                    "sccache stats:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                );
            }
        }

        info!("sccache configured successfully");
        Ok(())
    }

    /// Configure ccache
    fn configure_ccache(&self) -> Result<()> {
        // For ccache with Rust, we need to set it as the compiler wrapper
        env::set_var("RUSTC_WRAPPER", "ccache");

        // Set cache directory
        if let Some(dir) = &self.cache_dir {
            env::set_var("CCACHE_DIR", dir);
            std::fs::create_dir_all(dir)?;
        }

        // Set max cache size
        if let Some(size_gb) = self.max_size_gb {
            Command::new("ccache")
                .arg("--max-size")
                .arg(format!("{}G", size_gb))
                .output()
                .map_err(|e| Error::cache(format!("Failed to set ccache size: {}", e)))?;
        }

        // Enable compiler colors
        env::set_var("CCACHE_COMPRESS", "1");
        env::set_var("CCACHE_COMPILERCHECK", "content");

        if self.show_stats {
            // Show initial stats
            if let Ok(output) = Command::new("ccache").arg("-s").output() {
                debug!("ccache stats:\n{}", String::from_utf8_lossy(&output.stdout));
            }
        }

        info!("ccache configured successfully");
        Ok(())
    }

    /// Install the cache system if not present
    pub fn install_if_needed(&self) -> Result<()> {
        if !self.system.is_installed() {
            warn!(
                "Cache system '{}' is not installed. {}",
                self.system.wrapper_command().unwrap_or_default(),
                self.system.install_instructions()
            );

            // Try to auto-install sccache using cargo
            if matches!(self.system, CacheSystem::Sccache) {
                info!("Attempting to install sccache via cargo...");
                let result = Command::new("cargo")
                    .args(&["install", "sccache", "--locked"])
                    .status()
                    .map_err(|e| Error::cache(format!("Failed to install sccache: {}", e)))?;

                if result.success() {
                    info!("Successfully installed sccache");
                } else {
                    return Err(Error::cache("Failed to install sccache"));
                }
            }
        }
        Ok(())
    }

    /// Clear the cache
    pub fn clear_cache(&self) -> Result<()> {
        match &self.system {
            CacheSystem::Sccache => {
                Command::new("sccache").arg("--stop-server").output().ok();

                if let Some(dir) = &self.cache_dir {
                    if dir.exists() {
                        std::fs::remove_dir_all(dir)?;
                    }
                }

                info!("Cleared sccache");
            }
            CacheSystem::Ccache => {
                Command::new("ccache")
                    .arg("-C")
                    .output()
                    .map_err(|e| Error::cache(format!("Failed to clear ccache: {}", e)))?;

                info!("Cleared ccache");
            }
            _ => {}
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> Result<CacheStats> {
        match &self.system {
            CacheSystem::Sccache => {
                let output = Command::new("sccache")
                    .arg("-s")
                    .output()
                    .map_err(|e| Error::cache(format!("Failed to get sccache stats: {}", e)))?;

                let stats_str = String::from_utf8_lossy(&output.stdout);
                CacheStats::parse_sccache(&stats_str)
            }
            CacheSystem::Ccache => {
                let output = Command::new("ccache")
                    .arg("-s")
                    .output()
                    .map_err(|e| Error::cache(format!("Failed to get ccache stats: {}", e)))?;

                let stats_str = String::from_utf8_lossy(&output.stdout);
                CacheStats::parse_ccache(&stats_str)
            }
            _ => Ok(CacheStats::default()),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache size in bytes
    pub size_bytes: u64,
    /// Number of cached files
    pub file_count: u64,
}

impl CacheStats {
    /// Parse sccache statistics output
    fn parse_sccache(output: &str) -> Result<Self> {
        let mut stats = Self::default();

        for line in output.lines() {
            if line.contains("Cache hits") {
                if let Some(num) = line.split_whitespace().last() {
                    stats.hits = num.parse().unwrap_or(0);
                }
            } else if line.contains("Cache misses") {
                if let Some(num) = line.split_whitespace().last() {
                    stats.misses = num.parse().unwrap_or(0);
                }
            }
        }

        Ok(stats)
    }

    /// Parse ccache statistics output
    fn parse_ccache(output: &str) -> Result<Self> {
        let mut stats = Self::default();

        for line in output.lines() {
            if line.contains("cache hit") && line.contains("direct") {
                if let Some(num) = line.split_whitespace().last() {
                    stats.hits = num.parse().unwrap_or(0);
                }
            } else if line.contains("cache miss") {
                if let Some(num) = line.split_whitespace().last() {
                    stats.misses = num.parse().unwrap_or(0);
                }
            }
        }

        Ok(stats)
    }

    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            (self.hits as f64) / (total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Get the default cache directory
fn get_cache_dir() -> Result<PathBuf> {
    if let Some(dir) = dirs::cache_dir() {
        Ok(dir.join("cargo-optimize"))
    } else {
        // Fallback to temp directory
        Ok(env::temp_dir().join("cargo-optimize-cache"))
    }
}

/// Get available disk space (simplified)
fn get_available_disk_space() -> Result<u64> {
    // This is a simplified implementation
    // In a real implementation, you'd use platform-specific APIs
    Ok(50 * 1024 * 1024 * 1024) // Default to 50GB
}
