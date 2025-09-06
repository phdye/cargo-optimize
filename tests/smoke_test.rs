// Smoke test for cargo-optimize MVP
// Used for quick validation during rollbacks and checkpoints

#[test]
fn smoke_test_basic_functionality() {
    // Test 1: Can call auto_configure without panic
    cargo_optimize::auto_configure();
}

#[test]
fn smoke_test_mvp_module_accessible() {
    // Test 2: MVP module is accessible
    cargo_optimize::mvp::auto_configure_mvp();
}

#[test]
fn smoke_test_config_struct() {
    // Test 3: Config struct can be created
    let config = cargo_optimize::mvp::MvpConfig::default();
    assert!(config.backup);
    assert!(!config.force);
    assert!(!config.dry_run);
}

#[test]
fn smoke_test_version() {
    // Test 4: Version is accessible
    let version = cargo_optimize::version();
    assert_eq!(version, "0.1.0");
}

#[test]
fn smoke_test_dry_run() {
    // Test 5: Dry run doesn't modify anything
    use cargo_optimize::mvp::{MvpConfig, auto_configure_with_options};
    
    let config = MvpConfig {
        backup: true,
        force: false,
        dry_run: true,
    };
    
    // Should complete without actually changing files
    auto_configure_with_options(config);
}

// Run smoke tests with: cargo test --test smoke_test
// Expected: All 5 tests pass in <1 second
// Use for: Quick validation during rollbacks
