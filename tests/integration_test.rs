#[test]
fn test_mvp_runs_without_panic() {
    // The simplest possible test - just verify it doesn't panic
    cargo_optimize::auto_configure();
}

#[test]
fn test_mvp_module_exists() {
    // Verify the MVP module is accessible
    cargo_optimize::mvp::auto_configure_mvp();
}
