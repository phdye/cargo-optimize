/// Quick Start Example for cargo-optimize
/// 
/// This example demonstrates the simplest way to use cargo-optimize
/// to automatically configure fast linkers for your project.
fn main() {
    println!("=== cargo-optimize Quick Start Example ===\n");
    
    // That's literally it! One function call.
    cargo_optimize::auto_configure();
    
    println!("\n✓ Configuration complete!");
    println!("\nTo see the effect:");
    println!("  1. Check .cargo/config.toml for linker settings");
    println!("  2. Run 'cargo clean && cargo build --release'");
    println!("  3. Enjoy faster build times!");
}
