use cargo_optimize;

fn main() {
    println!("Testing cargo-optimize MVP: Linker Selection\n");
    println!("Version: {}\n", cargo_optimize::version());
    
    // Test 1: Can we detect and configure linkers?
    println!("=== Test 1: Linker Detection & Configuration ===");
    cargo_optimize::auto_configure();
    
    // Test 2: Did it create the config?
    println!("\n=== Test 2: Config File Check ===");
    if std::path::Path::new(".cargo/config.toml").exists() {
        println!("✓ Config file created");
        match std::fs::read_to_string(".cargo/config.toml") {
            Ok(content) => {
                println!("Config content:\n{}", content);
                if content.contains("mold") {
                    println!("✓ Using mold linker (fastest!)");
                } else if content.contains("lld") {
                    println!("✓ Using lld linker (very fast)");
                } else if content.contains("gold") {
                    println!("✓ Using gold linker (fast)");
                }
            }
            Err(e) => println!("✗ Failed to read config: {}", e),
        }
    } else {
        println!("ℹ No config file created (might be on non-Linux system or no fast linkers available)");
    }
    
    // Test 3: Instructions for real-world testing
    println!("\n=== Test 3: Real-World Impact ===");
    println!("To measure the actual improvement:");
    println!("  1. cd to any Rust project");
    println!("  2. cargo clean");
    println!("  3. time cargo build --release");
    println!("  4. Note the time");
    println!("  5. rm .cargo/config.toml");
    println!("  6. cargo clean");
    println!("  7. time cargo build --release");
    println!("  8. Compare the times!");
    println!("\nExpected improvement: 20-70% faster linking!");
}
