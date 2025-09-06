/// Minimal Viable Product - Just automatic linker optimization
/// 
/// This is a completely standalone implementation that doesn't depend on
/// any of the complex modules. Once this works, we can gradually add features.
///
/// Supports both Windows (including from Cygwin) and Linux platforms.
/// Safely handles existing .cargo/config.toml files.
use std::process::Command;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Configuration options for the MVP
pub struct MvpConfig {
    /// Whether to create a backup of existing config
    pub backup: bool,
    /// Whether to force overwrite existing settings
    pub force: bool,
    /// Whether to run in dry-run mode (no changes)
    pub dry_run: bool,
}

impl Default for MvpConfig {
    fn default() -> Self {
        MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
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
                Ok(ConfigResult::Skipped(reason)) => {
                    println!("cargo-optimize: ⚠️  {}", reason);
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
    Skipped(String),
    DryRun,
}

fn configure_linker_safe(linker: &str, config: &MvpConfig) -> Result<ConfigResult, Box<dyn std::error::Error>> {
    let config_dir = Path::new(".cargo");
    let config_path = config_dir.join("config.toml");
    
    // Get the new config content for this linker
    let new_content = get_linker_config(linker)?;
    
    // Dry run mode - just report what would be done
    if config.dry_run {
        if config_path.exists() {
            println!("cargo-optimize: Would update existing .cargo/config.toml");
        } else {
            println!("cargo-optimize: Would create .cargo/config.toml");
        }
        return Ok(ConfigResult::DryRun);
    }
    
    // Check if config already exists
    if config_path.exists() {
        // Read existing config
        let mut existing_content = String::new();
        fs::File::open(&config_path)?.read_to_string(&mut existing_content)?;
        
        // Check if it already has linker configuration
        if has_linker_config(&existing_content) {
            if config.force {
                // Backup and overwrite
                if config.backup {
                    backup_config(&config_path)?;
                }
                fs::write(&config_path, new_content)?;
                Ok(ConfigResult::Updated)
            } else {
                // Check if it's already using a fast linker
                if is_using_fast_linker(&existing_content) {
                    Ok(ConfigResult::AlreadyOptimized)
                } else {
                    Ok(ConfigResult::Skipped(
                        "Existing config has linker settings. Use --force to override or manually merge:\n\
                         \n\
                         Add this to your .cargo/config.toml:\n\
                         \n".to_string() + &new_content
                    ))
                }
            }
        } else {
            // No linker config exists - safe to append
            if config.backup {
                backup_config(&config_path)?;
            }
            
            // Append our config with a separator
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&config_path)?;
            
            writeln!(file, "\n# Added by cargo-optimize")?;
            write!(file, "{}", new_content)?;
            
            Ok(ConfigResult::Updated)
        }
    } else {
        // No config exists - create it
        fs::create_dir_all(config_dir)?;
        fs::write(&config_path, new_content)?;
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

fn detect_best_linker() -> Result<String, Box<dyn std::error::Error>> {
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
}
