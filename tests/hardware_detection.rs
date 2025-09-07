//! Comprehensive tests for hardware detection module.
//!
//! Tests the hardware detection functionality including:
//! - System information detection
//! - Percentage calculations
//! - Fallback mechanisms
//! - Resource checking

use cargo_optimize::hardware::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_hardware_detection_basic() {
    let hw = HardwareInfo::detect();
    
    // CPU detection should work on all platforms
    assert!(hw.cpu_count > 0, "Should detect at least 1 logical CPU");
    assert!(hw.physical_cpu_count > 0, "Should detect at least 1 physical CPU");
    assert!(
        hw.physical_cpu_count <= hw.cpu_count,
        "Physical CPUs should not exceed logical CPUs"
    );
    
    // Memory detection
    assert!(hw.total_memory > 0, "Should detect total memory");
    assert!(
        hw.available_memory <= hw.total_memory,
        "Available memory should not exceed total memory"
    );
    
    // OS detection
    assert!(!hw.os_name.is_empty(), "Should detect OS name");
    assert!(!hw.arch.is_empty(), "Should detect architecture");
    
    // Platform-specific checks
    #[cfg(target_os = "windows")]
    assert!(
        hw.os_name.to_lowercase().contains("windows"),
        "Should detect Windows on Windows platform"
    );
    
    #[cfg(target_os = "linux")]
    assert!(
        hw.os_name.to_lowercase().contains("linux"),
        "Should detect Linux on Linux platform"
    );
    
    #[cfg(target_os = "macos")]
    assert!(
        hw.os_name.to_lowercase().contains("mac"),
        "Should detect macOS on Mac platform"
    );
}

#[test]
fn test_cpu_percentage_calculations() {
    let hw = HardwareInfo {
        cpu_count: 16,
        physical_cpu_count: 8,
        total_memory: 32 * 1024 * 1024 * 1024,
        available_memory: 16 * 1024 * 1024 * 1024,
        disks: vec![],
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Test logical CPU calculations
    assert_eq!(hw.cpu_percentage(100.0, false), 16);
    assert_eq!(hw.cpu_percentage(75.0, false), 12);
    assert_eq!(hw.cpu_percentage(50.0, false), 8);
    assert_eq!(hw.cpu_percentage(25.0, false), 4);
    assert_eq!(hw.cpu_percentage(12.5, false), 2);
    
    // Test physical CPU calculations
    assert_eq!(hw.cpu_percentage(100.0, true), 8);
    assert_eq!(hw.cpu_percentage(50.0, true), 4);
    assert_eq!(hw.cpu_percentage(25.0, true), 2);
    
    // Test minimum value enforcement
    assert_eq!(hw.cpu_percentage(0.0, false), 1);
    assert_eq!(hw.cpu_percentage(0.1, false), 1);
    assert_eq!(hw.cpu_percentage(0.0, true), 1);
}

#[test]
fn test_memory_percentage_calculations() {
    let hw = HardwareInfo {
        available_memory: 8 * 1024 * 1024 * 1024, // 8 GB
        total_memory: 16 * 1024 * 1024 * 1024,
        cpu_count: 4,
        physical_cpu_count: 4,
        disks: vec![],
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    assert_eq!(hw.memory_percentage(100.0), 8 * 1024 * 1024 * 1024);
    assert_eq!(hw.memory_percentage(50.0), 4 * 1024 * 1024 * 1024);
    assert_eq!(hw.memory_percentage(25.0), 2 * 1024 * 1024 * 1024);
    assert_eq!(hw.memory_percentage(12.5), 1024 * 1024 * 1024);
    assert_eq!(hw.memory_percentage(0.0), 0);
}

#[test]
fn test_disk_percentage_calculations() {
    let hw = HardwareInfo {
        disks: vec![
            DiskInfo {
                mount_point: "/".to_string(),
                total_space: 500 * 1024 * 1024 * 1024,
                available_space: 100 * 1024 * 1024 * 1024,
                file_system: "ext4".to_string(),
                is_ssd: Some(true),
            },
            DiskInfo {
                mount_point: "/home".to_string(),
                total_space: 1000 * 1024 * 1024 * 1024,
                available_space: 200 * 1024 * 1024 * 1024,
                file_system: "ext4".to_string(),
                is_ssd: Some(false),
            },
        ],
        cpu_count: 4,
        physical_cpu_count: 4,
        total_memory: 8 * 1024 * 1024 * 1024,
        available_memory: 4 * 1024 * 1024 * 1024,
        os_name: "Linux".to_string(),
        os_version: "5.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Test root mount
    assert_eq!(
        hw.disk_percentage("/", 50.0),
        Some(50 * 1024 * 1024 * 1024)
    );
    assert_eq!(
        hw.disk_percentage("/", 100.0),
        Some(100 * 1024 * 1024 * 1024)
    );
    
    // Test /home mount
    assert_eq!(
        hw.disk_percentage("/home", 50.0),
        Some(100 * 1024 * 1024 * 1024)
    );
    assert_eq!(
        hw.disk_percentage("/home", 25.0),
        Some(50 * 1024 * 1024 * 1024)
    );
    
    // Test non-existent mount
    assert_eq!(hw.disk_percentage("/nonexistent", 50.0), None);
}

#[test]
fn test_parse_percentage_valid() {
    // With percent sign
    assert_eq!(parse_percentage("50%"), Some(50.0));
    assert_eq!(parse_percentage("0%"), Some(0.0));
    assert_eq!(parse_percentage("100%"), Some(100.0));
    assert_eq!(parse_percentage("33.33%"), Some(33.33));
    assert_eq!(parse_percentage("99.99%"), Some(99.99));
    
    // Without percent sign
    assert_eq!(parse_percentage("50"), Some(50.0));
    assert_eq!(parse_percentage("0"), Some(0.0));
    assert_eq!(parse_percentage("100"), Some(100.0));
    assert_eq!(parse_percentage("75.5"), Some(75.5));
    
    // With whitespace
    assert_eq!(parse_percentage(" 50% "), Some(50.0));
    assert_eq!(parse_percentage("  75  "), Some(75.0));
}

#[test]
fn test_parse_percentage_invalid() {
    // Out of range
    assert_eq!(parse_percentage("101%"), None);
    assert_eq!(parse_percentage("-1%"), None);
    assert_eq!(parse_percentage("150"), None);
    assert_eq!(parse_percentage("-50"), None);
    
    // Invalid format
    assert_eq!(parse_percentage("abc"), None);
    assert_eq!(parse_percentage("50%%"), None);
    assert_eq!(parse_percentage(""), None);
    assert_eq!(parse_percentage("%50"), None);
    assert_eq!(parse_percentage("fifty"), None);
}

#[test]
fn test_calculate_from_percentage_or_value() {
    // Percentage calculations
    assert_eq!(calculate_from_percentage_or_value("50%", 10), Some(5));
    assert_eq!(calculate_from_percentage_or_value("75%", 8), Some(6));
    assert_eq!(calculate_from_percentage_or_value("100%", 16), Some(16));
    assert_eq!(calculate_from_percentage_or_value("25%", 100), Some(25));
    assert_eq!(calculate_from_percentage_or_value("33.33%", 3), Some(1));
    
    // Absolute values
    assert_eq!(calculate_from_percentage_or_value("4", 10), Some(4));
    assert_eq!(calculate_from_percentage_or_value("16", 20), Some(16));
    assert_eq!(calculate_from_percentage_or_value("0", 10), Some(0));
    
    // Invalid inputs
    assert_eq!(calculate_from_percentage_or_value("abc", 10), None);
    assert_eq!(calculate_from_percentage_or_value("200%", 10), None);
    assert_eq!(calculate_from_percentage_or_value("-5", 10), None);
    assert_eq!(calculate_from_percentage_or_value("", 10), None);
}

#[test]
fn test_recommended_parallelism() {
    // Test with abundant resources
    let hw1 = HardwareInfo {
        cpu_count: 16,
        physical_cpu_count: 8,
        available_memory: 32 * 1024 * 1024 * 1024, // 32 GB
        total_memory: 64 * 1024 * 1024 * 1024,
        disks: vec![],
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Should be limited by CPU (75% of 16 = 12)
    let p1 = hw1.recommended_parallelism();
    assert!(p1 >= 1);
    assert!(p1 <= 12);
    
    // Test with limited memory
    let hw2 = HardwareInfo {
        cpu_count: 16,
        physical_cpu_count: 8,
        available_memory: 2 * 1024 * 1024 * 1024, // 2 GB
        total_memory: 4 * 1024 * 1024 * 1024,
        disks: vec![],
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Should be limited by memory (2GB / 500MB = 4)
    let p2 = hw2.recommended_parallelism();
    assert!(p2 >= 1);
    assert!(p2 <= 4);
    
    // Test with minimal resources
    let hw3 = HardwareInfo {
        cpu_count: 1,
        physical_cpu_count: 1,
        available_memory: 256 * 1024 * 1024, // 256 MB
        total_memory: 512 * 1024 * 1024,
        disks: vec![],
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Should return 1 (minimum)
    assert_eq!(hw3.recommended_parallelism(), 1);
}

#[test]
fn test_check_resources_sufficient() {
    let hw = HardwareInfo {
        available_memory: 8 * 1024 * 1024 * 1024, // 8 GB
        total_memory: 16 * 1024 * 1024 * 1024,
        disks: vec![DiskInfo {
            mount_point: if cfg!(windows) { "C:\\".to_string() } else { "/".to_string() },
            total_space: 500 * 1024 * 1024 * 1024,
            available_space: 50 * 1024 * 1024 * 1024, // 50 GB
            file_system: "ext4".to_string(),
            is_ssd: Some(true),
        }],
        cpu_count: 4,
        physical_cpu_count: 4,
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Should pass with reasonable requirements
    assert!(hw.check_resources(4.0, 20.0).is_ok());
    assert!(hw.check_resources(8.0, 50.0).is_ok());
    assert!(hw.check_resources(1.0, 10.0).is_ok());
}

#[test]
fn test_check_resources_insufficient() {
    let hw = HardwareInfo {
        available_memory: 2 * 1024 * 1024 * 1024, // 2 GB
        total_memory: 4 * 1024 * 1024 * 1024,
        disks: vec![DiskInfo {
            mount_point: if cfg!(windows) { "C:\\".to_string() } else { "/".to_string() },
            total_space: 100 * 1024 * 1024 * 1024,
            available_space: 5 * 1024 * 1024 * 1024, // 5 GB
            file_system: "ext4".to_string(),
            is_ssd: Some(false),
        }],
        cpu_count: 2,
        physical_cpu_count: 2,
        os_name: "Test OS".to_string(),
        os_version: "1.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    // Should fail with excessive memory requirement
    let result1 = hw.check_resources(4.0, 4.0);
    assert!(result1.is_err());
    assert!(result1.unwrap_err().to_string().contains("Insufficient memory"));
    
    // Should fail with excessive disk requirement
    let result2 = hw.check_resources(1.0, 10.0);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("Insufficient disk space"));
}

#[test]
fn test_fallback_hardware() {
    let hw = get_fallback_hardware();
    
    // Check default values
    assert_eq!(hw.cpu_count, 2);
    assert_eq!(hw.physical_cpu_count, 2);
    assert_eq!(hw.total_memory, 4 * 1024 * 1024 * 1024);
    assert_eq!(hw.available_memory, 2 * 1024 * 1024 * 1024);
    assert_eq!(hw.disks.len(), 1);
    
    // Check OS-specific defaults
    #[cfg(target_os = "windows")]
    {
        assert_eq!(hw.os_name, "Windows");
        assert_eq!(hw.disks[0].mount_point, "C:\\");
        assert_eq!(hw.disks[0].file_system, "NTFS");
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(hw.disks[0].mount_point, "/");
        assert_eq!(hw.disks[0].file_system, "ext4");
    }
    
    // Fallback should still be usable
    assert!(hw.recommended_parallelism() >= 1);
    assert!(hw.check_resources(1.0, 5.0).is_ok());
}

#[test]
fn test_hardware_summary_format() {
    let hw = HardwareInfo {
        cpu_count: 8,
        physical_cpu_count: 4,
        total_memory: 16 * 1024 * 1024 * 1024,
        available_memory: 8 * 1024 * 1024 * 1024,
        disks: vec![
            DiskInfo {
                mount_point: "/".to_string(),
                total_space: 500 * 1024 * 1024 * 1024,
                available_space: 100 * 1024 * 1024 * 1024,
                file_system: "ext4".to_string(),
                is_ssd: Some(true),
            },
            DiskInfo {
                mount_point: "/home".to_string(),
                total_space: 1000 * 1024 * 1024 * 1024,
                available_space: 500 * 1024 * 1024 * 1024,
                file_system: "ext4".to_string(),
                is_ssd: Some(false),
            },
        ],
        os_name: "Linux".to_string(),
        os_version: "5.15.0".to_string(),
        arch: "x86_64".to_string(),
    };
    
    let summary = hw.summary();
    
    // Check that summary contains expected information
    assert!(summary.contains("Linux"));
    assert!(summary.contains("5.15.0"));
    assert!(summary.contains("x86_64"));
    assert!(summary.contains("8 logical cores"));
    assert!(summary.contains("4 physical"));
    assert!(summary.contains("16.0 GB total"));
    assert!(summary.contains("8.0 GB available"));
    assert!(summary.contains("2 mounted"));
    
    // Test Display trait
    let display = format!("{}", hw);
    assert_eq!(display, summary);
}

#[test]
fn test_hardware_detection_stability() {
    // Run detection multiple times to ensure stability
    let hw1 = HardwareInfo::detect();
    thread::sleep(Duration::from_millis(100));
    let hw2 = HardwareInfo::detect();
    
    // Core hardware shouldn't change
    assert_eq!(hw1.cpu_count, hw2.cpu_count);
    assert_eq!(hw1.physical_cpu_count, hw2.physical_cpu_count);
    assert_eq!(hw1.os_name, hw2.os_name);
    assert_eq!(hw1.arch, hw2.arch);
    
    // Memory might change slightly but total should be constant
    assert_eq!(hw1.total_memory, hw2.total_memory);
}

#[test]
fn test_edge_cases() {
    // Test with zero CPUs (should use fallback minimum)
    let hw = HardwareInfo {
        cpu_count: 0, // Invalid but testing edge case
        physical_cpu_count: 0,
        available_memory: 1024 * 1024 * 1024,
        total_memory: 2 * 1024 * 1024 * 1024,
        disks: vec![],
        os_name: "Test".to_string(),
        os_version: "1.0".to_string(),
        arch: "test".to_string(),
    };
    
    // Should still return minimum of 1
    assert_eq!(hw.cpu_percentage(100.0, false), 1);
    assert_eq!(hw.recommended_parallelism(), 1);
    
    // Test with very large percentages (should be capped at input validation)
    assert_eq!(parse_percentage("1000%"), None);
    assert_eq!(parse_percentage("-100%"), None);
    
    // Test empty disk list
    assert_eq!(hw.disk_percentage("/", 50.0), None);
}

/// Test that demonstrates usage patterns for the hardware module.
#[test]
fn test_usage_examples() {
    let hw = HardwareInfo::detect();
    
    // Example 1: Get 75% of available CPUs for build jobs
    let build_jobs = hw.cpu_percentage(75.0, false);
    assert!(build_jobs >= 1);
    println!("Using {} build jobs (75% of {} cores)", build_jobs, hw.cpu_count);
    
    // Example 2: Allocate cache based on available memory
    let cache_size = hw.memory_percentage(25.0); // Use 25% of available memory for cache
    println!("Cache size: {} bytes", cache_size);
    
    // Example 3: Check if we have enough resources for optimization
    if hw.check_resources(2.0, 5.0).is_ok() {
        println!("Sufficient resources for optimization");
    }
    
    // Example 4: Parse user input for CPU count
    let user_input = "50%";
    if let Some(cores) = calculate_from_percentage_or_value(user_input, hw.cpu_count) {
        println!("User requested {} cores", cores);
    }
    
    // Example 5: Get conservative parallelism recommendation
    let parallelism = hw.recommended_parallelism();
    println!("Recommended parallelism: {}", parallelism);
}
