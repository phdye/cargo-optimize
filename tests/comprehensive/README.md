# Comprehensive Test Suite for cargo-optimize

This directory contains a comprehensive testing framework designed to ensure the reliability, performance, and correctness of the cargo-optimize tool.

## Test Categories

The test suite is organized into the following categories:

### 1. **Unit Tests** (`unit_tests_*.rs`)
- `unit_tests_config.rs` - Tests for configuration management
- `unit_tests_detector.rs` - Tests for hardware/environment detection
- `unit_tests_analyzer.rs` - Tests for project analysis
- `unit_tests_optimizer.rs` - Tests for the optimization engine

### 2. **Integration Tests** (`integration_tests_comprehensive.rs`)
- End-to-end optimization workflows
- Real project analysis
- Workspace optimization
- Build script integration
- CI/CD environment detection

### 3. **Property-Based Tests** (`property_based_tests.rs`)
- Uses proptest for invariant verification
- Tests configuration serialization properties
- Validates optimization level consistency
- Ensures hardware detection consistency

### 4. **Fuzz Tests** (`fuzz_tests.rs`)
- Random input generation for robustness testing
- Config parsing with malformed inputs
- Path handling edge cases
- Concurrent access stress testing
- Memory allocation stress tests

### 5. **Performance Tests** (`performance_tests.rs`)
- Benchmarks for critical operations
- Configuration creation/serialization performance
- Hardware detection speed
- Project analysis performance
- Sustained load testing

### 6. **Golden Master Tests** (`golden_master_tests.rs`)
- Regression prevention through snapshot testing
- Default configuration preservation
- Optimization feature matrix validation
- CLI output format consistency
- Error message format stability

### 7. **Stress Tests** (`stress_tests.rs`)
- Large project handling (50+ crates, 1000+ files)
- Concurrent access patterns
- Memory allocation stress
- Rapid configuration changes
- File system operation stress

### 8. **Boundary Value Tests** (`boundary_tests.rs`)
- Empty/minimal projects
- Maximum parameter values (usize::MAX)
- Zero values and edge cases
- Unicode in paths and strings
- Float precision edge cases

### 9. **Regression Tests** (`regression_tests.rs`)
- Tests for specific fixed issues
- Config serialization bugs
- Workspace path handling
- Memory leak prevention
- Race condition fixes

### 10. **CI/CD Tests** (`ci_cd_tests.rs`)
- GitHub Actions integration
- GitLab CI compatibility
- Jenkins pipeline support
- CI-specific optimizations

## Test Runner

The test suite includes a sophisticated test runner (`test_runner_comprehensive.rs`) that provides:

- **Parallel execution** for faster test runs
- **Category filtering** to run specific test types
- **Multiple report formats** (Text, JSON, JUnit, HTML)
- **Fail-fast mode** for CI environments
- **Verbose output** for debugging
- **Environment validation** before test execution

## Running Tests

### Run all tests:
```bash
cargo test --bin test_main
```

### Run specific categories:
```bash
cargo test --bin test_main --categories unit,integration
```

### Run with verbose output:
```bash
cargo test --bin test_main --verbose
```

### Generate JUnit report for CI:
```bash
cargo test --bin test_main --report junit
```

### Run in CI mode:
```bash
cargo test --bin test_main --categories all --fail-fast --report junit
```

### List available categories:
```bash
cargo test --bin test_main --list-categories
```

## Test Coverage

The comprehensive test suite covers:

- ✅ **Configuration Management**: Default values, builder pattern, serialization
- ✅ **Hardware Detection**: CPU, memory, OS detection across platforms
- ✅ **Project Analysis**: Code statistics, dependency analysis, complexity scoring
- ✅ **Optimization Engine**: All optimization levels, feature combinations
- ✅ **Error Handling**: Graceful failure, permission errors, malformed input
- ✅ **Performance**: Sub-second analysis for large projects
- ✅ **Concurrency**: Thread-safe operations, race condition prevention
- ✅ **Platform Support**: Windows, Linux, macOS specific tests
- ✅ **Edge Cases**: Unicode, extreme values, empty projects
- ✅ **Regression Prevention**: Golden masters, snapshot testing

## Test Infrastructure

### Key Features:
- **Modular design** - Each test category in its own module
- **Property-based testing** - Automatic test case generation
- **Fuzzing** - Robustness against random inputs
- **Performance benchmarks** - Track performance regressions
- **Golden masters** - Prevent output format regressions
- **Comprehensive assertions** - Using pretty_assertions for clear failures

### Dependencies:
- `criterion` - Benchmarking framework
- `proptest` - Property-based testing
- `arbitrary` - Fuzz input generation
- `fastrand` - Fast random number generation
- `tempfile` - Temporary directory management
- `pretty_assertions` - Better assertion output

## Continuous Improvement

The test suite is designed to grow with the project:

1. **Add regression tests** for any bugs found in production
2. **Update golden masters** when intentional changes are made
3. **Add property tests** for new invariants
4. **Benchmark new features** to prevent performance regressions
5. **Fuzz test** new input parsing code

## Test Metrics

Current test coverage includes:
- **300+ individual test cases**
- **10 test categories**
- **20+ modules tested**
- **Support for 3 major platforms**
- **5 report formats**

## Contributing

When adding new features to cargo-optimize:

1. Add unit tests in the appropriate `unit_tests_*.rs` file
2. Add integration tests for end-to-end workflows
3. Consider adding property-based tests for invariants
4. Add fuzzing for input parsing
5. Add performance benchmarks for critical paths
6. Update golden masters if output formats change
7. Add regression tests for any bugs fixed

## License

This test suite is part of the cargo-optimize project and is dual-licensed under MIT OR Apache-2.0.
