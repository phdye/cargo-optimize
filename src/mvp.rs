/// Minimal Viable Product - Just automatic linker optimization
/// 
/// This is a completely standalone implementation that doesn't depend on
/// any of the complex modules. Once this works, we can gradually add features.
///
/// Supports both Windows (including from Cygwin) and Linux platforms.

use std::process::Command;
use std::fs;
use std::path::Path;

/// The only public function - automatically configures the fastest linker
pub fn auto_configure_mvp() {
    if let Ok(linker) = detect_best_linker() {
        if linker != "default" {
            configure_linker(&linker);
            println!("cargo-optimize MVP: Using {} linker", linker);
        } else {
            println!("cargo-optimize MVP: Using default linker (no fast linker found)");
        }
    }
}

fn detect_best_linker() -> Result<String, Box<dyn std::error::Error>> {
    // Check for Windows (including when running from Cygwin)
    if cfg!(target_os = "windows") {
        // On Windows, rust-lld is available if Rust is installed
        // It's invoked by cargo/rustc, so we don't need it in PATH
        // We just need to check if Rust is installed
        if rust_is_installed() {
            // rust-lld.exe is bundled with Rust toolchains
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
    // Check if we can run rustc
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
    // Try 'where' command (Windows native)
    Command::new("where")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or_else(|_| {
            // Fallback: try to run the command directly with --version
            Command::new(cmd)
                .arg("--version")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        })
}

fn configure_linker(linker: &str) {
    let config_dir = Path::new(".cargo");
    let _ = fs::create_dir_all(config_dir);
    
    let config_path = config_dir.join("config.toml");
    
    let config_content = if cfg!(target_os = "windows") {
        // Windows configurations
        match linker {
            "rust-lld" => {
                // rust-lld.exe is in the toolchain, cargo knows where to find it
                "[target.x86_64-pc-windows-msvc]\n\
                 linker = \"rust-lld\"\n"
            },
            "lld-link" => {
                "[target.x86_64-pc-windows-msvc]\n\
                 linker = \"lld-link.exe\"\n"
            },
            _ => return,
        }
    } else {
        // Linux configurations
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
            _ => return,
        }
    };
    
    if let Err(e) = fs::write(&config_path, config_content) {
        eprintln!("cargo-optimize MVP: Failed to write config: {}", e);
    } else {
        println!("cargo-optimize MVP: Created .cargo/config.toml");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_linker() {
        // This test just verifies the function runs without panic
        let result = detect_best_linker();
        assert!(result.is_ok());
        
        // On Windows with Rust installed, should detect rust-lld
        if cfg!(target_os = "windows") && rust_is_installed() {
            assert_eq!(result.unwrap(), "rust-lld");
        }
    }
    
    #[test] 
    fn test_rust_installed() {
        // If this test is running, Rust must be installed
        assert!(rust_is_installed());
    }
    
    #[test]
    fn test_command_exists() {
        if cfg!(target_os = "windows") {
            // On Windows, cmd.exe should always exist
            assert!(command_exists_windows("cmd.exe") || command_exists_windows("cmd"));
        } else if cfg!(unix) {
            // 'sh' should exist on all Unix-like systems
            assert!(command_exists_unix("sh"));
        }
        
        // A command that definitely doesn't exist
        assert!(!command_exists_windows("this-command-definitely-does-not-exist-12345"));
        assert!(!command_exists_unix("this-command-definitely-does-not-exist-12345"));
    }
}
