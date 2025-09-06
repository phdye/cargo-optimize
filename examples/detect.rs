//! Example: Detect hardware and environment

use cargo_optimize::detector::{Environment, HardwareInfo, ToolchainInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Hardware & Environment Detection\n");

    // Detect complete environment
    let env = Environment::detect()?;

    // Display hardware information
    println!("💻 Hardware Information");
    println!("=======================");
    display_hardware(&env.hardware);

    // Display toolchain information
    println!("\n🦀 Rust Toolchain");
    println!("==================");
    display_toolchain(&env.toolchain);

    // Display environment information
    println!("\n🌍 Environment");
    println!("===============");

    if let Some(ci) = &env.ci_environment {
        println!("CI Environment: {:?}", ci);
        let settings = ci.recommended_settings();
        println!(
            "  Recommended parallel jobs: {}",
            settings.max_parallel_jobs
        );
        println!("  Cache enabled: {}", settings.enable_cache);
    } else {
        println!("CI Environment: Not detected (local development)");
    }

    println!("Container: {}", if env.is_container { "Yes" } else { "No" });
    println!("WSL: {}", if env.is_wsl { "Yes" } else { "No" });

    // Display optimization recommendations
    println!("\n⚡ Optimization Recommendations");
    println!("================================");

    println!(
        "Recommended parallel jobs: {}",
        env.hardware.recommended_jobs()
    );
    println!(
        "Sufficient memory for aggressive opts: {}",
        if env.hardware.has_sufficient_memory() {
            "Yes"
        } else {
            "No"
        }
    );
    println!(
        "CPU target for native optimization: {}",
        env.hardware.cpu_target()
    );
    println!("Preferred linker: {}", env.hardware.operating_system.preferred_linker());

    // Check for specific features
    println!("\n🔧 Available Features");
    println!("=====================");

    use cargo_optimize::detector::RustFeature;
    let features = [
        (RustFeature::ParallelFrontend, "Parallel Frontend"),
        (RustFeature::SplitDebuginfo, "Split Debuginfo"),
        (RustFeature::ShareGenerics, "Share Generics"),
        (RustFeature::BuildStdCore, "Build Std Core"),
    ];

    for (feature, name) in features {
        let available = env.toolchain.has_feature(feature);
        let symbol = if available { "✅" } else { "❌" };
        println!(
            "{} {}: {}",
            symbol,
            name,
            if available {
                "Available"
            } else {
                "Not available"
            }
        );
    }

    Ok(())
}

fn display_hardware(hw: &HardwareInfo) {
    println!("CPU: {}", hw.cpu_brand);
    println!("Architecture: {:?}", hw.cpu_arch);
    println!(
        "Cores: {} physical, {} logical",
        hw.cpu_cores, hw.logical_cpus
    );

    if hw.cpu_frequency > 0 {
        println!("Frequency: {} MHz", hw.cpu_frequency);
    }

    println!(
        "Memory: {} / {} available",
        format_bytes(hw.available_memory),
        format_bytes(hw.total_memory)
    );

    println!("Operating System: {:?}", hw.os);

    // Check for SIMD support
    println!("\nSIMD Support:");
    println!(
        "  AVX2: {}",
        if hw.cpu_arch.supports_avx2() {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "  NEON: {}",
        if hw.cpu_arch.supports_neon() {
            "✅"
        } else {
            "❌"
        }
    );
}

fn display_toolchain(toolchain: &ToolchainInfo) {
    println!("Rust version: {}", toolchain.rust_version);
    println!("Cargo version: {}", toolchain.cargo_version);
    println!("Default target: {}", toolchain.default_target);
    println!("Channel: {:?}", toolchain.channel);

    if toolchain.installed_targets.len() > 1 {
        println!("Installed targets:");
        for target in &toolchain.installed_targets {
            println!("  - {}", target);
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}
