//! Baseline Performance Measurement Utilities
//! Used to establish performance baselines for regression testing

use std::time::{Duration, Instant};
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BaselineMetrics {
    pub linker_detection_time: Duration,
    pub config_generation_time: Duration,
    pub file_operations_time: Duration,
    pub memory_usage_kb: u64,
    pub disk_usage_kb: u64,
}

impl Default for BaselineMetrics {
    fn default() -> Self {
        Self {
            linker_detection_time: Duration::from_millis(50),  // Target: <100ms
            config_generation_time: Duration::from_millis(10), // Target: <50ms
            file_operations_time: Duration::from_millis(20),   // Target: <100ms
            memory_usage_kb: 1024,  // Target: <5MB
            disk_usage_kb: 512,     // Target: <2MB
        }
    }
}

pub fn measure_linker_detection() -> Duration {
    let start = Instant::now();
    
    // Simulate the actual linker detection process
    let _ = cargo_optimize::mvp::detect_best_linker();
    
    start.elapsed()
}

pub fn measure_config_generation() -> Duration {
    let start = Instant::now();
    
    // Measure config file generation
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join(".cargo").join("config.toml");
    
    // Create the .cargo directory
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    
    // Measure generation time
    let _ = cargo_optimize::mvp::create_optimized_config(&config_path);
    
    start.elapsed()
}

pub fn measure_memory_usage() -> u64 {
    // Simple memory measurement using process stats
    // This is a simplified version - in production we'd use more sophisticated tools
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output();
    
    match output {
        Ok(output) => {
            let rss_str = String::from_utf8_lossy(&output.stdout);
            rss_str.trim().parse().unwrap_or(0)
        }
        Err(_) => {
            // Fallback for Windows - use a reasonable estimate
            2048 // 2MB baseline
        }
    }
}

pub fn establish_baseline() -> BaselineMetrics {
    println!("📊 Establishing performance baselines...");
    
    // Warm up the system
    for _ in 0..3 {
        let _ = measure_linker_detection();
        let _ = measure_config_generation();
    }
    
    // Take multiple measurements and average
    let mut linker_times = Vec::new();
    let mut config_times = Vec::new();
    
    for i in 0..10 {
        println!("  Measurement {}/10", i + 1);
        linker_times.push(measure_linker_detection());
        config_times.push(measure_config_generation());
        
        // Small delay between measurements
        std::thread::sleep(Duration::from_millis(10));
    }
    
    // Calculate averages
    let avg_linker = linker_times.iter().sum::<Duration>() / linker_times.len() as u32;
    let avg_config = config_times.iter().sum::<Duration>() / config_times.len() as u32;
    
    let memory = measure_memory_usage();
    
    let baseline = BaselineMetrics {
        linker_detection_time: avg_linker,
        config_generation_time: avg_config,
        file_operations_time: avg_linker + avg_config, // Combined file ops
        memory_usage_kb: memory,
        disk_usage_kb: 512, // Estimate for config files
    };
    
    println!("✅ Baseline established:");
    println!("   Linker detection: {:?}", baseline.linker_detection_time);
    println!("   Config generation: {:?}", baseline.config_generation_time);
    println!("   Memory usage: {} KB", baseline.memory_usage_kb);
    
    baseline
}

pub fn save_baseline(baseline: &BaselineMetrics, path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(baseline)?;
    fs::write(path, json)?;
    println!("💾 Baseline saved to: {}", path.display());
    Ok(())
}

pub fn load_baseline(path: &Path) -> std::io::Result<BaselineMetrics> {
    let json = fs::read_to_string(path)?;
    let baseline = serde_json::from_str(&json)?;
    Ok(baseline)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_baseline_measurement() {
        let baseline = establish_baseline();
        
        // Verify measurements are reasonable
        assert!(baseline.linker_detection_time < Duration::from_secs(1));
        assert!(baseline.config_generation_time < Duration::from_millis(500));
        assert!(baseline.memory_usage_kb > 0);
    }
    
    #[test]
    fn test_baseline_persistence() {
        let baseline = BaselineMetrics::default();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        // Test save/load cycle
        save_baseline(&baseline, temp_file.path()).unwrap();
        let loaded = load_baseline(temp_file.path()).unwrap();
        
        assert_eq!(baseline.linker_detection_time, loaded.linker_detection_time);
        assert_eq!(baseline.config_generation_time, loaded.config_generation_time);
    }
}
