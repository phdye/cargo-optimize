//! Build profile management and optimization

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use toml::Value;
use tracing::{debug, info};

/// Build profile type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileType {
    /// Development profile
    Dev,
    /// Release profile
    Release,
    /// Test profile
    Test,
    /// Bench profile
    Bench,
    /// Custom profile
    Custom(String),
}

impl ProfileType {
    /// Get the profile name as used in Cargo.toml
    pub fn name(&self) -> &str {
        match self {
            Self::Dev => "dev",
            Self::Release => "release",
            Self::Test => "test",
            Self::Bench => "bench",
            Self::Custom(name) => name,
        }
    }
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Optimized build profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedProfile {
    /// Profile type
    pub profile_type: ProfileType,
    /// Optimization level (0-3, or "z"/"s")
    pub opt_level: OptLevel,
    /// Debug symbols
    pub debug: bool,
    /// Debug assertions
    pub debug_assertions: bool,
    /// Overflow checks
    pub overflow_checks: bool,
    /// Link-time optimization
    pub lto: Lto,
    /// Panic strategy
    pub panic: PanicStrategy,
    /// Incremental compilation
    pub incremental: bool,
    /// Codegen units
    pub codegen_units: u32,
    /// Strip symbols
    pub strip: Strip,
    /// Split debuginfo
    pub split_debuginfo: Option<SplitDebuginfo>,
}

impl OptimizedProfile {
    /// Create optimized dev profile
    pub fn optimized_dev() -> Self {
        Self {
            profile_type: ProfileType::Dev,
            opt_level: OptLevel::Zero,
            debug: true,
            debug_assertions: true,
            overflow_checks: true,
            lto: Lto::Off,
            panic: PanicStrategy::Unwind,
            incremental: true,
            codegen_units: 256, // Maximum parallelism
            strip: Strip::None,
            split_debuginfo: Some(SplitDebuginfo::Unpacked),
        }
    }

    /// Create optimized release profile
    pub fn optimized_release() -> Self {
        Self {
            profile_type: ProfileType::Release,
            opt_level: OptLevel::Three,
            debug: false,
            debug_assertions: false,
            overflow_checks: false,
            lto: Lto::Thin, // Thin LTO for balance
            panic: PanicStrategy::Abort,
            incremental: false,
            codegen_units: 1, // Better optimization
            strip: Strip::Symbols,
            split_debuginfo: None,
        }
    }

    /// Create optimized test profile
    pub fn optimized_test() -> Self {
        Self {
            profile_type: ProfileType::Test,
            opt_level: OptLevel::Zero,
            debug: true,
            debug_assertions: true,
            overflow_checks: true,
            lto: Lto::Off,
            panic: PanicStrategy::Unwind,
            incremental: true,
            codegen_units: 256, // Fast compilation
            strip: Strip::None,
            split_debuginfo: Some(SplitDebuginfo::Unpacked),
        }
    }

    /// Create optimized bench profile
    pub fn optimized_bench() -> Self {
        Self {
            profile_type: ProfileType::Bench,
            opt_level: OptLevel::Three,
            debug: false,
            debug_assertions: false,
            overflow_checks: false,
            lto: Lto::Thin,
            panic: PanicStrategy::Abort,
            incremental: false,
            codegen_units: 1,
            strip: Strip::Debuginfo,
            split_debuginfo: None,
        }
    }

    /// Create a fast compilation profile
    pub fn fast_compile() -> Self {
        Self {
            profile_type: ProfileType::Custom("fast".to_string()),
            opt_level: OptLevel::Zero,
            debug: false, // No debug symbols for speed
            debug_assertions: false,
            overflow_checks: false,
            lto: Lto::Off,
            panic: PanicStrategy::Abort,
            incremental: true,
            codegen_units: 256,
            strip: Strip::Symbols,
            split_debuginfo: None,
        }
    }

    /// Convert to TOML value
    pub fn to_toml(&self) -> Value {
        let mut table = toml::Table::new();

        table.insert("opt-level".to_string(), self.opt_level.to_toml());
        table.insert("debug".to_string(), Value::Boolean(self.debug));
        table.insert(
            "debug-assertions".to_string(),
            Value::Boolean(self.debug_assertions),
        );
        table.insert(
            "overflow-checks".to_string(),
            Value::Boolean(self.overflow_checks),
        );
        table.insert("lto".to_string(), self.lto.to_toml());
        table.insert("panic".to_string(), Value::String(self.panic.to_string()));
        table.insert("incremental".to_string(), Value::Boolean(self.incremental));
        table.insert(
            "codegen-units".to_string(),
            Value::Integer(self.codegen_units as i64),
        );

        if self.strip != Strip::None {
            table.insert("strip".to_string(), self.strip.to_toml());
        }

        if let Some(split) = &self.split_debuginfo {
            table.insert(
                "split-debuginfo".to_string(),
                Value::String(split.to_string()),
            );
        }

        Value::Table(table)
    }
}

/// Optimization level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptLevel {
    /// No optimizations
    Zero,
    /// Basic optimizations
    One,
    /// Some optimizations
    Two,
    /// All optimizations
    Three,
    /// Optimize for size
    S,
    /// Optimize for size more aggressively
    Z,
}

impl OptLevel {
    fn to_toml(&self) -> Value {
        match self {
            Self::Zero => Value::Integer(0),
            Self::One => Value::Integer(1),
            Self::Two => Value::Integer(2),
            Self::Three => Value::Integer(3),
            Self::S => Value::String("s".to_string()),
            Self::Z => Value::String("z".to_string()),
        }
    }
}

/// Link-time optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lto {
    /// No LTO
    Off,
    /// Thin LTO (faster)
    Thin,
    /// Fat LTO (slower, better optimization)
    Fat,
}

impl Lto {
    fn to_toml(&self) -> Value {
        match self {
            Self::Off => Value::Boolean(false),
            Self::Thin => Value::String("thin".to_string()),
            Self::Fat => Value::Boolean(true),
        }
    }
}

/// Panic strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicStrategy {
    /// Unwind on panic
    Unwind,
    /// Abort on panic
    Abort,
}

impl ToString for PanicStrategy {
    fn to_string(&self) -> String {
        match self {
            Self::Unwind => "unwind".to_string(),
            Self::Abort => "abort".to_string(),
        }
    }
}

/// Symbol stripping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strip {
    /// Don't strip
    None,
    /// Strip debuginfo only
    Debuginfo,
    /// Strip all symbols
    Symbols,
}

impl Strip {
    fn to_toml(&self) -> Value {
        match self {
            Self::None => Value::String("none".to_string()),
            Self::Debuginfo => Value::String("debuginfo".to_string()),
            Self::Symbols => Value::String("symbols".to_string()),
        }
    }
}

/// Split debuginfo strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDebuginfo {
    /// Don't split
    Off,
    /// Split into separate files (unpacked)
    Unpacked,
    /// Split and pack into archive
    Packed,
}

impl ToString for SplitDebuginfo {
    fn to_string(&self) -> String {
        match self {
            Self::Off => "off".to_string(),
            Self::Unpacked => "unpacked".to_string(),
            Self::Packed => "packed".to_string(),
        }
    }
}

/// Profile manager
pub struct ProfileManager {
    /// Profiles to apply
    profiles: HashMap<ProfileType, OptimizedProfile>,
}

impl ProfileManager {
    /// Create a new profile manager with default optimized profiles
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        profiles.insert(ProfileType::Dev, OptimizedProfile::optimized_dev());
        profiles.insert(ProfileType::Release, OptimizedProfile::optimized_release());
        profiles.insert(ProfileType::Test, OptimizedProfile::optimized_test());
        profiles.insert(ProfileType::Bench, OptimizedProfile::optimized_bench());

        Self { profiles }
    }

    /// Add or update a profile
    pub fn set_profile(&mut self, profile: OptimizedProfile) {
        self.profiles.insert(profile.profile_type.clone(), profile);
    }

    /// Apply profiles to Cargo.toml
    pub fn apply_to_cargo_toml(&self, cargo_toml_path: impl AsRef<Path>) -> Result<()> {
        let path = cargo_toml_path.as_ref();
        info!("Applying optimized profiles to {:?}", path);

        // Read existing Cargo.toml
        let contents = std::fs::read_to_string(path)?;
        let mut doc: toml::Value = toml::from_str(&contents).map_err(|e| Error::Toml(e))?;

        // Get or create the profile section
        let profiles = doc
            .as_table_mut()
            .ok_or_else(|| Error::config("Invalid Cargo.toml"))?
            .entry("profile")
            .or_insert(Value::Table(toml::Table::new()));

        let profile_table = profiles
            .as_table_mut()
            .ok_or_else(|| Error::config("Invalid profile section"))?;

        // Apply each profile
        for (profile_type, profile) in &self.profiles {
            debug!("Applying profile: {:?}", profile_type);
            profile_table.insert(profile_type.name().to_string(), profile.to_toml());
        }

        // Write back
        let new_contents = toml::to_string_pretty(&doc)?;
        std::fs::write(path, new_contents)?;

        info!("Successfully applied optimized profiles");
        Ok(())
    }

    /// Generate profile recommendations based on project analysis
    pub fn recommend_profiles(
        project_size: ProjectSize,
        ci_environment: bool,
    ) -> HashMap<ProfileType, OptimizedProfile> {
        let mut profiles = HashMap::new();

        match project_size {
            ProjectSize::Small => {
                // Small projects: prioritize compilation speed
                let mut dev = OptimizedProfile::optimized_dev();
                dev.codegen_units = 256;
                profiles.insert(ProfileType::Dev, dev);

                let mut release = OptimizedProfile::optimized_release();
                release.lto = Lto::Off; // Skip LTO for small projects
                profiles.insert(ProfileType::Release, release);
            }
            ProjectSize::Medium => {
                // Medium projects: balanced approach
                profiles.insert(ProfileType::Dev, OptimizedProfile::optimized_dev());
                profiles.insert(ProfileType::Release, OptimizedProfile::optimized_release());
            }
            ProjectSize::Large => {
                // Large projects: maximize optimization opportunities
                let mut dev = OptimizedProfile::optimized_dev();
                dev.split_debuginfo = Some(SplitDebuginfo::Packed);
                profiles.insert(ProfileType::Dev, dev);

                let mut release = OptimizedProfile::optimized_release();
                release.lto = Lto::Fat; // Full LTO for large projects
                release.codegen_units = 1;
                profiles.insert(ProfileType::Release, release);
            }
        }

        // CI-specific adjustments
        if ci_environment {
            for profile in profiles.values_mut() {
                profile.incremental = false; // Disable incremental in CI
                profile.codegen_units = profile.codegen_units.min(16); // Limit parallelism
            }
        }

        profiles
    }
}

/// Project size classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSize {
    /// Small project (<5k lines)
    Small,
    /// Medium project (5k-50k lines)
    Medium,
    /// Large project (>50k lines)
    Large,
}

impl ProjectSize {
    /// Classify based on lines of code
    pub fn from_lines(lines: usize) -> Self {
        if lines < 5_000 {
            Self::Small
        } else if lines < 50_000 {
            Self::Medium
        } else {
            Self::Large
        }
    }
}
