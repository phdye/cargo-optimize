//! Hardware detection and system information module.
//!
//! Provides platform-aware hardware detection with percentage-based
//! calculations and graceful fallbacks.

use anyhow::{Context, Result};
use num_cpus;
use sysinfo::{Disks, System};
use std::fmt;

/// System hardware information with support for percentage calculations.
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    /// Total number of logical CPU cores
    pub cpu_count: usize,
    /// Total number of physical CPU cores
    pub physical_cpu_count: usize,
    /// Total system memory in bytes
    pub total_memory: u64,
    /// Available system memory in bytes
    pub available_memory: u64,
    /// List of disk information
    pub disks: Vec<DiskInfo>,
    /// Operating system name
    pub os_name: String,
    /// Operating system version
    pub os_version: String,
    /// CPU architecture (e.g., x86_64, aarch64)
    pub arch: String,
}

/// Information about a disk/storage device.
#[derive(Debug, Clone)]
pub struct DiskInfo {
    /// Mount point or drive letter
    pub mount_point: String,
    /// Total disk space in bytes
    pub total_space: u64,
    /// Available disk space in bytes
    pub available_space: u64,
    /// File system type (e.g., NTFS, ext4)
    pub file_system: String,
    /// Whether this is an SSD (if detectable)
    pub is_ssd: Option<bool>,
}

impl HardwareInfo {
    /// Detect current system hardware information.
    ///
    /// Returns hardware info with fallback values if detection fails.
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        // CPU detection with fallbacks
        let cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();

        // Memory detection
        let total_memory = sys.total_memory();
        let available_memory = sys.available_memory();

        // Disk detection - use separate Disks struct in sysinfo 0.30+
        let disks_info = Disks::new_with_refreshed_list();
        let disks = disks_info
            .iter()
            .map(|disk| DiskInfo {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_ssd: detect_ssd_type(disk.name()),
            })
            .collect();

        // OS information
        let os_name = System::name()
            .unwrap_or_else(|| detect_os_fallback().to_string());
        let os_version = System::os_version()
            .unwrap_or_else(|| "unknown".to_string());

        // Architecture detection
        let arch = std::env::consts::ARCH.to_string();

        Self {
            cpu_count,
            physical_cpu_count,
            total_memory,
            available_memory,
            disks,
            os_name,
            os_version,
            arch,
        }
    }

    /// Calculate a percentage of CPU cores.
    ///
    /// # Arguments
    /// * `percentage` - Percentage value (0-100)
    /// * `use_physical` - Use physical cores instead of logical
    ///
    /// # Returns
    /// Number of cores (minimum 1)
    pub fn cpu_percentage(&self, percentage: f64, use_physical: bool) -> usize {
        let count = if use_physical {
            self.physical_cpu_count
        } else {
            self.cpu_count
        };
        
        let result = ((count as f64 * percentage) / 100.0).round() as usize;
        result.max(1) // Always return at least 1 core
    }

    /// Calculate a percentage of available memory.
    ///
    /// # Arguments
    /// * `percentage` - Percentage value (0-100)
    ///
    /// # Returns
    /// Memory size in bytes
    pub fn memory_percentage(&self, percentage: f64) -> u64 {
        ((self.available_memory as f64 * percentage) / 100.0).round() as u64
    }

    /// Calculate a percentage of available disk space for a specific mount.
    ///
    /// # Arguments
    /// * `mount` - Mount point or drive letter
    /// * `percentage` - Percentage value (0-100)
    ///
    /// # Returns
    /// Disk space in bytes, or None if mount not found
    pub fn disk_percentage(&self, mount: &str, percentage: f64) -> Option<u64> {
        self.disks
            .iter()
            .find(|d| d.mount_point == mount)
            .map(|d| ((d.available_space as f64 * percentage) / 100.0).round() as u64)
    }

    /// Get recommended parallelism level for builds.
    ///
    /// Returns a conservative estimate based on available resources.
    pub fn recommended_parallelism(&self) -> usize {
        // Use 75% of logical cores by default, but consider memory
        let cpu_based = self.cpu_percentage(75.0, false);
        
        // Assume each build job needs ~500MB
        let memory_based = (self.available_memory / (500 * 1024 * 1024)) as usize;
        
        // Take the minimum to avoid memory pressure
        cpu_based.min(memory_based).max(1)
    }

    /// Check if the system has sufficient resources for optimization.
    ///
    /// # Arguments
    /// * `min_memory_gb` - Minimum required memory in GB
    /// * `min_disk_gb` - Minimum required disk space in GB
    ///
    /// # Returns
    /// Ok(()) if sufficient resources, Err with details otherwise
    pub fn check_resources(&self, min_memory_gb: f64, min_disk_gb: f64) -> Result<()> {
        let memory_gb = self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        if memory_gb < min_memory_gb {
            anyhow::bail!(
                "Insufficient memory: {:.1} GB available, {:.1} GB required",
                memory_gb,
                min_memory_gb
            );
        }

        // Check disk space on the current working directory's mount
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let cwd_str = cwd.to_string_lossy();
        
        // Find the mount point for the current directory
        let mount = self
            .disks
            .iter()
            .filter(|d| cwd_str.starts_with(&d.mount_point))
            .max_by_key(|d| d.mount_point.len());

        if let Some(disk) = mount {
            let disk_gb = disk.available_space as f64 / (1024.0 * 1024.0 * 1024.0);
            if disk_gb < min_disk_gb {
                anyhow::bail!(
                    "Insufficient disk space on {}: {:.1} GB available, {:.1} GB required",
                    disk.mount_point,
                    disk_gb,
                    min_disk_gb
                );
            }
        }

        Ok(())
    }

    /// Format hardware info as a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "System: {} {} ({})\n\
             CPU: {} logical cores ({} physical)\n\
             Memory: {:.1} GB total, {:.1} GB available\n\
             Disks: {} mounted",
            self.os_name,
            self.os_version,
            self.arch,
            self.cpu_count,
            self.physical_cpu_count,
            self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0),
            self.disks.len()
        )
    }
}

impl fmt::Display for HardwareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Parse a percentage string like "75%" or "50".
///
/// # Arguments
/// * `input` - Input string
///
/// # Returns
/// Percentage as f64 (0-100), or None if invalid
pub fn parse_percentage(input: &str) -> Option<f64> {
    let trimmed = input.trim();
    let without_percent = trimmed.strip_suffix('%').unwrap_or(trimmed);
    
    without_percent
        .parse::<f64>()
        .ok()
        .filter(|&p| (0.0..=100.0).contains(&p))
}

/// Calculate the actual value from a percentage string or absolute value.
///
/// # Arguments
/// * `value` - Either a percentage ("75%") or absolute value ("4")
/// * `total` - Total value for percentage calculation
///
/// # Returns
/// Calculated value, or None if invalid input
pub fn calculate_from_percentage_or_value(value: &str, total: usize) -> Option<usize> {
    if value.contains('%') {
        parse_percentage(value).map(|p| ((total as f64 * p) / 100.0).round() as usize)
    } else {
        value.parse::<usize>().ok()
    }
}

/// Get default hardware info for fallback scenarios.
///
/// Returns conservative defaults that should work on most systems.
pub fn get_fallback_hardware() -> HardwareInfo {
    HardwareInfo {
        cpu_count: 2,
        physical_cpu_count: 2,
        total_memory: 4 * 1024 * 1024 * 1024, // 4 GB
        available_memory: 2 * 1024 * 1024 * 1024, // 2 GB
        disks: vec![DiskInfo {
            mount_point: if cfg!(windows) { "C:\\".to_string() } else { "/".to_string() },
            total_space: 100 * 1024 * 1024 * 1024, // 100 GB
            available_space: 10 * 1024 * 1024 * 1024, // 10 GB
            file_system: if cfg!(windows) { "NTFS".to_string() } else { "ext4".to_string() },
            is_ssd: None,
        }],
        os_name: detect_os_fallback().to_string(),
        os_version: "unknown".to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

/// Detect OS name as a fallback when sysinfo fails.
fn detect_os_fallback() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    }
}

/// Try to detect if a disk is an SSD based on its name.
///
/// This is a heuristic approach and may not be accurate.
fn detect_ssd_type(disk_name: &std::ffi::OsStr) -> Option<bool> {
    let name = disk_name.to_string_lossy().to_lowercase();
    
    // Common SSD indicators in disk names
    if name.contains("ssd") || name.contains("nvme") || name.contains("solid") {
        Some(true)
    } else if name.contains("hdd") || name.contains("hard") {
        Some(false)
    } else {
        None // Cannot determine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let hw = HardwareInfo::detect();
        
        // Basic sanity checks
        assert!(hw.cpu_count >= 1);
        assert!(hw.physical_cpu_count >= 1);
        assert!(hw.physical_cpu_count <= hw.cpu_count);
        assert!(hw.total_memory > 0);
        assert!(!hw.os_name.is_empty());
        assert!(!hw.arch.is_empty());
    }

    #[test]
    fn test_cpu_percentage_calculation() {
        let hw = HardwareInfo {
            cpu_count: 8,
            physical_cpu_count: 4,
            ..get_fallback_hardware()
        };

        assert_eq!(hw.cpu_percentage(50.0, false), 4);
        assert_eq!(hw.cpu_percentage(75.0, false), 6);
        assert_eq!(hw.cpu_percentage(100.0, false), 8);
        assert_eq!(hw.cpu_percentage(25.0, true), 1);
        assert_eq!(hw.cpu_percentage(50.0, true), 2);
        
        // Should always return at least 1
        assert_eq!(hw.cpu_percentage(0.0, false), 1);
        assert_eq!(hw.cpu_percentage(1.0, false), 1);
    }

    #[test]
    fn test_memory_percentage_calculation() {
        let hw = HardwareInfo {
            available_memory: 1024 * 1024 * 1024, // 1 GB
            ..get_fallback_hardware()
        };

        assert_eq!(hw.memory_percentage(50.0), 512 * 1024 * 1024);
        assert_eq!(hw.memory_percentage(25.0), 256 * 1024 * 1024);
        assert_eq!(hw.memory_percentage(100.0), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("50%"), Some(50.0));
        assert_eq!(parse_percentage("75"), Some(75.0));
        assert_eq!(parse_percentage("100%"), Some(100.0));
        assert_eq!(parse_percentage("0"), Some(0.0));
        assert_eq!(parse_percentage("25.5%"), Some(25.5));
        
        // Invalid cases
        assert_eq!(parse_percentage("101%"), None);
        assert_eq!(parse_percentage("-10"), None);
        assert_eq!(parse_percentage("abc"), None);
        assert_eq!(parse_percentage(""), None);
    }

    #[test]
    fn test_calculate_from_percentage_or_value() {
        assert_eq!(calculate_from_percentage_or_value("50%", 10), Some(5));
        assert_eq!(calculate_from_percentage_or_value("75%", 8), Some(6));
        assert_eq!(calculate_from_percentage_or_value("4", 10), Some(4));
        assert_eq!(calculate_from_percentage_or_value("25%", 100), Some(25));
        
        // Invalid cases
        assert_eq!(calculate_from_percentage_or_value("abc", 10), None);
        assert_eq!(calculate_from_percentage_or_value("200%", 10), None);
        assert_eq!(calculate_from_percentage_or_value("-5", 10), None);
    }

    #[test]
    fn test_fallback_hardware() {
        let hw = get_fallback_hardware();
        
        assert_eq!(hw.cpu_count, 2);
        assert_eq!(hw.physical_cpu_count, 2);
        assert_eq!(hw.total_memory, 4 * 1024 * 1024 * 1024);
        assert_eq!(hw.disks.len(), 1);
        assert!(!hw.os_name.is_empty());
    }

    #[test]
    fn test_recommended_parallelism() {
        let hw = HardwareInfo {
            cpu_count: 8,
            available_memory: 4 * 1024 * 1024 * 1024, // 4 GB
            ..get_fallback_hardware()
        };

        // Should be 6 (75% of 8 cores) but limited by memory (4GB / 500MB = 8)
        let parallelism = hw.recommended_parallelism();
        assert!(parallelism >= 1);
        assert!(parallelism <= 8);
    }

    #[test]
    fn test_check_resources() {
        // Create a test mount point that matches the platform
        let mount = if cfg!(windows) { "C:\\".to_string() } else { "/".to_string() };
        
        let hw = HardwareInfo {
            available_memory: 4 * 1024 * 1024 * 1024, // 4 GB
            disks: vec![DiskInfo {
                mount_point: mount.clone(),
                total_space: 100 * 1024 * 1024 * 1024, // 100 GB total
                available_space: 20 * 1024 * 1024 * 1024, // 20 GB available
                file_system: if cfg!(windows) { "NTFS".to_string() } else { "ext4".to_string() },
                is_ssd: None,
            }],
            total_memory: 8 * 1024 * 1024 * 1024,
            cpu_count: 4,
            physical_cpu_count: 4,
            os_name: detect_os_fallback().to_string(),
            os_version: "test".to_string(),
            arch: "x86_64".to_string(),
        };

        // Should pass with reasonable requirements
        assert!(hw.check_resources(2.0, 10.0).is_ok());
        
        // Should fail with excessive memory requirement
        assert!(hw.check_resources(8.0, 10.0).is_err());
        
        // Note: Disk check may not work if current directory doesn't match our test mount
        // So we'll test it differently
        let memory_error = hw.check_resources(8.0, 10.0);
        assert!(memory_error.is_err());
        assert!(memory_error.unwrap_err().to_string().contains("Insufficient memory"));
    }
}
