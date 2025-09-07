//! Example demonstrating hardware detection capabilities.
//!
//! Run with: `cargo run --example hardware_info`

use cargo_optimize::hardware::{self, HardwareInfo};

fn main() {
    println!("=== Hardware Detection Example ===\n");
    
    // Detect current system hardware
    let hw = HardwareInfo::detect();
    
    // Display summary
    println!("System Information:");
    println!("{}", hw);
    println!();
    
    // Demonstrate percentage calculations
    println!("Resource Allocation Examples:");
    println!("  75% of logical CPUs: {} cores", hw.cpu_percentage(75.0, false));
    println!("  50% of physical CPUs: {} cores", hw.cpu_percentage(50.0, true));
    println!("  25% of available memory: {:.2} GB", 
        hw.memory_percentage(25.0) as f64 / (1024.0 * 1024.0 * 1024.0));
    println!();
    
    // Show recommended parallelism
    println!("Recommended build parallelism: {} jobs", hw.recommended_parallelism());
    println!("  (Based on CPU and memory constraints)");
    println!();
    
    // Parse user input examples
    println!("Parsing user input:");
    let inputs = vec!["50%", "75%", "4", "100%"];
    for input in inputs {
        if let Some(value) = hardware::parse_percentage(input) {
            println!("  '{}' -> {:.1}%", input, value);
        }
        if let Some(cores) = hardware::calculate_from_percentage_or_value(input, hw.cpu_count) {
            println!("    = {} cores (out of {})", cores, hw.cpu_count);
        }
    }
    println!();
    
    // Check resource availability
    println!("Resource checks:");
    let checks = vec![
        (1.0, 5.0, "Minimal build"),
        (2.0, 10.0, "Standard build"),
        (4.0, 20.0, "Large project"),
        (8.0, 50.0, "Enterprise build"),
    ];
    
    for (mem_gb, disk_gb, desc) in checks {
        match hw.check_resources(mem_gb, disk_gb) {
            Ok(()) => println!("  ✓ {} ({}GB RAM, {}GB disk): OK", desc, mem_gb, disk_gb),
            Err(e) => println!("  ✗ {} ({}GB RAM, {}GB disk): {}", desc, mem_gb, disk_gb, e),
        }
    }
    println!();
    
    // Show disk information
    if !hw.disks.is_empty() {
        println!("Disk Information:");
        for disk in &hw.disks {
            println!("  Mount: {}", disk.mount_point);
            println!("    File System: {}", disk.file_system);
            println!("    Available: {:.1} GB of {:.1} GB",
                disk.available_space as f64 / (1024.0 * 1024.0 * 1024.0),
                disk.total_space as f64 / (1024.0 * 1024.0 * 1024.0));
            if let Some(is_ssd) = disk.is_ssd {
                println!("    SSD: {}", if is_ssd { "Yes" } else { "No" });
            }
        }
        println!();
    }
    
    // Demonstrate fallback
    println!("Fallback hardware (for testing):");
    let fallback = hardware::get_fallback_hardware();
    println!("  CPUs: {}", fallback.cpu_count);
    println!("  Memory: {:.1} GB", fallback.total_memory as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("  OS: {}", fallback.os_name);
}
