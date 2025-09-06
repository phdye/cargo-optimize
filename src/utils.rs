//! Utility functions for cargo-optimize

use crate::{Error, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info};

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Create a progress bar
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a spinner for indeterminate progress
pub fn create_spinner(message: &str) -> ProgressBar {
    // In test mode, create a hidden progress bar to avoid terminal conflicts
    if is_test_mode() {
        let pb = ProgressBar::hidden();
        pb.set_message(message.to_string());
        return pb;
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Check if running in CI environment
pub fn is_ci() -> bool {
    env::var("CI").is_ok()
        || env::var("CONTINUOUS_INTEGRATION").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("TRAVIS").is_ok()
}

/// Check if running with verbose output
pub fn is_verbose() -> bool {
    env::var("CARGO_OPTIMIZE_VERBOSE").is_ok()
        || env::var("VERBOSE").is_ok()
        || env::args().any(|arg| arg == "-v" || arg == "--verbose")
}

/// Check if running in dry-run mode
pub fn is_dry_run() -> bool {
    env::var("CARGO_OPTIMIZE_DRY_RUN").is_ok() || env::args().any(|arg| arg == "--dry-run")
}

/// Check if running in test mode
pub fn is_test_mode() -> bool {
    // Check various indicators that we're in a test environment
    cfg!(test) || 
    env::var("CARGO_TEST").is_ok() ||
    env::var("RUST_TEST_THREADS").is_ok() ||
    env::args().any(|arg| arg.contains("test"))
}

/// Backup a file before modifying it
pub fn backup_file(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let backup_path = path.with_extension(format!(
        "{}.backup",
        path.extension().and_then(|s| s.to_str()).unwrap_or("")
    ));

    fs::copy(path, &backup_path)?;
    debug!("Created backup: {:?}", backup_path);

    Ok(backup_path)
}

/// Restore a file from backup
pub fn restore_from_backup(backup_path: impl AsRef<Path>) -> Result<()> {
    let backup_path = backup_path.as_ref();
    let original_path = backup_path.with_extension("");

    fs::copy(backup_path, original_path)?;
    fs::remove_file(backup_path)?;

    debug!("Restored from backup: {:?}", backup_path);
    Ok(())
}

/// Run a command and capture output
pub fn run_command(command: &str, args: &[&str]) -> Result<String> {
    debug!("Running command: {} {:?}", command, args);

    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| Error::other(format!("Failed to run {}: {}", command, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("{} failed: {}", command, stderr)));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if a command is available
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Get the cargo target directory
pub fn get_target_dir() -> Result<PathBuf> {
    // Check CARGO_TARGET_DIR environment variable
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(target_dir));
    }

    // Check cargo metadata
    if let Ok(output) = run_command("cargo", &["metadata", "--format-version", "1"]) {
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&output) {
            if let Some(target_dir) = metadata["target_directory"].as_str() {
                return Ok(PathBuf::from(target_dir));
            }
        }
    }

    // Default to ./target
    Ok(PathBuf::from("target"))
}

/// Clean the target directory
pub fn clean_target_dir() -> Result<()> {
    let target_dir = get_target_dir()?;

    if target_dir.exists() {
        info!("Cleaning target directory: {:?}", target_dir);
        fs::remove_dir_all(&target_dir)?;
    }

    Ok(())
}

/// Get Rust version
pub fn get_rust_version() -> Result<String> {
    let output = run_command("rustc", &["--version"])?;
    Ok(output.trim().to_string())
}

/// Check if using nightly Rust
pub fn is_nightly() -> bool {
    get_rust_version()
        .map(|v| v.contains("nightly"))
        .unwrap_or(false)
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs == 0 {
        format!("{}ms", millis)
    } else if secs < 60 {
        format!("{}.{:03}s", secs, millis)
    } else {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    }
}

/// Create a temporary directory for testing
#[cfg(test)]
pub fn create_temp_project() -> Result<tempfile::TempDir> {
    use std::io::Write;

    let temp_dir = tempfile::tempdir()?;
    let project_path = temp_dir.path();

    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    let mut file = fs::File::create(&cargo_toml)?;
    file.write_all(b"[package]\nname = \"test-project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n")?;

    // Create src/main.rs
    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir)?;

    let main_rs = src_dir.join("main.rs");
    let mut file = fs::File::create(&main_rs)?;
    file.write_all(b"fn main() {\n    println!(\"Hello, world!\");\n}\n")?;

    Ok(temp_dir)
}

/// Platform-specific settings
pub struct PlatformSettings;

impl PlatformSettings {
    /// Get platform-specific environment variables
    pub fn get_env_vars() -> Vec<(String, String)> {
        let mut vars = Vec::new();

        // Windows-specific
        if cfg!(target_os = "windows") {
            // Use faster Windows heap
            vars.push(("RUST_HEAP".to_string(), "1".to_string()));
        }

        // macOS-specific
        if cfg!(target_os = "macos") {
            // Use faster allocator on macOS
            vars.push(("RUST_ALLOCATOR".to_string(), "system".to_string()));
        }

        // Linux-specific
        if cfg!(target_os = "linux") {
            // Use jemalloc if available
            if command_exists("jemalloc-config") {
                vars.push(("RUST_ALLOCATOR".to_string(), "jemalloc".to_string()));
            }
        }

        vars
    }

    /// Apply platform-specific settings
    pub fn apply() {
        for (key, value) in Self::get_env_vars() {
            env::set_var(key, value);
        }
    }
}

/// Benchmark helper
pub struct Benchmark;

impl Benchmark {
    /// Run a simple benchmark
    pub fn run<F: Fn()>(name: &str, iterations: u32, f: F) -> Duration {
        print_info(&format!("Running benchmark: {}", name));

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            f();
        }
        let elapsed = start.elapsed();

        let per_iteration = elapsed / iterations;
        print_success(&format!(
            "Benchmark complete: {} iterations in {}, {} per iteration",
            iterations,
            format_duration(elapsed),
            format_duration(per_iteration)
        ));

        elapsed
    }
}
