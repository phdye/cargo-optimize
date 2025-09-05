//! Linker detection and configuration

use crate::{Error, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};
use which::which;

/// Available linkers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Linker {
    /// Default system linker
    Default,
    /// GNU Gold linker
    Gold,
    /// LLVM LLD linker
    Lld,
    /// Mold linker (fastest)
    Mold,
    /// Custom linker
    Custom(PathBuf),
}

impl Linker {
    /// Detect available linkers on the system
    pub fn detect_available() -> Vec<Self> {
        let mut available = vec![Self::Default];

        // Check for mold
        if which("mold").is_ok() || which("ld.mold").is_ok() {
            available.push(Self::Mold);
            debug!("Found mold linker");
        }

        // Check for lld
        if which("lld").is_ok() || which("ld.lld").is_ok() || which("rust-lld").is_ok() {
            available.push(Self::Lld);
            debug!("Found lld linker");
        }

        // Check for gold
        if which("gold").is_ok() || which("ld.gold").is_ok() {
            available.push(Self::Gold);
            debug!("Found gold linker");
        }

        info!("Available linkers: {:?}", available);
        available
    }

    /// Get the fastest available linker
    pub fn fastest_available() -> Self {
        let available = Self::detect_available();

        // Prefer mold > lld > gold > default
        if available.contains(&Self::Mold) {
            Self::Mold
        } else if available.contains(&Self::Lld) {
            Self::Lld
        } else if available.contains(&Self::Gold) {
            Self::Gold
        } else {
            Self::Default
        }
    }

    /// Get the linker executable name
    pub fn executable(&self) -> String {
        match self {
            Self::Default => "cc".to_string(),
            Self::Gold => "gold".to_string(),
            Self::Lld => "lld".to_string(),
            Self::Mold => "mold".to_string(),
            Self::Custom(path) => path.to_string_lossy().to_string(),
        }
    }

    /// Get the linker flag for rustc
    pub fn rustc_flag(&self) -> Option<String> {
        match self {
            Self::Default => None,
            Self::Gold => Some("-Clink-arg=-fuse-ld=gold".to_string()),
            Self::Lld => Some("-Clink-arg=-fuse-ld=lld".to_string()),
            Self::Mold => Some("-Clink-arg=-fuse-ld=mold".to_string()),
            Self::Custom(path) => Some(format!("-Clink-arg=-fuse-ld={}", path.display())),
        }
    }

    /// Get cargo config for this linker
    pub fn cargo_config(&self, target: &str) -> Option<String> {
        match self {
            Self::Default => None,
            Self::Gold | Self::Lld | Self::Mold => Some(format!(
                "[target.{}]\nlinker = \"clang\"\nrustflags = [\"{}\"]",
                target,
                self.rustc_flag().unwrap()
            )),
            Self::Custom(path) => Some(format!(
                "[target.{}]\nlinker = \"{}\"",
                target,
                path.display()
            )),
        }
    }

    /// Check if this linker is installed
    pub fn is_installed(&self) -> bool {
        match self {
            Self::Default => true,
            Self::Gold => which("gold").is_ok() || which("ld.gold").is_ok(),
            Self::Lld => {
                which("lld").is_ok() || which("ld.lld").is_ok() || which("rust-lld").is_ok()
            }
            Self::Mold => which("mold").is_ok() || which("ld.mold").is_ok(),
            Self::Custom(path) => path.exists(),
        }
    }

    /// Get installation instructions for this linker
    pub fn install_instructions(&self) -> &str {
        match self {
            Self::Default => "Default linker is always available",
            Self::Gold => {
                if cfg!(target_os = "linux") {
                    "Install with: sudo apt-get install binutils-gold (Debian/Ubuntu) or sudo dnf install binutils-gold (Fedora)"
                } else {
                    "Gold linker is primarily available on Linux"
                }
            }
            Self::Lld => {
                if cfg!(target_os = "linux") {
                    "Install with: sudo apt-get install lld (Debian/Ubuntu) or sudo dnf install lld (Fedora)"
                } else if cfg!(target_os = "macos") {
                    "Install with: brew install llvm"
                } else if cfg!(target_os = "windows") {
                    "Install with: scoop install llvm or download from https://releases.llvm.org/"
                } else {
                    "Check your package manager for 'lld' or 'llvm'"
                }
            }
            Self::Mold => {
                if cfg!(target_os = "linux") {
                    "Install from: https://github.com/rui314/mold\nOr: sudo apt-get install mold (Ubuntu 22.04+)"
                } else {
                    "Mold is currently only available on Linux. Consider using lld instead."
                }
            }
            Self::Custom(_) => "Custom linker - ensure it's in your PATH",
        }
    }
}

/// Linker configuration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Selected linker
    pub linker: Linker,
    /// Target triple
    pub target: String,
    /// Additional linker arguments
    pub extra_args: Vec<String>,
    /// Use thin LTO
    pub thin_lto: bool,
    /// Number of LTO codegen units
    pub lto_codegen_units: Option<usize>,
}

impl LinkerConfig {
    /// Create a new linker configuration
    pub fn new(target: String) -> Self {
        Self {
            linker: Linker::fastest_available(),
            target,
            extra_args: Vec::new(),
            thin_lto: false,
            lto_codegen_units: None,
        }
    }

    /// Auto-detect best configuration
    pub fn auto_detect() -> Result<Self> {
        let target = detect_target()?;
        let mut config = Self::new(target);

        // Enable thin LTO if available
        config.thin_lto = true;

        // Add platform-specific optimizations
        if cfg!(target_os = "linux") {
            config.extra_args.push("--as-needed".to_string());
            config.extra_args.push("--gc-sections".to_string());
        }

        Ok(config)
    }

    /// Apply this configuration to the environment
    pub fn apply(&self) -> Result<()> {
        info!("Applying linker configuration: {:?}", self);

        // Set RUSTFLAGS
        if let Some(flag) = self.linker.rustc_flag() {
            let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();
            if !rustflags.is_empty() {
                rustflags.push(' ');
            }
            rustflags.push_str(&flag);

            // Add extra args
            for arg in &self.extra_args {
                rustflags.push_str(&format!(" -Clink-arg={}", arg));
            }

            // Add LTO settings
            if self.thin_lto {
                rustflags.push_str(" -Clto=thin");
                if let Some(units) = self.lto_codegen_units {
                    rustflags.push_str(&format!(" -Ccodegen-units={}", units));
                }
            }

            env::set_var("RUSTFLAGS", rustflags);
            debug!("Set RUSTFLAGS for linker configuration");
        }

        // Write cargo config if needed
        if let Some(config_content) = self.linker.cargo_config(&self.target) {
            self.write_cargo_config(&config_content)?;
        }

        Ok(())
    }

    /// Write cargo configuration
    fn write_cargo_config(&self, content: &str) -> Result<()> {
        let cargo_dir = PathBuf::from(".cargo");
        std::fs::create_dir_all(&cargo_dir)?;

        let config_path = cargo_dir.join("config.toml");

        // Read existing config if it exists
        let mut existing_config = if config_path.exists() {
            std::fs::read_to_string(&config_path)?
        } else {
            String::new()
        };

        // Append our configuration if not already present
        if !existing_config.contains(&format!("[target.{}]", self.target)) {
            if !existing_config.is_empty() && !existing_config.ends_with('\n') {
                existing_config.push('\n');
            }
            existing_config.push_str("\n# Added by cargo-optimize\n");
            existing_config.push_str(content);
            existing_config.push('\n');

            std::fs::write(&config_path, existing_config)?;
            info!("Updated .cargo/config.toml with linker configuration");
        }

        Ok(())
    }
}

/// Detect the current target triple
fn detect_target() -> Result<String> {
    // Try to get from environment
    if let Ok(target) = env::var("TARGET") {
        return Ok(target);
    }

    // Try to get from rustc
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .map_err(|e| Error::detection(format!("Failed to run rustc: {}", e)))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        if line.starts_with("host:") {
            if let Some(target) = line.split_whitespace().nth(1) {
                return Ok(target.to_string());
            }
        }
    }

    Err(Error::detection("Could not detect target triple"))
}

/// Platform-specific linker optimizations
pub struct LinkerOptimizations;

impl LinkerOptimizations {
    /// Get recommended linker for the current platform
    pub fn recommended_linker() -> Linker {
        if cfg!(target_os = "linux") {
            // On Linux, prefer mold > lld > gold
            Linker::fastest_available()
        } else if cfg!(target_os = "macos") {
            // On macOS, lld is the best option (mold doesn't support macOS yet)
            if Linker::Lld.is_installed() {
                Linker::Lld
            } else {
                Linker::Default
            }
        } else if cfg!(target_os = "windows") {
            // On Windows, lld works well
            if Linker::Lld.is_installed() {
                Linker::Lld
            } else {
                Linker::Default
            }
        } else {
            Linker::Default
        }
    }

    /// Get platform-specific linker flags
    pub fn platform_flags() -> Vec<String> {
        let mut flags = Vec::new();

        if cfg!(target_os = "linux") {
            // Linux-specific optimizations
            flags.push("--as-needed".to_string());
            flags.push("--gc-sections".to_string());
            flags.push("-O2".to_string());
        } else if cfg!(target_os = "macos") {
            // macOS-specific optimizations
            flags.push("-dead_strip".to_string());
        } else if cfg!(target_os = "windows") {
            // Windows-specific optimizations
            if cfg!(target_env = "msvc") {
                flags.push("/OPT:REF".to_string());
                flags.push("/OPT:ICF".to_string());
            }
        }

        flags
    }
}
