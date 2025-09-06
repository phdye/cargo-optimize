//! Test helper functions

/// Helper to create test hardware info with all required fields
pub fn create_test_hardware_info() -> cargo_optimize::detector::HardwareInfo {
    use cargo_optimize::detector::*;

    let cpu_info = CpuInfo {
        logical_cores: 4,
        physical_cores: 2,
        model_name: "Test CPU".to_string(),
        base_frequency: Some(2400),
        max_frequency: Some(3600),
        features: vec!["sse2".to_string(), "avx".to_string()],
    };

    let memory_info = MemoryInfo {
        total_bytes: 8 * 1024 * 1024 * 1024,          // 8GB
        available_bytes: 4 * 1024 * 1024 * 1024,      // 4GB
        swap_total_bytes: 2 * 1024 * 1024 * 1024,     // 2GB
        swap_available_bytes: 1 * 1024 * 1024 * 1024, // 1GB
    };

    let os_info = OsInfo {
        family: "linux".to_string(),
        name: "Test OS".to_string(),
        version: "1.0".to_string(),
        arch: "x86_64".to_string(),
        is_64bit: true,
    };

    HardwareInfo {
        // New format
        cpu: cpu_info.clone(),
        memory: memory_info.clone(),
        os: os_info.clone(),

        // Legacy format (for backward compatibility)
        cpu_cores: cpu_info.physical_cores,
        logical_cpus: cpu_info.logical_cores,
        total_memory: memory_info.total_bytes,
        available_memory: memory_info.available_bytes,
        cpu_arch: CpuArchitecture::X86_64,
        operating_system: OperatingSystem::Linux,
        cpu_brand: cpu_info.model_name,
        cpu_frequency: cpu_info.base_frequency.unwrap_or(0),
    }
}
