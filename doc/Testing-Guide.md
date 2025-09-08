# Testing cargo-optimize

## Running Tests

cargo-optimize uses standard Rust testing infrastructure. No special tools are required.

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test config_safety_comprehensive

# Run with specific thread count
cargo test -- --test-threads=4
```

## Cross-Platform Compatibility

Tests run identically on all platforms:
- ✅ Windows
- ✅ Linux  
- ✅ macOS

## Test Isolation

All tests run in complete isolation:
- Each test creates its own temporary directory
- No tests modify the global current directory
- No tests can affect the actual `.cargo/config.toml`
- Tests can run in parallel without conflicts

## Implementation Details

The test suite uses a path-aware API that allows tests to specify which directory they're working in:
```rust
// Each test works in its own isolated directory
let temp_dir = TempDir::new().unwrap();
auto_configure_with_options_at(config, Some(temp_dir.path()));
```

This ensures complete test isolation and eliminates file conflicts on all platforms.

## Historical Note

Earlier versions required `cargo-nextest` on Windows due to file locking issues. This has been fixed by implementing proper test isolation, and nextest is no longer required.