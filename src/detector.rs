//! Hardware and environment detection

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tracing::{debug, info};

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Number of logical cores (with hyperthreading)
    pub logical_cores: usize,
    /// Number of physical cores
    pub physical_cores: usize,
    /// CPU model name
    pub model_name: String,
    /// Base frequency in MHz
    pub base_frequency: Option<u64>,
    /// Max frequency in MHz
    pub max_frequency: Option<u64>,
    /// CPU features (SSE, AVX, etc.)
    pub features: Vec<String>,
}

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}/{} cores)", 
               self.model_name, self.logical_cores, self.physical_cores)
    }
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total memory in bytes
    pub total_bytes: u64,
    /// Available memory in bytes
    pub available_bytes: u64,
    /// Total swap in bytes
    pub swap_total_bytes: u64,
    /// Available swap in bytes
    pub swap_available_bytes: u64,
}

impl MemoryInfo {
    /// Get total memory in GB
    pub fn total_gb(&self) -> u64 {
        self.total_bytes / (1024 * 1024 * 1024)
    }
    
    /// Get available memory in GB
    pub fn available_gb(&self) -> u64 {
        self.available_bytes / (1024 * 1024 * 1024)
    }
    
    /// Get memory usage percentage
    pub fn usage_percent(&self) -> u8 {
        if self.total_bytes == 0 {
            return 0;
        }
        let used = self.total_bytes - self.available_bytes;
        ((used * 100) / self.total_bytes).min(100) as u8
    }
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    /// OS family (windows, unix, etc.)
    pub family: String,
    /// OS name (Ubuntu, Windows 11, etc.)
    pub name: String,
    /// OS version
    pub version: String,
    /// Architecture (x86_64, aarch64, etc.)
    pub arch: String,
    /// Is 64-bit OS
    pub is_64bit: bool,
}

impl OsInfo {
    /// Check if this is a Unix-like OS
    pub fn is_unix(&self) -> bool {
        self.family.to_lowercase() == "unix" || 
        self.family.to_lowercase() == "linux" ||
        self.family.to_lowercase() == "macos"
    }
    
    /// Check if this is Windows
    pub fn is_windows(&self) -> bool {
        self.family.to_lowercase() == "windows"
    }
}

/// Hardware information structure expected by tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU information
    pub cpu: CpuInfo,
    /// Memory information
    pub memory: MemoryInfo,
    /// Operating system information
    pub os: OsInfo,
    
    // Keep existing fields for backward compatibility
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Number of logical CPUs (with hyperthreading)
    pub logical_cpus: usize,
    /// Total system memory in bytes
    pub total_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// CPU architecture
    pub cpu_arch: CpuArchitecture,
    /// CPU brand string
    pub cpu_brand: String,
    /// CPU frequency in MHz
    pub cpu_frequency: u64,
}

impl HardwareInfo {
    /// Detect current hardware
    pub fn detect() -> Result<Self> {
        let detector = SystemDetector::new();
        Ok(detector.detect_all())
    }
    
    /// Get recommended number of parallel jobs
    pub fn recommended_jobs(&self) -> usize {
        // Use logical CPUs but leave some headroom
        let jobs = (self.logical_cpus as f32 * 0.75).ceil() as usize;
        jobs.max(1)
    }
    
    /// Check if we have enough memory for aggressive optimizations
    pub fn has_sufficient_memory(&self) -> bool {
        // Need at least 4GB available
        self.available_memory >= 4 * 1024 * 1024 * 1024
    }
    
    /// Get the CPU target string for native optimizations
    pub fn cpu_target(&self) -> &str {
        match self.cpu_arch {
            CpuArchitecture::X86_64 => "native",
            CpuArchitecture::Aarch64 => "native",
            CpuArchitecture::X86 => "pentium4",
            CpuArchitecture::Arm => "armv7",
            CpuArchitecture::Other(_) => "generic",
        }
    }
}

/// System detector with caching capabilities
pub struct SystemDetector {
    cached_cpu: Arc<Mutex<Option<CpuInfo>>>,
    cached_memory: Arc<Mutex<Option<MemoryInfo>>>,
    cached_os: Arc<Mutex<Option<OsInfo>>>,
}

impl SystemDetector {
    /// Create a new system detector
    pub fn new() -> Self {
        Self {
            cached_cpu: Arc::new(Mutex::new(None)),
            cached_memory: Arc::new(Mutex::new(None)),
            cached_os: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Detect CPU information
    pub fn detect_cpu(&self) -> CpuInfo {
        // Check cache first
        {
            let cache = self.cached_cpu.lock().unwrap();
            if let Some(ref cpu_info) = *cache {
                return cpu_info.clone();
            }
        }
        
        // Detect CPU info
        info!("Detecting CPU information...");
        
        let mut system = System::new_all();
        system.refresh_all();
        
        let logical_cores = num_cpus::get();
        let physical_cores = num_cpus::get_physical();
        
        let model_name = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());
            
        let base_frequency = system
            .cpus()
            .first()
            .map(|cpu| cpu.frequency());
            
        // Try to detect CPU features (simplified version)
        let features = Self::detect_cpu_features();
        
        let cpu_info = CpuInfo {
            logical_cores,
            physical_cores,
            model_name,
            base_frequency,
            max_frequency: base_frequency, // Simplified
            features,
        };
        
        // Cache the result
        {
            let mut cache = self.cached_cpu.lock().unwrap();
            *cache = Some(cpu_info.clone());
        }
        
        debug!("CPU detected: {:?}", cpu_info);
        cpu_info
    }
    
    /// Detect memory information
    pub fn detect_memory(&self) -> MemoryInfo {
        // Check cache first
        {
            let cache = self.cached_memory.lock().unwrap();
            if let Some(ref memory_info) = *cache {
                return memory_info.clone();
            }
        }
        
        // Detect memory info
        info!("Detecting memory information...");
        
        let mut system = System::new_all();
        system.refresh_all();
        
        let memory_info = MemoryInfo {
            total_bytes: system.total_memory(),
            available_bytes: system.available_memory(),
            swap_total_bytes: system.total_swap(),
            swap_available_bytes: system.free_swap(),
        };
        
        // Cache the result
        {
            let mut cache = self.cached_memory.lock().unwrap();
            *cache = Some(memory_info.clone());
        }
        
        debug!("Memory detected: {:?}", memory_info);
        memory_info
    }
    
    /// Detect operating system information
    pub fn detect_os(&self) -> OsInfo {
        // Check cache first
        {
            let cache = self.cached_os.lock().unwrap();
            if let Some(ref os_info) = *cache {
                return os_info.clone();
            }
        }
        
        // Detect OS info
        info!("Detecting operating system information...");
        
        let family = if cfg!(windows) {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "unix".to_string()
        } else if cfg!(unix) {
            "unix".to_string()
        } else {
            "unknown".to_string()
        };
        
        let name = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let is_64bit = arch.contains("64");
        
        // Try to get more detailed version info
        let version = Self::detect_os_version();
        
        let os_info = OsInfo {
            family,
            name,
            version,
            arch,
            is_64bit,
        };
        
        // Cache the result
        {
            let mut cache = self.cached_os.lock().unwrap();
            *cache = Some(os_info.clone());
        }
        
        debug!("OS detected: {:?}", os_info);
        os_info
    }
    
    /// Detect all hardware information
    pub fn detect_all(&self) -> HardwareInfo {
        let cpu = self.detect_cpu();
        let memory = self.detect_memory();
        let os = self.detect_os();
        
        // Convert to legacy format for backward compatibility
        let cpu_arch = CpuArchitecture::detect().unwrap_or(CpuArchitecture::Other("unknown".to_string()));
        
        HardwareInfo {
            // New format
            cpu: cpu.clone(),
            memory: memory.clone(),
            os: os.clone(),
            
            // Legacy format
            cpu_cores: cpu.physical_cores,
            logical_cpus: cpu.logical_cores,
            total_memory: memory.total_bytes,
            available_memory: memory.available_bytes,
            cpu_arch,
            cpu_brand: cpu.model_name,
            cpu_frequency: cpu.base_frequency.unwrap_or(0),
        }
    }
    
    /// Detect CPU features (simplified implementation)
    fn detect_cpu_features() -> Vec<String> {
        let mut features = Vec::new();
        
        // This is a simplified version - in a real implementation,
        // we'd use CPUID or similar platform-specific methods
        #[cfg(target_arch = "x86_64")]
        {
            features.push("sse2".to_string());
            if is_x86_feature_detected!("sse4.1") {
                features.push("sse4.1".to_string());
            }
            if is_x86_feature_detected!("sse4.2") {
                features.push("sse4.2".to_string());
            }
            if is_x86_feature_detected!("avx") {
                features.push("avx".to_string());
            }
            if is_x86_feature_detected!("avx2") {
                features.push("avx2".to_string());
            }
        }
        
        features
    }
    
    /// Detect OS version (simplified implementation)
    fn detect_os_version() -> String {
        #[cfg(windows)]
        {
            // On Windows, try to get version from registry or WMI
            "Unknown".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            // Try to read /etc/os-release
            if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
                for line in contents.lines() {
                    if line.starts_with("VERSION_ID=") {
                        return line.split('=').nth(1)
                            .unwrap_or("Unknown")
                            .trim_matches('"')
                            .to_string();
                    }
                }
            }
            "Unknown".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            // Try to get macOS version
            use std::process::Command;
            if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
            "Unknown".to_string()
        }
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        {
            "Unknown".to_string()
        }
    }
}

impl Default for SystemDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU architecture
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuArchitecture {
    /// x86_64 / AMD64
    X86_64,
    /// x86 32-bit
    X86,
    /// ARM 64-bit
    Aarch64,
    /// ARM 32-bit
    Arm,
    /// Other architecture
    Other(String),
}

impl CpuArchitecture {
    /// Detect current CPU architecture
    pub fn detect() -> Result<Self> {
        let arch = env::consts::ARCH;
        
        Ok(match arch {
            "x86_64" => Self::X86_64,
            "x86" => Self::X86,
            "aarch64" => Self::Aarch64,
            "arm" => Self::Arm,
            other => Self::Other(other.to_string()),
        })
    }
    
    /// Check if this architecture supports AVX2
    pub fn supports_avx2(&self) -> bool {
        matches!(self, Self::X86_64)
        // Note: In a real implementation, we'd use CPUID to check
    }
    
    /// Check if this architecture supports NEON
    pub fn supports_neon(&self) -> bool {
        matches!(self, Self::Aarch64 | Self::Arm)
    }
}

/// Operating system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingSystem {
    /// Linux
    Linux,
    /// macOS
    MacOS,
    /// Windows
    Windows,
    /// FreeBSD
    FreeBSD,
    /// Other OS
    Other(String),
}

impl OperatingSystem {
    /// Detect current operating system
    pub fn detect() -> Result<Self> {
        let os = env::consts::OS;
        
        Ok(match os {
            "linux" => Self::Linux,
            "macos" => Self::MacOS,
            "windows" => Self::Windows,
            "freebsd" => Self::FreeBSD,
            other => Self::Other(other.to_string()),
        })
    }
    
    /// Get the preferred linker for this OS
    pub fn preferred_linker(&self) -> &str {
        match self {
            Self::Linux => "mold",  // or lld
            Self::MacOS => "lld",   // mold doesn't support macOS yet
            Self::Windows => "lld",
            Self::FreeBSD => "lld",
            Self::Other(_) => "default",
        }
    }
    
    /// Check if this OS supports a specific linker
    pub fn supports_linker(&self, linker: &str) -> bool {
        match (self, linker) {
            (Self::Linux, "mold" | "lld" | "gold") => true,
            (Self::MacOS, "lld") => true,
            (Self::Windows, "lld" | "link") => true,
            (Self::FreeBSD, "lld") => true,
            _ => false,
        }
    }
    
    /// Get the file extension for executables
    pub fn exe_extension(&self) -> &str {
        match self {
            Self::Windows => ".exe",
            _ => "",
        }
    }
    
    /// Get the file extension for dynamic libraries
    pub fn dylib_extension(&self) -> &str {
        match self {
            Self::Windows => ".dll",
            Self::MacOS => ".dylib",
            _ => ".so",
        }
    }
}

/// Environment detection result
#[derive(Debug, Clone)]
pub struct Environment {
    /// Hardware information
    pub hardware: HardwareInfo,
    /// Toolchain information
    pub toolchain: ToolchainInfo,
    /// CI environment detection
    pub ci_environment: Option<CiEnvironment>,
    /// Container environment
    pub is_container: bool,
    /// WSL environment
    pub is_wsl: bool,
}

impl Environment {
    /// Detect complete environment
    pub fn detect() -> Result<Self> {
        let hardware = HardwareInfo::detect()?;
        let toolchain = ToolchainInfo::detect()?;
        let ci_environment = CiEnvironment::detect();
        let is_container = Self::detect_container();
        let is_wsl = Self::detect_wsl();
        
        Ok(Self {
            hardware,
            toolchain,
            ci_environment,
            is_container,
            is_wsl,
        })
    }
    
    /// Detect if running in a container
    fn detect_container() -> bool {
        // Check for Docker
        std::path::Path::new("/.dockerenv").exists() ||
        // Check for Kubernetes
        env::var("KUBERNETES_SERVICE_HOST").is_ok() ||
        // Check for common container indicators
        std::path::Path::new("/run/secrets/kubernetes.io").exists()
    }
    
    /// Detect if running in WSL
    fn detect_wsl() -> bool {
        if cfg!(target_os = "linux") {
            if let Ok(version) = std::fs::read_to_string("/proc/version") {
                return version.to_lowercase().contains("microsoft");
            }
        }
        false
    }
}

/// Toolchain information
#[derive(Debug, Clone)]
pub struct ToolchainInfo {
    /// Rust version
    pub rust_version: String,
    /// Cargo version
    pub cargo_version: String,
    /// Default target triple
    pub default_target: String,
    /// Available targets
    pub installed_targets: Vec<String>,
    /// Toolchain channel (stable, beta, nightly)
    pub channel: ToolchainChannel,
}

impl ToolchainInfo {
    /// Detect Rust toolchain information
    pub fn detect() -> Result<Self> {
        use std::process::Command;
        
        // Get rustc version
        let rust_version = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|e| Error::detection(format!("Failed to run rustc: {}", e)))?;
            
        let rust_version = String::from_utf8_lossy(&rust_version.stdout)
            .trim()
            .to_string();
        
        // Get cargo version
        let cargo_version = Command::new("cargo")
            .arg("--version")
            .output()
            .map_err(|e| Error::detection(format!("Failed to run cargo: {}", e)))?;
            
        let cargo_version = String::from_utf8_lossy(&cargo_version.stdout)
            .trim()
            .to_string();
        
        // Get default target
        let default_target = Command::new("rustc")
            .arg("-vV")
            .output()
            .map_err(|e| Error::detection(format!("Failed to get rustc info: {}", e)))?;
            
        let default_target = String::from_utf8_lossy(&default_target.stdout);
        let default_target = default_target
            .lines()
            .find(|line| line.starts_with("host:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("unknown")
            .to_string();
        
        // Detect channel from version string
        let channel = if rust_version.contains("nightly") {
            ToolchainChannel::Nightly
        } else if rust_version.contains("beta") {
            ToolchainChannel::Beta
        } else {
            ToolchainChannel::Stable
        };
        
        Ok(Self {
            rust_version,
            cargo_version,
            default_target: default_target.clone(),
            installed_targets: vec![default_target],
            channel,
        })
    }
    
    /// Check if a specific Rust feature is available
    pub fn has_feature(&self, feature: RustFeature) -> bool {
        use RustFeature::*;
        
        match feature {
            ParallelFrontend => self.channel == ToolchainChannel::Nightly,
            SplitDebuginfo => true, // Available in stable since 1.65
            ShareGenerics => true,   // Available in stable
            BuildStdCore => self.channel == ToolchainChannel::Nightly,
        }
    }
}

/// Toolchain channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolchainChannel {
    /// Stable channel
    Stable,
    /// Beta channel
    Beta,
    /// Nightly channel
    Nightly,
}

/// Rust compiler features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustFeature {
    /// Parallel compilation frontend
    ParallelFrontend,
    /// Split debuginfo
    SplitDebuginfo,
    /// Share generics
    ShareGenerics,
    /// Build std from source
    BuildStdCore,
}

/// CI environment detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiEnvironment {
    /// GitHub Actions
    GitHubActions,
    /// GitLab CI
    GitLabCi,
    /// Jenkins
    Jenkins,
    /// CircleCI
    CircleCi,
    /// Travis CI
    TravisCi,
    /// Azure Pipelines
    AzurePipelines,
    /// Other CI
    Other(String),
}

impl CiEnvironment {
    /// Detect CI environment
    pub fn detect() -> Option<Self> {
        if env::var("GITHUB_ACTIONS").is_ok() {
            Some(Self::GitHubActions)
        } else if env::var("GITLAB_CI").is_ok() {
            Some(Self::GitLabCi)
        } else if env::var("JENKINS_URL").is_ok() {
            Some(Self::Jenkins)
        } else if env::var("CIRCLECI").is_ok() {
            Some(Self::CircleCi)
        } else if env::var("TRAVIS").is_ok() {
            Some(Self::TravisCi)
        } else if env::var("TF_BUILD").is_ok() {
            Some(Self::AzurePipelines)
        } else if env::var("CI").is_ok() {
            Some(Self::Other("Unknown CI".to_string()))
        } else {
            None
        }
    }
    
    /// Get recommended settings for this CI environment
    pub fn recommended_settings(&self) -> CiSettings {
        match self {
            Self::GitHubActions => CiSettings {
                max_parallel_jobs: 2,
                enable_cache: true,
                cache_key_prefix: "github-actions".to_string(),
            },
            _ => CiSettings::default(),
        }
    }
}

/// CI-specific settings
#[derive(Debug, Clone)]
pub struct CiSettings {
    /// Maximum parallel jobs
    pub max_parallel_jobs: usize,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache key prefix
    pub cache_key_prefix: String,
}

impl Default for CiSettings {
    fn default() -> Self {
        Self {
            max_parallel_jobs: 2,
            enable_cache: true,
            cache_key_prefix: "ci".to_string(),
        }
    }
}
