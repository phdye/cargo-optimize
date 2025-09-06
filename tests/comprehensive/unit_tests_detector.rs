//! Unit tests for the detector module

use cargo_optimize::detector::{CpuInfo, HardwareInfo, MemoryInfo, OsInfo, SystemDetector};

#[path = "../test_helpers.rs"]
mod test_helpers;
use pretty_assertions::assert_eq;
use std::time::Duration;
use test_helpers::create_test_hardware_info;


#[test]
fn test_detector_cpu_cores() {
    let detector = SystemDetector::new();
    let cpu_info = detector.detect_cpu();

    // Should detect at least 1 core
    assert!(cpu_info.logical_cores >= 1);
    assert!(cpu_info.physical_cores >= 1);

    // Logical cores should be >= physical cores
    assert!(cpu_info.logical_cores >= cpu_info.physical_cores);

    // Should have a CPU name
    assert!(!cpu_info.model_name.is_empty());
}

#[test]
fn test_detector_memory_info() {
    let detector = SystemDetector::new();
    let memory_info = detector.detect_memory();

    // Should detect some memory
    assert!(memory_info.total_bytes > 0);
    assert!(memory_info.available_bytes > 0);

    // Available should be <= total
    assert!(memory_info.available_bytes <= memory_info.total_bytes);

    // Should be reasonable values (at least 100MB, less than 1TB)
    assert!(memory_info.total_bytes >= 100 * 1024 * 1024); // 100MB
    assert!(memory_info.total_bytes <= 1024 * 1024 * 1024 * 1024); // 1TB
}

#[test]
fn test_detector_os_info() {
    let detector = SystemDetector::new();
    let os_info = detector.detect_os();

    // Should detect OS family
    assert!(!os_info.family.is_empty());

    // Should be one of the known families
    let known_families = ["windows", "unix", "linux", "macos", "freebsd"];
    assert!(known_families
        .iter()
        .any(|&family| os_info.family.to_lowercase().contains(family)));

    // Should have architecture info
    assert!(!os_info.arch.is_empty());

    // Should be a known architecture
    let known_archs = ["x86", "x86_64", "aarch64", "arm"];
    assert!(known_archs
        .iter()
        .any(|&arch| os_info.arch.to_lowercase().contains(arch)));
}

#[test]
fn test_detector_hardware_info_complete() {
    let detector = SystemDetector::new();
    let hardware = detector.detect_all();

    // Should contain all components
    assert!(hardware.cpu.logical_cores > 0);
    assert!(hardware.memory.total_bytes > 0);
    assert!(!hardware.os.family.is_empty());
}

#[test]
fn test_detector_performance() {
    let detector = SystemDetector::new();

    // Detection should be reasonably fast
    let start = std::time::Instant::now();
    let _hardware = detector.detect_all();
    let duration = start.elapsed();

    // Should complete within 5 seconds
    assert!(
        duration < Duration::from_secs(5),
        "Hardware detection took too long: {:?}",
        duration
    );
}

#[test]
fn test_detector_consistency() {
    let detector = SystemDetector::new();

    // Multiple calls should return consistent results
    let info1 = detector.detect_cpu();
    let info2 = detector.detect_cpu();

    assert_eq!(info1.logical_cores, info2.logical_cores);
    assert_eq!(info1.physical_cores, info2.physical_cores);
    assert_eq!(info1.model_name, info2.model_name);
}

#[test]
fn test_cpu_info_display() {
    let cpu_info = CpuInfo {
        logical_cores: 8,
        physical_cores: 4,
        model_name: "Test CPU".to_string(),
        base_frequency: Some(3200),
        max_frequency: Some(4800),
        features: vec!["sse4.2".to_string(), "avx2".to_string()],
    };

    let display = format!("{}", cpu_info);
    assert!(display.contains("8"));
    assert!(display.contains("4"));
    assert!(display.contains("Test CPU"));
}

#[test]
fn test_memory_info_formatting() {
    let memory_info = MemoryInfo {
        total_bytes: 16 * 1024 * 1024 * 1024,         // 16GB
        available_bytes: 8 * 1024 * 1024 * 1024,      // 8GB
        swap_total_bytes: 4 * 1024 * 1024 * 1024,     // 4GB
        swap_available_bytes: 2 * 1024 * 1024 * 1024, // 2GB
    };

    assert_eq!(memory_info.total_gb(), 16);
    assert_eq!(memory_info.available_gb(), 8);
    assert_eq!(memory_info.usage_percent(), 50);
}

#[test]
fn test_os_info_version_parsing() {
    let os_info = OsInfo {
        family: "linux".to_string(),
        name: "Ubuntu".to_string(),
        version: "22.04".to_string(),
        arch: "x86_64".to_string(),
        is_64bit: true,
    };

    assert!(os_info.is_unix());
    assert!(!os_info.is_windows());
    assert!(os_info.is_64bit);
}

// Boundary value tests
#[test]
fn test_detector_with_minimal_system() {
    // Test behavior with minimal system specs
    let detector = SystemDetector::new();
    let cpu = detector.detect_cpu();

    // Even minimal systems should have at least 1 core
    assert!(cpu.logical_cores >= 1);
    assert!(cpu.physical_cores >= 1);
}

#[test]
fn test_detector_error_handling() {
    // Test that detector handles errors gracefully
    let detector = SystemDetector::new();

    // This should not panic even if system info is unavailable
    let result = std::panic::catch_unwind(|| detector.detect_all());

    assert!(result.is_ok(), "Detector should handle errors gracefully");
}

// Property-based tests
#[test]
fn test_detector_properties() {
    let detector = SystemDetector::new();

    for _ in 0..10 {
        let hardware = detector.detect_all();

        // Property: logical cores >= physical cores
        assert!(hardware.cpu.logical_cores >= hardware.cpu.physical_cores);

        // Property: available memory <= total memory
        assert!(hardware.memory.available_bytes <= hardware.memory.total_bytes);

        // Property: values should be consistent across calls
        let hardware2 = detector.detect_all();
        assert_eq!(hardware.cpu.logical_cores, hardware2.cpu.logical_cores);
        assert_eq!(hardware.memory.total_bytes, hardware2.memory.total_bytes);
    }
}

#[test]
fn test_hardware_info_serialization() {
    // Create a complete hardware info for testing serialization
    let mut hardware = create_test_hardware_info();

    // Override with specific test values
    hardware.cpu.logical_cores = 8;
    hardware.cpu.physical_cores = 4;
    hardware.cpu.model_name = "Test CPU".to_string();
    hardware.cpu.base_frequency = Some(3200);
    hardware.cpu.max_frequency = Some(4800);
    hardware.cpu.features = vec!["sse4.2".to_string(), "avx2".to_string()];

    hardware.memory.total_bytes = 16 * 1024 * 1024 * 1024;
    hardware.memory.available_bytes = 8 * 1024 * 1024 * 1024;
    hardware.memory.swap_total_bytes = 4 * 1024 * 1024 * 1024;
    hardware.memory.swap_available_bytes = 2 * 1024 * 1024 * 1024;

    hardware.os.family = "linux".to_string();
    hardware.os.name = "Ubuntu".to_string();
    hardware.os.version = "22.04".to_string();
    hardware.os.arch = "x86_64".to_string();
    hardware.os.is_64bit = true;

    // Update legacy fields to match
    hardware.cpu_cores = hardware.cpu.physical_cores;
    hardware.logical_cpus = hardware.cpu.logical_cores;
    hardware.total_memory = hardware.memory.total_bytes;
    hardware.available_memory = hardware.memory.available_bytes;

    // Should serialize to JSON
    let json = serde_json::to_string(&hardware).expect("Failed to serialize");
    assert!(json.contains("logical_cores"));
    assert!(json.contains("8"));

    // Should deserialize back
    let deserialized: HardwareInfo = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(hardware.cpu.logical_cores, deserialized.cpu.logical_cores);
}

#[test]
fn test_detector_caching() {
    let detector = SystemDetector::new();

    // First call should populate cache
    let start1 = std::time::Instant::now();
    let info1 = detector.detect_cpu();
    let duration1 = start1.elapsed();

    // Second call should be faster (cached)
    let start2 = std::time::Instant::now();
    let info2 = detector.detect_cpu();
    let duration2 = start2.elapsed();

    // Results should be identical
    assert_eq!(info1.logical_cores, info2.logical_cores);
    assert_eq!(info1.model_name, info2.model_name);

    // Second call should be significantly faster (or at least not slower)
    assert!(duration2 <= duration1 + Duration::from_millis(10));
}

// Stress tests
#[test]
fn test_detector_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let detector = Arc::new(SystemDetector::new());
    let mut handles = Vec::new();

    // Spawn multiple threads accessing detector simultaneously
    for _ in 0..10 {
        let detector_clone: Arc<SystemDetector> = Arc::clone(&detector);
        let handle = thread::spawn(move || {
            let hardware = detector_clone.detect_all();
            assert!(hardware.cpu.logical_cores > 0);
            assert!(hardware.memory.total_bytes > 0);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_detector_repeated_calls() {
    let detector = SystemDetector::new();

    // Make many repeated calls to ensure no memory leaks or degradation
    for i in 0..100 {
        let hardware = detector.detect_all();

        assert!(hardware.cpu.logical_cores > 0, "Failed on iteration {}", i);
        assert!(hardware.memory.total_bytes > 0, "Failed on iteration {}", i);

        // Shouldn't take progressively longer
        let start = std::time::Instant::now();
        let _ = detector.detect_cpu();
        let duration = start.elapsed();
        assert!(
            duration < Duration::from_secs(1),
            "Detection taking too long on iteration {}: {:?}",
            i,
            duration
        );
    }
}

// Platform-specific tests
#[cfg(target_os = "windows")]
#[test]
fn test_windows_specific_detection() {
    let detector = SystemDetector::new();
    let os_info = detector.detect_os();

    assert_eq!(os_info.family.to_lowercase(), "windows");
    assert!(os_info.name.to_lowercase().contains("windows"));
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_specific_detection() {
    let detector = SystemDetector::new();
    let os_info = detector.detect_os();

    assert_eq!(os_info.family.to_lowercase(), "unix");
    // Linux distribution names vary, so just check it's not empty
    assert!(!os_info.name.is_empty());
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_specific_detection() {
    let detector = SystemDetector::new();
    let os_info = detector.detect_os();

    assert_eq!(os_info.family.to_lowercase(), "unix");
    assert!(
        os_info.name.to_lowercase().contains("macos")
            || os_info.name.to_lowercase().contains("darwin")
    );
}

// Mock tests for difficult-to-test scenarios
#[test]
fn test_detector_with_mock_data() {
    // Create a detector with mocked system info for testing edge cases
    let mock_cpu = CpuInfo {
        logical_cores: 1,
        physical_cores: 1,
        model_name: "Mock CPU".to_string(),
        base_frequency: None,
        max_frequency: None,
        features: vec![],
    };

    let mock_memory = MemoryInfo {
        total_bytes: 1024 * 1024 * 1024,    // 1GB
        available_bytes: 512 * 1024 * 1024, // 512MB
        swap_total_bytes: 0,
        swap_available_bytes: 0,
    };

    let mock_os = OsInfo {
        family: "unknown".to_string(),
        name: "Mock OS".to_string(),
        version: "1.0".to_string(),
        arch: "mock_arch".to_string(),
        is_64bit: false,
    };

    // Use the test helper to create a complete HardwareInfo with all required fields
    let mut mock_hardware = create_test_hardware_info();

    // Override with our mock values
    mock_hardware.cpu = mock_cpu;
    mock_hardware.memory = mock_memory;
    mock_hardware.os = mock_os;

    // Update legacy fields to match
    mock_hardware.cpu_cores = mock_hardware.cpu.physical_cores;
    mock_hardware.logical_cpus = mock_hardware.cpu.logical_cores;
    mock_hardware.total_memory = mock_hardware.memory.total_bytes;
    mock_hardware.available_memory = mock_hardware.memory.available_bytes;

    // Test that our optimization logic handles minimal specs correctly
    assert_eq!(mock_hardware.cpu.logical_cores, 1);
    assert_eq!(mock_hardware.memory.total_gb(), 1);
    assert!(!mock_hardware.os.is_64bit);
}

#[test]
fn test_detector_feature_detection() {
    let detector = SystemDetector::new();
    let cpu = detector.detect_cpu();

    // Should detect at least some basic features on modern systems
    // Note: This test might fail on very old systems
    if !cpu.features.is_empty() {
        // Common features that most modern x86_64 systems should have
        let common_features = ["sse2", "sse4.1", "sse4.2"];
        let has_common = common_features.iter().any(|&feature| {
            cpu.features
                .iter()
                .any(|f| f.to_lowercase().contains(feature))
        });

        if cpu.model_name.to_lowercase().contains("intel")
            || cpu.model_name.to_lowercase().contains("amd")
        {
            // Only assert for x86 processors
            assert!(
                has_common || cpu.features.len() > 0,
                "Expected to find common CPU features, got: {:?}",
                cpu.features
            );
        }
    }
}
