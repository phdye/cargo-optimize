//! Example: Basic usage of cargo-optimize

use cargo_optimize::{self, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Basic cargo-optimize usage example\n");

    // Method 1: Auto-configure with defaults
    println!("Method 1: Auto-configure");
    println!("------------------------");
    cargo_optimize::auto_configure();

    // Method 2: Configure with custom settings
    println!("\nMethod 2: Custom configuration");
    println!("------------------------------");

    let mut config = Config::new();
    config
        .set_optimization_level(cargo_optimize::OptimizationLevel::Balanced)
        .set_parallel_jobs(8)
        .verbose();

    // Apply the configuration
    cargo_optimize::optimize_with_config(config)?;

    // Check if optimizations are active
    if cargo_optimize::is_optimized() {
        println!("\n✓ Optimizations are active!");
    }

    println!("\nVersion: {}", cargo_optimize::version());

    Ok(())
}
