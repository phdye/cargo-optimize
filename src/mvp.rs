/// Minimal Viable Product - Just automatic linker optimization
/// 
/// This is a completely standalone implementation that doesn't depend on
/// any of the complex modules. Once this works, we can gradually add features.
///
/// Supports both Windows (including from Cygwin) and Linux platforms.
/// Safely handles existing .cargo/config.toml files with intelligent merging.
use std::process::Command;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Configuration options for the MVP
pub struct MvpConfig {
    /// Whether to create a backup of existing config
    pub backup: bool,
    /// Whether to force overwrite existing settings
    pub force: bool,
    /// Whether to run in dry-run mode (no changes)
    pub dry_run: bool,
    /// Whether to include timestamps in comments (disable for deterministic testing)
    pub include_timestamps: bool,
}

impl Default for MvpConfig {
    fn default() -> Self {
        MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: true,
        }
    }
}

/// The main public function - automatically configures the fastest linker
pub fn auto_configure_mvp() {
    auto_configure_with_options(MvpConfig::default());
}

/// Configure with custom options
pub fn auto_configure_with_options(config: MvpConfig) {
    match detect_best_linker() {
        Ok(linker) if linker != "default" => {
            match configure_linker_safe(&linker, &config) {
                Ok(ConfigResult::Created) => {
                    println!("cargo-optimize: ✅ Created .cargo/config.toml with {} linker", linker);
                }
                Ok(ConfigResult::Updated) => {
                    println!("cargo-optimize: ✅ Updated .cargo/config.toml to use {} linker", linker);
                }
                Ok(ConfigResult::AlreadyOptimized) => {
                    println!("cargo-optimize: ℹ️  Config already optimized with fast linker");
                }
                Ok(ConfigResult::DryRun) => {
                    println!("cargo-optimize: 🔍 Would configure {} linker (dry run)", linker);
                }
                Err(e) => {
                    eprintln!("cargo-optimize: ❌ Failed to configure linker: {}", e);
                }
            }
        }
        Ok(_) => {
            println!("cargo-optimize: ℹ️  No fast linker found - using default");
        }
        Err(e) => {
            eprintln!("cargo-optimize: ❌ Error detecting linker: {}", e);
        }
    }
}

/// Configure with custom options at a specific base path (for testing and isolated usage)
pub fn auto_configure_with_options_at(config: MvpConfig, base_path: Option<&Path>) {
    match detect_best_linker() {
        Ok(linker) if linker != "default" => {
            match configure_linker_safe_at(&linker, &config, base_path) {
                Ok(ConfigResult::Created) => {
                    println!("cargo-optimize: ✅ Created .cargo/config.toml with {} linker", linker);
                }
                Ok(ConfigResult::Updated) => {
                    println!("cargo-optimize: ✅ Updated .cargo/config.toml to use {} linker", linker);
                }
                Ok(ConfigResult::AlreadyOptimized) => {
                    println!("cargo-optimize: ℹ️  Config already optimized with fast linker");
                }
                Ok(ConfigResult::DryRun) => {
                    println!("cargo-optimize: 🔍 Would configure {} linker (dry run)", linker);
                }
                Err(e) => {
                    eprintln!("cargo-optimize: ❌ Failed to configure linker: {}", e);
                }
            }
        }
        Ok(_) => {
            println!("cargo-optimize: ℹ️  No fast linker found - using default");
        }
        Err(e) => {
            eprintln!("cargo-optimize: ❌ Error detecting linker: {}", e);
        }
    }
}

#[derive(Debug)]
enum ConfigResult {
    Created,
    Updated,
    AlreadyOptimized,
    DryRun,
}

fn configure_linker_safe(linker: &str, config: &MvpConfig) -> Result<ConfigResult, Box<dyn std::error::Error>> {
    configure_linker_safe_at(linker, config, None)
}

fn configure_linker_safe_at(linker: &str, config: &MvpConfig, base_path: Option<&Path>) -> Result<ConfigResult, Box<dyn std::error::Error>> {
    // Use base_path or current directory
    let base = base_path.map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let config_dir = base.join(".cargo");
    let config_path = config_dir.join("config.toml");
    
    // Get the new config content for this linker
    let new_content = get_linker_config(linker)?;
    
    // Dry run mode - just report what would be done, NO file operations
    if config.dry_run {
        if config_path.exists() {
            let existing_content = fs::read_to_string(&config_path)?;
            if has_linker_config(&existing_content) {
                if is_using_fast_linker(&existing_content) {
                    println!("cargo-optimize: Config already has fast linker (dry run)");
                } else {
                    println!("cargo-optimize: Would update existing linker config (dry run)");
                }
            } else {
                println!("cargo-optimize: Would append linker config to existing .cargo/config.toml (dry run)");
            }
        } else {
            // Don't create directories in dry-run mode
            println!("cargo-optimize: Would create .cargo/config.toml with {} linker (dry run)", linker);
        }
        // IMPORTANT: Return early, do NOT continue to actual file operations
        return Ok(ConfigResult::DryRun);
    }
    
    // Check if config already exists
    if config_path.exists() {
        // Read and validate existing config
        let existing_content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => {
                return Err(format!("Failed to read existing config: {}", e).into());
            }
        };
        
        // Handle empty config file
        if existing_content.trim().is_empty() {
            // Treat empty file as no config - just write new content
            fs::write(&config_path, &new_content)?;
            return Ok(ConfigResult::Updated);
        }
        
        // Basic TOML validation - check for common syntax issues
        if !is_valid_toml_syntax(&existing_content) {
            if config.backup {
                let backup_path = backup_config(&config_path)?;
                eprintln!("cargo-optimize: ⚠️  Existing config appears to be malformed. Backed up to {}", backup_path.display());
            }
            
            // If force flag is set, overwrite with new config
            if config.force {
                fs::write(&config_path, &new_content)?;
                return Ok(ConfigResult::Updated);
            }
            
            return Err("Existing config.toml appears to be malformed. Please fix it manually or use --force to overwrite.".into());
        }
        
        // Check if it already has linker configuration
        if has_linker_config(&existing_content) {
            if config.force {
                // Backup and merge instead of overwriting completely
                if config.backup {
                    backup_config(&config_path)?;
                }
                
                // Try to merge intelligently
                let merged_content = merge_linker_config(&existing_content, &new_content, linker, config)?;
                fs::write(&config_path, merged_content)?;
                Ok(ConfigResult::Updated)
            } else {
                // Check if it's already using a fast linker
                if is_using_fast_linker(&existing_content) {
                    Ok(ConfigResult::AlreadyOptimized)
                } else {
                    // If no fast linker is configured, append the configuration
                    if config.backup {
                        backup_config(&config_path)?;
                    }
                    let merged_content = append_linker_config(&existing_content, &new_content, config)?;
                    fs::write(&config_path, merged_content)?;
                    Ok(ConfigResult::Updated)
                }
            }
        } else {
            // No linker config exists - safe to append
            if config.backup {
                backup_config(&config_path)?;
            }
            
            // Append our config with proper formatting
            let merged_content = append_linker_config(&existing_content, &new_content, config)?;
            fs::write(&config_path, merged_content)?;
            
            Ok(ConfigResult::Updated)
        }
    } else {
        // No config exists - create it
        fs::create_dir_all(config_dir)?;
        
        // Add a header comment for new files
        let content_with_header = if config.include_timestamps {
            let timestamp = format_timestamp();
            format!(
                "# Cargo configuration - optimized by cargo-optimize\n\
                 # Generated: {}\n\n{}",
                timestamp,
                new_content
            )
        } else {
            format!(
                "# Cargo configuration - optimized by cargo-optimize\n\n{}",
                new_content
            )
        };
        
        fs::write(&config_path, content_with_header)?;
        Ok(ConfigResult::Created)
    }
}

fn get_linker_config(linker: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = if cfg!(target_os = "windows") {
        match linker {
            "rust-lld" => {
                "[target.x86_64-pc-windows-msvc]\n\
                 linker = \"rust-lld\"\n"
            },
            "lld-link" => {
                "[target.x86_64-pc-windows-msvc]\n\
                 linker = \"lld-link.exe\"\n"
            },
            _ => return Err("Unknown linker".into()),
        }
    } else {
        match linker {
            "mold" => {
                "[target.x86_64-unknown-linux-gnu]\n\
                 linker = \"clang\"\n\
                 rustflags = [\"-C\", \"link-arg=-fuse-ld=mold\"]\n"
            },
            "lld" => {
                "[target.x86_64-unknown-linux-gnu]\n\
                 linker = \"clang\"\n\
                 rustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]\n"
            },
            "gold" => {
                "[target.x86_64-unknown-linux-gnu]\n\
                 linker = \"clang\"\n\
                 rustflags = [\"-C\", \"link-arg=-fuse-ld=gold\"]\n"
            },
            _ => return Err("Unknown linker".into()),
        }
    };
    
    Ok(config.to_string())
}

fn has_linker_config(content: &str) -> bool {
    // Check if the config already has linker settings for the current platform
    if cfg!(target_os = "windows") {
        content.contains("[target.x86_64-pc-windows-msvc]") ||
        content.contains("[target.'cfg(windows)']")
    } else {
        content.contains("[target.x86_64-unknown-linux-gnu]") ||
        content.contains("[target.'cfg(unix)']") ||
        content.contains("[target.'cfg(target_os = \"linux\")']")
    }
}

fn is_using_fast_linker(content: &str) -> bool {
    // Check if already using a fast linker
    content.contains("rust-lld") ||
    content.contains("lld-link") ||
    content.contains("mold") ||
    content.contains("lld") ||
    content.contains("gold")
}

fn backup_config(config_path: &Path) -> io::Result<PathBuf> {
    let backup_path = config_path.with_extension("toml.backup");
    
    // If backup already exists, add a number
    let mut final_backup_path = backup_path.clone();
    let mut counter = 1;
    while final_backup_path.exists() {
        final_backup_path = config_path.with_extension(format!("toml.backup.{}", counter));
        counter += 1;
    }
    
    fs::copy(config_path, &final_backup_path)?;
    println!("cargo-optimize: 📋 Backed up existing config to {}", final_backup_path.display());
    
    Ok(final_backup_path)
}

/// Basic TOML syntax validation - checks for common issues
fn is_valid_toml_syntax(content: &str) -> bool {
    // Basic checks for TOML validity
    let mut bracket_count = 0;
    let mut quote_count = 0;
    let mut in_string = false;
    let mut prev_char = ' ';
    
    for ch in content.chars() {
        match ch {
            '"' if prev_char != '\\' => {
                quote_count += 1;
                in_string = !in_string;
            }
            '[' if !in_string => bracket_count += 1,
            ']' if !in_string => bracket_count -= 1,
            _ => {}
        }
        
        if bracket_count < 0 {
            return false; // More closing brackets than opening
        }
        
        prev_char = ch;
    }
    
    // Check for balanced brackets and quotes
    bracket_count == 0 && quote_count % 2 == 0
}

/// Merge linker configuration intelligently
fn merge_linker_config(existing: &str, new_config: &str, linker: &str, config: &MvpConfig) -> Result<String, Box<dyn std::error::Error>> {
    // Find the target section in existing config
    let target_section = if cfg!(target_os = "windows") {
        "[target.x86_64-pc-windows-msvc]"
    } else {
        "[target.x86_64-unknown-linux-gnu]"
    };
    
    // Check if the target section exists
    if let Some(section_start) = existing.find(target_section) {
        // Find the end of this section (next section or end of file)
        let section_content_start = section_start + target_section.len();
        let section_end = existing[section_content_start..]
            .find("\n[")
            .map(|pos| section_content_start + pos)
            .unwrap_or(existing.len());
        
        // Extract the section
        let before_section = &existing[..section_start];
        let after_section = &existing[section_end..];
        
        // Build the new section with merged content
        let mut merged = String::new();
        merged.push_str(before_section);
        
        // Add comment about the update
        merged.push_str(&format!("# Updated by cargo-optimize to use {} linker\n", linker));
        
        // Add the new configuration
        merged.push_str(new_config);
        
        // Add the rest of the file
        merged.push_str(after_section);
        
        Ok(merged)
    } else {
        // No existing target section, append the new config
        append_linker_config(existing, new_config, config)
    }
}

/// Append linker configuration to existing config file
fn append_linker_config(existing: &str, new_config: &str, config: &MvpConfig) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::from(existing);
    
    // Ensure there's proper spacing
    if !result.ends_with('\n') {
        result.push('\n');
    }
    
    // Add a visual separator and comment
    result.push_str("\n# ============================================\n");
    result.push_str("# Added by cargo-optimize for faster builds\n");
    if config.include_timestamps {
        result.push_str(&format!("# Timestamp: {}\n", format_timestamp()));
    }
    result.push_str("# ============================================\n\n");
    
    // Add the new configuration
    result.push_str(new_config);
    
    // Ensure file ends with newline
    if !result.ends_with('\n') {
        result.push('\n');
    }
    
    Ok(result)
}

/// Format current timestamp without external dependencies
fn format_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    
    let total_secs = duration.as_secs();
    
    // Simple timestamp format - not as nice as chrono but works without dependencies
    // This is good enough for a comment in the config file
    format!("Unix timestamp: {}", total_secs)
}

/// Detect the best available linker for the current platform
/// 
/// Returns the name of the fastest linker available, or "default" if no fast linker is found.
/// On Windows, prefers rust-lld. On Linux, prefers mold > lld > gold.
pub fn detect_best_linker() -> Result<String, Box<dyn std::error::Error>> {
    // Check for Windows (including when running from Cygwin)
    if cfg!(target_os = "windows") {
        // On Windows, rust-lld is available if Rust is installed
        if rust_is_installed() {
            return Ok("rust-lld".to_string());
        }
        
        // Fallback: check for LLVM's lld-link
        if command_exists_windows("lld-link.exe") || command_exists_windows("lld-link") {
            return Ok("lld-link".to_string());
        }
    } else if cfg!(target_os = "linux") {
        // Linux linkers
        let linkers = [
            ("mold", "mold"),
            ("lld", "lld"),  
            ("gold", "gold"),
        ];
        
        for (name, command) in &linkers {
            if command_exists_unix(command) {
                return Ok(name.to_string());
            }
        }
    }
    
    // Default to system linker
    Ok("default".to_string())
}

fn rust_is_installed() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_exists_unix(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_exists_windows(cmd: &str) -> bool {
    Command::new("where")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or_else(|_| {
            Command::new(cmd)
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_detect_linker() {
        let result = detect_best_linker();
        assert!(result.is_ok());
        
        if cfg!(target_os = "windows") && rust_is_installed() {
            assert_eq!(result.unwrap(), "rust-lld");
        }
    }
    
    #[test] 
    fn test_rust_installed() {
        assert!(rust_is_installed());
    }
    
    #[test]
    fn test_has_linker_config() {
        let windows_config = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"";
        let linux_config = "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"";
        
        if cfg!(target_os = "windows") {
            assert!(has_linker_config(windows_config));
            assert!(!has_linker_config(linux_config));
        } else {
            assert!(has_linker_config(linux_config));
            assert!(!has_linker_config(windows_config));
        }
    }
    
    #[test]
    fn test_is_using_fast_linker() {
        assert!(is_using_fast_linker("linker = \"rust-lld\""));
        assert!(is_using_fast_linker("linker = \"mold\""));
        assert!(is_using_fast_linker("link-arg=-fuse-ld=lld"));
        assert!(!is_using_fast_linker("linker = \"link.exe\""));
    }
    
    #[test]
    fn test_backup_numbering() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".cargo").join("config.toml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        
        // Create original file
        fs::write(&config_path, "original").unwrap();
        
        // First backup
        let backup1 = backup_config(&config_path).unwrap();
        assert_eq!(backup1.file_name().unwrap(), "config.toml.backup");
        
        // Second backup should get a number
        fs::write(&config_path, "modified").unwrap();
        let backup2 = backup_config(&config_path).unwrap();
        assert_eq!(backup2.file_name().unwrap(), "config.toml.backup.1");
    }
    
    #[test]
    fn test_valid_toml_syntax() {
        // Valid TOML
        assert!(is_valid_toml_syntax("[package]\nname = \"test\""));
        assert!(is_valid_toml_syntax("[target]\nlinker = \"lld\""));
        assert!(is_valid_toml_syntax(""));
        
        // Invalid TOML
        assert!(!is_valid_toml_syntax("[unclosed"));
        assert!(!is_valid_toml_syntax("name = \"unclosed string"));
        assert!(!is_valid_toml_syntax("][["));
        assert!(!is_valid_toml_syntax("[too][many]]"));
    }
    
    #[test]
    fn test_append_linker_config() {
        let existing = "[package]\nname = \"test\"\n";
        let new_config = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n";
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false, // Disable timestamps for deterministic testing
        };
        
        let result = append_linker_config(existing, new_config, &config).unwrap();
        
        // Should contain both the original and new config
        assert!(result.contains("[package]"));
        assert!(result.contains("name = \"test\""));
        assert!(result.contains("[target.x86_64-pc-windows-msvc]"));
        assert!(result.contains("linker = \"rust-lld\""));
        
        // Should have the comment header
        assert!(result.contains("Added by cargo-optimize"));
    }
    
    #[test]
    fn test_merge_linker_config() {
        let existing = "[package]\nname = \"test\"\n\n[target.x86_64-pc-windows-msvc]\nrustflags = [\"-C\", \"opt-level=3\"]\n";
        let new_config = "[target.x86_64-pc-windows-msvc]\nlinker = \"rust-lld\"\n";
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false, // Disable timestamps for deterministic testing
        };
        
        if cfg!(target_os = "windows") {
            let result = merge_linker_config(existing, new_config, "rust-lld", &config).unwrap();
            
            // Should preserve package section
            assert!(result.contains("[package]"));
            assert!(result.contains("name = \"test\""));
            
            // Should have the updated target section
            assert!(result.contains("[target.x86_64-pc-windows-msvc]"));
            assert!(result.contains("linker = \"rust-lld\""));
            
            // Should have update comment
            assert!(result.contains("Updated by cargo-optimize"));
        }
    }
}

/// Create optimized config at specified path (used by tests)
pub fn create_optimized_config(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let linker = detect_best_linker()?;
    if linker != "default" {
        let config_content = get_linker_config(&linker)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(config_path, config_content)?;
    }
    Ok(())
}
