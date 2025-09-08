#[test]
fn debug_directory_structure() {
    use cargo_optimize::config::*;
    use std::fs;
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create .cargo directory
    let cargo_dir = temp_dir.path().join(".cargo");
    fs::create_dir_all(&cargo_dir).expect("Failed to create .cargo dir");
    
    // Create backups directory
    let backup_dir = cargo_dir.join("backups");
    fs::create_dir_all(&backup_dir).expect("Failed to create backup dir");
    
    println!("Temp dir: {:?}", temp_dir.path());
    println!("Cargo dir: {:?}, exists: {}", cargo_dir, cargo_dir.exists());
    println!("Backup dir: {:?}, exists: {}", backup_dir, backup_dir.exists());
    
    // Create initial config
    let config_path = cargo_dir.join("config.toml");
    fs::write(&config_path, "[build]\njobs = 2\n").expect("Failed to write config");
    println!("Config path: {:?}, exists: {}", config_path, config_path.exists());
    
    // Create cargo-optimize.toml
    let optimize_config_path = temp_dir.path().join("cargo-optimize.toml");
    fs::write(&optimize_config_path, r#"
[global]
optimization_level = "aggressive"
auto_detect_hardware = false
"#).expect("Failed to write cargo-optimize.toml");
    
    // No need to change directory - use base_dir instead
    println!("Temp dir: {:?}", temp_dir.path());
    
    // Create manager with base directory
    std::env::set_var("DEBUG_DIR_GLOBAL__AUTO_DETECT_HARDWARE", "false");
    let manager = ConfigManager::new_with_base_dir(temp_dir.path(), "DEBUG_DIR_")
        .expect("Failed to create manager");
    
    println!("Manager backup dir: {:?}", manager.config().backup.backup_dir);
    
    // Try to create backup
    match manager.create_backup() {
        Ok(backup_path) => println!("Backup created at: {:?}", backup_path),
        Err(e) => println!("Backup failed: {:?}", e),
    }
    
    // Try to apply config
    match manager.apply() {
        Ok(_) => println!("Config applied successfully"),
        Err(e) => println!("Apply failed: {:?}", e),
    }
    
    // Check what was created
    println!("\nDirectory contents:");
    for entry in fs::read_dir(temp_dir.path()).unwrap() {
        if let Ok(entry) = entry {
            println!("  {:?}", entry.path());
        }
    }
    
    if cargo_dir.exists() {
        println!("\n.cargo contents:");
        for entry in fs::read_dir(&cargo_dir).unwrap() {
            if let Ok(entry) = entry {
                println!("  {:?}", entry.path());
            }
        }
    }
    
    std::env::remove_var("DEBUG_DIR_GLOBAL__AUTO_DETECT_HARDWARE");
}
