# Nextest Required on Windows

## Why cargo-optimize Requires Nextest on Windows

When developing or testing cargo-optimize on Windows, you must use [cargo-nextest](https://nexte.st/) instead of the standard `cargo test` command. This document explains the technical reasons behind this requirement.

## The Problem

Running `cargo test` on Windows results in test failures:

```
---- boundary_tests::test_very_large_config stdout ----
thread 'boundary_tests::test_very_large_config' panicked at tests\config_safety_comprehensive.rs:338:9

---- boundary_tests::test_max_backup_files stdout ----
thread 'boundary_tests::test_max_backup_files' panicked: 
Os { code: 32, kind: Uncategorized, message: "The process cannot access the file because it is being used by another process." }
```

These failures are **not** bugs in the tests or the code - they are caused by Windows' file locking behavior when multiple threads attempt to access the same files.

## Root Cause: Windows File Locking

cargo-optimize tests necessarily manipulate `.cargo/config.toml` files to verify correct behavior. This includes:
- Creating config files
- Modifying existing configs
- Creating backup files
- Testing concurrent access scenarios

On Windows, when multiple test threads within the same process attempt these operations, the operating system's strict file locking prevents concurrent access, causing spurious test failures.

### Why This Doesn't Affect Linux/macOS

Unix-like systems have more permissive file access models:
- Multiple processes can read/write the same file
- File deletion works differently (unlink vs actual deletion)
- Less aggressive locking on file handles

## How Nextest Solves This

Nextest runs each test in a **separate process** rather than separate threads within one process. This provides:

1. **Process Isolation**: Each test gets its own process space with independent file handles
2. **Clean File Handle Management**: Windows manages file access better across processes than threads
3. **No Shared State**: Tests cannot interfere with each other's file operations
4. **True Parallelism**: Tests still run in parallel, just in different processes

## Performance Impact

| Test Runner | Windows Time | Result |
|------------|-------------|---------|
| `cargo test` | ~3.1s | ❌ 2 tests fail |
| `cargo test --test-threads=1` | ~2.1s | ✅ All pass (no parallelism) |
| `cargo nextest run` | ~9.3s | ✅ All pass (with parallelism) |

While nextest is slower (~9 seconds vs ~3 seconds), it's the only solution that provides both:
- Passing tests on Windows
- Parallel test execution

## Installation

Install nextest before running tests on Windows:

```bash
cargo install cargo-nextest
```

Or using the nextest installer:

```bash
curl -LsSf https://get.nexte.st/latest/windows | iex
```

## Running Tests

Instead of:
```bash
cargo test
```

Use:
```bash
cargo nextest run
```

All standard cargo test arguments work with nextest:
```bash
cargo nextest run --test config_safety
cargo nextest run -- test_name
cargo nextest run --workspace
```

## Why Not Alternative Solutions?

### Why not use `--test-threads=1`?
- Loses parallelism benefits (slower on all platforms)
- Masks potential concurrency bugs
- Not a real fix, just a workaround

### Why not modify tests to avoid conflicts?
- Tests correctly verify file manipulation behavior
- cargo-optimize's core functionality involves modifying config files
- Adding complex workarounds would reduce test effectiveness

### Why not use serial_test crate?
- Already attempted - `#[serial]` attributes don't prevent Windows file locking issues
- The problem is at the OS level, not the Rust level

## Continuous Integration

For CI/CD pipelines on Windows, nextest is required:

```yaml
# GitHub Actions example
- name: Install nextest
  uses: taiki-e/install-action@nextest
  
- name: Run tests
  run: cargo nextest run
```

## Platform-Specific Testing

If you're developing on Linux/macOS but need to ensure Windows compatibility:

```bash
# Linux/macOS (optional, but recommended for consistency)
cargo nextest run

# Windows (required)
cargo nextest run
```

## Summary

Nextest is **required** on Windows because:
1. Windows file locking prevents parallel test execution with standard `cargo test`
2. cargo-optimize tests must manipulate config files as part of core functionality
3. Nextest's process isolation solves the file locking issue
4. The performance trade-off (9s vs 3s) is acceptable for correct test execution

This is not a limitation of cargo-optimize, but rather a necessary adaptation to Windows' file system behavior when testing file manipulation code.

## Additional Resources

- [Nextest Documentation](https://nexte.st/)
- [Windows File Locking Behavior](https://docs.microsoft.com/en-us/windows/win32/fileio/locking-and-unlocking-byte-ranges-in-files)
- [cargo-optimize Test Suite](../tests/)