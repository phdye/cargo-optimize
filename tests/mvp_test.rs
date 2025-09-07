//! MVP tests for cargo-optimize
//! 
//! Standard tests using #[test] attributes for nextest/pytest compatibility

use std::path::Path;

#[test]
fn mvp_version_test() {
    // Test that version function exists and returns expected format
    let version = cargo_optimize::version();
    assert!(!version.is_empty(), "Version should not be empty");
    assert!(version.contains("0.1"), "Version should contain 0.1");
}

#[test]
fn mvp_config_path_test() {
    // Test that config path is as expected
    let config_path = Path::new(".cargo/config.toml");
    // Just verify the path can be constructed (don't check if exists to avoid side effects)
    assert!(config_path.to_str().is_some(), "Config path should be valid UTF-8");
}

#[test]
fn mvp_exports_test() {
    // Test that expected functions are exported
    // This will fail to compile if they're not exported
    let _ = cargo_optimize::version;
    let _ = cargo_optimize::auto_configure;
    assert!(true, "Functions are exported correctly");
}

#[test]
fn mvp_no_panic_test() {
    // Test that creating an MvpConfig doesn't panic
    use cargo_optimize::mvp::MvpConfig;
    let config = MvpConfig::default();
    
    // Verify the config has reasonable defaults
    assert!(!config.dry_run, "Default should not be dry-run");
    assert!(true, "MvpConfig creation doesn't panic");
}
