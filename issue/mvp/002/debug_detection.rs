use std::process::Command;

fn main() {
    println!("=== Testing Windows Linker Detection ===\n");
    
    // Test 1: Try 'where' command
    println!("Test 1: Using 'where' command");
    let result = Command::new("where")
        .arg("rust-lld.exe")
        .output();
    
    match result {
        Ok(output) => {
            println!("  Status: {}", if output.status.success() { "SUCCESS" } else { "FAILED" });
            println!("  Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("  Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("  Error running 'where': {}", e);
        }
    }
    
    // Test 2: Try direct execution with --version
    println!("\nTest 2: Direct execution with --version");
    let result = Command::new("rust-lld")
        .arg("--version")
        .output();
    
    match result {
        Ok(output) => {
            println!("  Status: {}", if output.status.success() { "SUCCESS" } else { "FAILED" });
            println!("  Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("  Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("  Error running 'rust-lld --version': {}", e);
        }
    }
    
    // Test 3: Try with .exe extension
    println!("\nTest 3: Direct execution of rust-lld.exe with --version");
    let result = Command::new("rust-lld.exe")
        .arg("--version")
        .output();
    
    match result {
        Ok(output) => {
            println!("  Status: {}", if output.status.success() { "SUCCESS" } else { "FAILED" });
            println!("  Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("  Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            println!("  Error running 'rust-lld.exe --version': {}", e);
        }
    }
    
    // Test 4: Check what cfg! reports
    println!("\nTest 4: Platform detection");
    println!("  cfg!(target_os = \"windows\"): {}", cfg!(target_os = "windows"));
    println!("  cfg!(target_os = \"linux\"): {}", cfg!(target_os = "linux"));
    
    // Test 5: Try the actual MVP detection
    println!("\nTest 5: MVP detection function");
    cargo_optimize::mvp::auto_configure_mvp();
}
