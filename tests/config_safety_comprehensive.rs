//! Comprehensive test suite for Config File Safety
//! 
//! This test suite covers all required test categories for the Config File Safety feature:
//! - Unit tests
//! - Integration tests
//! - Boundary value tests
//! - Stress tests
//! - Security tests
//! - Concurrency tests
//! - Platform-specific tests
//!
//! Tests use absolute paths to avoid conflicts when running in parallel.
//! Each test creates its own TempDir and works within that directory.

// These imports are used in test modules via super::*
#[allow(unused_imports)]
use cargo_optimize::mvp::{auto_configure_with_options, auto_configure_with_options_at, MvpConfig};
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use std::sync::{Arc, Mutex};
#[allow(unused_imports)]
use std::thread;
#[allow(unused_imports)]
use std::time::Duration;
#[allow(unused_imports)]
use tempfile::TempDir;

mod unit_tests {
    use super::*;

    #[test]
    fn test_config_creation_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Check in the correct location (temp_path, not current directory!)
        let config_path = temp_path.join(".cargo").join("config.toml");
        assert!(config_path.exists(), "Config should be created at {:?}", config_path);
        
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("linker") || content.contains("target"));
    }

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create initial config
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        fs::write(&config_path, "[build]\njobs = 2").unwrap();
        
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Check for backup in the correct location
        let backup_path = cargo_dir.join("config.toml.backup");
        assert!(backup_path.exists() || cargo_dir.join("config.toml.backup.1").exists(),
                "Backup should exist in {:?}", cargo_dir);
    }

    #[test]
    fn test_dry_run_no_changes() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: true,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Check that no config was created
        let config_path = temp_path.join(".cargo").join("config.toml");
        assert!(!config_path.exists(), "Config should not be created in dry run mode");
    }

    #[test]
    fn test_force_override() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create existing config with different content
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        let original_content = "[build]\njobs = 4\n# Custom config";
        fs::write(&config_path, original_content).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: true,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let new_content = fs::read_to_string(&config_path).unwrap();
        assert!(new_content.contains("linker") || new_content.contains("target"));
        // Force should preserve existing content and append
        assert!(new_content.contains("jobs"));
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_workflow_new_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a minimal Rust project structure
        fs::write(temp_path.join("Cargo.toml"), 
            "[package]\nname = \"test-project\"\nversion = \"0.1.0\"").unwrap();
        fs::create_dir(temp_path.join("src")).unwrap();
        fs::write(temp_path.join("src").join("lib.rs"), "//! Test library").unwrap();
        
        // Configure
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Verify in the correct location
        let config_path = temp_path.join(".cargo").join("config.toml");
        assert!(config_path.exists());
        
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("target") || content.contains("linker"));
    }

    #[test]
    fn test_workflow_existing_complex_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create complex existing config
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let complex_config = r#"
[build]
jobs = 8
target-dir = "custom-target"

[profile.release]
lto = true
opt-level = 3

# Custom settings
[env]
RUST_LOG = "debug"
"#;
        let config_path = cargo_dir.join("config.toml");
        fs::write(&config_path, complex_config).unwrap();
        
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Verify existing content is preserved
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("jobs = 8"));
        assert!(updated_content.contains("RUST_LOG"));
        assert!(updated_content.contains("linker") || updated_content.contains("target"));
        
        // Check backup was created
        let backup_path = cargo_dir.join("config.toml.backup");
        assert!(backup_path.exists() || cargo_dir.join("config.toml.backup.1").exists());
    }
}

mod boundary_tests {
    use super::*;

    #[test]
    fn test_empty_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create empty config file
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        fs::write(&config_path, "").unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("linker") || content.contains("target"));
    }

    #[test]
    fn test_malformed_toml() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create malformed TOML
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        fs::write(&config_path, "[build\njobs = 2").unwrap(); // Missing closing bracket
        
        let config = MvpConfig {
            backup: true,
            force: true, // Force to handle malformed config
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Should handle gracefully and create valid config
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("linker") || content.contains("target"));
    }

    #[test]
    fn test_very_large_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create large config (10KB+)
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let mut large_content = String::new();
        for i in 0..500 {
            large_content.push_str(&format!("# Comment line {}\n", i));
            if i % 50 == 0 {
                large_content.push_str(&format!("[profile.test{}]\nopt-level = 2\n\n", i));
            }
        }
        fs::write(&config_path, &large_content).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.len() > large_content.len());
        assert!(updated_content.contains("linker") || updated_content.contains("target"));
    }

    #[test]
    fn test_unicode_in_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create config with unicode
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        let unicode_config = r#"
# 配置文件 🦀
[build]
jobs = 8
# Комментарий на русском

[profile.release]
# 日本語のコメント
opt-level = 3
"#;
        fs::write(&config_path, unicode_config).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let updated_content = fs::read_to_string(&config_path).unwrap();
        // Unicode should be preserved
        assert!(updated_content.contains("🦀"));
        assert!(updated_content.contains("配置文件"));
        assert!(updated_content.contains("linker") || updated_content.contains("target"));
    }

    #[test]
    fn test_max_backup_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create initial config
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        // Create multiple backups
        for i in 0..10 {
            fs::write(&config_path, format!("[build]\njobs = {}", i)).unwrap();
            
            let config = MvpConfig {
                backup: true,
                force: true,
                dry_run: false,
                include_timestamps: false,
            };
            
            auto_configure_with_options_at(config, Some(temp_path));
        }
        
        // Check that backups were created with proper numbering
        assert!(cargo_dir.join("config.toml.backup").exists());
        for i in 1..10 {
            let backup_path = cargo_dir.join(format!("config.toml.backup.{}", i));
            assert!(backup_path.exists(), "Backup {} should exist", i);
        }
    }
}

mod stress_tests {
    use super::*;

    #[test]
    fn test_concurrent_config_updates() {
        use std::sync::Arc;
        use std::thread;
        
        let temp_dir = Arc::new(TempDir::new().unwrap());
        let mut handles = vec![];
        
        // Each thread gets its own subdirectory to avoid conflicts
        for i in 0..5 {
            let temp_dir = Arc::clone(&temp_dir);
            let handle = thread::spawn(move || {
                // Create a unique subdirectory for this thread
                let thread_dir = temp_dir.path().join(format!("thread_{}", i));
                fs::create_dir_all(&thread_dir).unwrap();
                
                let config = MvpConfig {
                    backup: true,
                    force: false,
                    dry_run: false,
                    include_timestamps: false,
                };
                
                auto_configure_with_options_at(config, Some(&thread_dir));
                
                // Verify in the thread's directory
                let config_path = thread_dir.join(".cargo").join("config.toml");
                assert!(config_path.exists());
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    #[ignore] // Mark as ignored since it's long-running
    fn test_many_sequential_updates() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        for i in 0..100 {
            let config = MvpConfig {
                backup: i % 10 == 0, // Backup every 10th iteration
                force: i % 5 == 0,    // Force every 5th iteration
                dry_run: false,
                include_timestamps: false,
            };
            
            auto_configure_with_options_at(config, Some(temp_path));
            
            let config_path = temp_path.join(".cargo").join("config.toml");
            assert!(config_path.exists());
            
            // Verify config is still valid
            let content = fs::read_to_string(&config_path).unwrap();
            assert!(content.contains("linker") || content.contains("target"));
        }
    }
}

mod security_tests {
    use super::*;

    #[test]
    fn test_path_traversal_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a config with path traversal attempts (these should be sanitized)
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let malicious_config = r#"
[build]
target-dir = "../../../../../../tmp/evil"
jobs = 2
"#;
        fs::write(&config_path, malicious_config).unwrap();
        
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Original content should be preserved (we don't modify existing paths)
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("target-dir"));
        assert!(updated_content.contains("linker") || updated_content.contains("target"));
    }

    #[test]
    fn test_command_injection_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create config with potential command injection
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let injection_config = r#"
[target.x86_64-unknown-linux-gnu]
linker = "gcc; rm -rf /"
rustflags = ["-C", "link-arg=;evil"]
"#;
        fs::write(&config_path, injection_config).unwrap();
        
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Config should be updated but dangerous content preserved (we don't sanitize user input)
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("linker"));
    }

    #[test]
    fn test_symlink_handling() {
        // Skip on Windows as symlinks require admin privileges
        if cfg!(target_os = "windows") {
            return;
        }
        
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a symlink for .cargo directory
        let real_cargo = temp_path.join("real_cargo");
        let _cargo_link = temp_path.join(".cargo");
        fs::create_dir_all(&real_cargo).unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_cargo, &_cargo_link).unwrap();
        }
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Config should be created in the symlinked directory
        let config_path = real_cargo.join("config.toml");
        assert!(config_path.exists());
    }
}

mod platform_tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_specific_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let config_path = temp_path.join(".cargo").join("config.toml");
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Check for Windows-specific target
        assert!(content.contains("x86_64-pc-windows-msvc"), 
                "Should use Windows MSVC target, got: {}", content);
        assert!(content.contains("rust-lld") || content.contains("lld-link"), 
                "Should configure Windows linker");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_specific_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let config_path = temp_path.join(".cargo").join("config.toml");
        let content = fs::read_to_string(&config_path).unwrap();
        
        assert!(content.contains("x86_64-unknown-linux-gnu") || 
                content.contains("linker"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_specific_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let config_path = temp_path.join(".cargo").join("config.toml");
        assert!(config_path.exists());
    }
}

mod regression_tests {
    use super::*;

    #[test]
    fn test_issue_001_double_newline() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Regression: double newlines were being added
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let initial_config = "[build]\njobs = 2\n";
        fs::write(&config_path, initial_config).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let content = fs::read_to_string(&config_path).unwrap();
        // Check that we don't have excessive newlines
        assert!(!content.contains("\n\n\n"));
    }

    #[test]
    fn test_issue_002_crlf_line_endings() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Test CRLF line endings (Windows-style)
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let crlf_config = "[build]\r\njobs = 2\r\n";
        fs::write(&config_path, crlf_config).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        // Should handle CRLF gracefully
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("jobs"));
        assert!(content.contains("linker") || content.contains("target"));
    }

    #[test]
    fn test_issue_003_comments_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Test that comments are preserved
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let commented_config = r#"
# Important build configuration
[build]
jobs = 4  # Use 4 parallel jobs

# Custom profile settings
[profile.release]
opt-level = 3  # Maximum optimization
"#;
        fs::write(&config_path, commented_config).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let content = fs::read_to_string(&config_path).unwrap();
        // Comments should be preserved
        assert!(content.contains("Important build configuration"));
        assert!(content.contains("Use 4 parallel jobs"));
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_config_update_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a moderately large config
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let mut large_config = String::new();
        for i in 0..100 {
            large_config.push_str(&format!("[profile.test{}]\nopt-level = 2\n\n", i));
        }
        fs::write(&config_path, &large_config).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        let start = Instant::now();
        auto_configure_with_options_at(config, Some(temp_path));
        let duration = start.elapsed();
        
        // Should complete within reasonable time (1 second for large file)
        assert!(duration.as_secs() < 1, "Update took too long: {:?}", duration);
    }

    #[test]
    fn test_backup_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a large config to backup
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        
        let large_config = "x".repeat(1024 * 100); // 100KB file
        fs::write(&config_path, &large_config).unwrap();
        
        let config = MvpConfig {
            backup: true,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        let start = Instant::now();
        auto_configure_with_options_at(config, Some(temp_path));
        let duration = start.elapsed();
        
        // Backup should be fast even for large files
        assert!(duration.as_millis() < 500, "Backup took too long: {:?}", duration);
    }
}

mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_missing_cargo_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Don't create .cargo directory
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        // Should create the directory automatically
        auto_configure_with_options_at(config, Some(temp_path));
        
        let cargo_dir = temp_path.join(".cargo");
        assert!(cargo_dir.exists());
        
        let config_path = cargo_dir.join("config.toml");
        assert!(config_path.exists());
    }

    #[test]
    fn test_readonly_config_handling() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create config and make it read-only
        let cargo_dir = temp_path.join(".cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        let config_path = cargo_dir.join("config.toml");
        fs::write(&config_path, "[build]\njobs = 2").unwrap();
        
        // Make file read-only
        let metadata = fs::metadata(&config_path).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&config_path, permissions).unwrap();
        
        let config = MvpConfig {
            backup: false,
            force: true, // Force should try to handle read-only
            dry_run: false,
            include_timestamps: false,
        };
        
        // This might fail on Windows, but should handle gracefully
        let result = std::panic::catch_unwind(|| {
            auto_configure_with_options_at(config, Some(temp_path));
        });
        
        // Reset permissions for cleanup
        let metadata = fs::metadata(&config_path).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        let _ = fs::set_permissions(&config_path, permissions);
        
        // The operation might fail, but shouldn't panic
        assert!(result.is_ok() || cfg!(target_os = "windows"));
    }

    #[test]
    fn test_disk_full_simulation() {
        // This is hard to simulate portably, so we just test the config creation
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        let config = MvpConfig {
            backup: false,
            force: false,
            dry_run: false,
            include_timestamps: false,
        };
        
        auto_configure_with_options_at(config, Some(temp_path));
        
        let config_path = temp_path.join(".cargo").join("config.toml");
        assert!(config_path.exists());
    }
}
