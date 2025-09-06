//! Configuration for cargo-optimize

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure for cargo-optimize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Optimization level
    pub optimization_level: OptimizationLevel,

    /// Enable hardware detection
    pub auto_detect_hardware: bool,

    /// Enable project analysis
    pub analyze_project: bool,

    /// Enable linker optimization
    pub optimize_linker: bool,

    /// Enable build caching
    pub enable_cache: bool,

    /// Number of parallel jobs (None = auto-detect)
    pub parallel_jobs: Option<usize>,

    /// Custom linker path
    pub custom_linker: Option<PathBuf>,

    /// Enable incremental compilation
    pub incremental: bool,

    /// Split debuginfo
    pub split_debuginfo: bool,

    /// Target CPU (None = native)
    pub target_cpu: Option<String>,

    /// Additional cargo flags
    pub extra_cargo_flags: Vec<String>,

    /// Additional rustc flags
    pub extra_rustc_flags: Vec<String>,

    /// Enable verbose output
    pub verbose: bool,

    /// Dry run (don't apply changes)
    pub dry_run: bool,
}

impl Config {
    /// Create a new configuration with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the optimization level
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) -> &mut Self {
        self.optimization_level = level;
        self
    }

    /// Enable or disable hardware auto-detection
    pub fn set_auto_detect(&mut self, enabled: bool) -> &mut Self {
        self.auto_detect_hardware = enabled;
        self
    }

    /// Set the number of parallel jobs
    pub fn set_parallel_jobs(&mut self, jobs: usize) -> &mut Self {
        // Add reasonable bounds - max 1000 parallel jobs
        // This prevents issues with extreme values like usize::MAX
        let jobs = jobs.min(1000);
        self.parallel_jobs = Some(jobs);
        self
    }

    /// Enable verbose output
    pub fn verbose(&mut self) -> &mut Self {
        self.verbose = true;
        self
    }

    /// Enable dry run mode
    pub fn dry_run(&mut self) -> &mut Self {
        self.dry_run = true;
        self
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Balanced,
            auto_detect_hardware: true,
            analyze_project: true,
            optimize_linker: true,
            enable_cache: true,
            parallel_jobs: None,
            custom_linker: None,
            incremental: true,
            split_debuginfo: true,
            target_cpu: None,
            extra_cargo_flags: Vec::new(),
            extra_rustc_flags: Vec::new(),
            verbose: false,
            dry_run: false,
        }
    }
}

/// Optimization level presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    /// Conservative optimizations (safest, minimal changes)
    Conservative,

    /// Balanced optimizations (recommended)
    Balanced,

    /// Aggressive optimizations (maximum speed, may affect stability)
    Aggressive,

    /// Custom optimization (use individual settings)
    Custom,
}

impl OptimizationLevel {
    /// Get a description of this optimization level
    pub fn description(&self) -> &'static str {
        match self {
            Self::Conservative => "Conservative optimizations - safest, minimal changes",
            Self::Balanced => "Balanced optimizations - recommended for most projects",
            Self::Aggressive => "Aggressive optimizations - maximum speed, may affect stability",
            Self::Custom => "Custom optimizations - use individual settings",
        }
    }

    /// Check if this level should enable a specific feature
    pub fn should_enable(&self, feature: OptimizationFeature) -> bool {
        use OptimizationFeature::*;

        match (self, feature) {
            // Conservative enables only the safest features
            (Self::Conservative, FastLinker) => true,
            (Self::Conservative, Incremental) => true,
            (Self::Conservative, ParallelFrontend) => false,
            (Self::Conservative, SplitDebuginfo) => false,
            (Self::Conservative, Sccache) => true,
            (Self::Conservative, NativeCpu) => false,
            (Self::Conservative, ThinLto) => false,

            // Balanced enables most features
            (Self::Balanced, FastLinker) => true,
            (Self::Balanced, Incremental) => true,
            (Self::Balanced, ParallelFrontend) => true,
            (Self::Balanced, SplitDebuginfo) => true,
            (Self::Balanced, Sccache) => true,
            (Self::Balanced, NativeCpu) => false,
            (Self::Balanced, ThinLto) => true,

            // Aggressive enables everything
            (Self::Aggressive, _) => true,

            // Custom doesn't make automatic decisions
            (Self::Custom, _) => false,
        }
    }
}

/// Individual optimization features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationFeature {
    /// Use a fast linker (mold, lld, etc.)
    FastLinker,
    /// Enable incremental compilation
    Incremental,
    /// Enable parallel frontend
    ParallelFrontend,
    /// Split debuginfo into separate files
    SplitDebuginfo,
    /// Use sccache for caching
    Sccache,
    /// Optimize for native CPU
    NativeCpu,
    /// Use thin LTO
    ThinLto,
}

/// Build profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Include debug symbols
    pub debug: bool,
    /// Enable debug assertions
    pub debug_assertions: bool,
    /// Enable overflow checks
    pub overflow_checks: bool,
    /// Link-time optimization
    pub lto: LtoConfig,
    /// Panic strategy
    pub panic: PanicStrategy,
    /// Enable incremental compilation
    pub incremental: bool,
    /// Number of codegen units
    pub codegen_units: u16,
    /// Strip symbols
    pub strip: StripConfig,
}

/// LTO configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LtoConfig {
    /// No LTO
    Off,
    /// Thin LTO (faster)
    Thin,
    /// Fat LTO (slower, smaller binary)
    Fat,
}

/// Panic strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanicStrategy {
    /// Unwind on panic
    Unwind,
    /// Abort on panic
    Abort,
}

/// Symbol stripping configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StripConfig {
    /// Don't strip
    None,
    /// Strip debuginfo
    Debuginfo,
    /// Strip all symbols
    Symbols,
}
