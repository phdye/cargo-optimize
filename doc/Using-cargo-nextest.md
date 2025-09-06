# Using cargo-nextest with cargo-optimize

## Quick Start

cargo-nextest is now installed and configured for the project. Here's how to use it:

### Basic Commands

```bash
# Run all tests with nice output
cargo nextest run

# Run specific test file
cargo nextest run --test phase3_security

# Run tests matching a pattern
cargo nextest run test_fuzz

# Run with CI profile (includes retries)
cargo nextest run --profile ci

# Generate JUnit report
cargo nextest run --profile ci
# Report will be at: target/nextest/ci/junit.xml
```

### Comparison with cargo test

| Feature | cargo test | cargo nextest |
|---------|-----------|--------------|
| Output | Multiple summaries | Single unified summary |
| Timing | No per-test timing | Shows time for each test |
| Parallel | Yes, but basic | Advanced parallelism control |
| Retries | No | Yes, configurable |
| UI | Basic | Colorful, organized |
| JUnit | No | Yes, built-in |

### Example Output

**cargo test (confusing)**:
```
test result: ok. 5 passed; 0 failed; ...
test result: ok. 3 passed; 0 failed; ...
test result: ok. 1 passed; 0 failed; ...  # Looks like only 1 test!
```

**cargo nextest (clear)**:
```
Starting 13 tests across 1 binary
    PASS [0.039s] test_security_setup
    PASS [0.062s] test_chaos_recovery
    ... all tests shown ...
Summary [0.401s] 13 tests run: 13 passed ✅
```

### Advanced Usage

#### 1. Run tests in parallel with limited threads
```bash
cargo nextest run -j 4  # Use 4 threads
```

#### 2. Stop on first failure
```bash
cargo nextest run --fail-fast
```

#### 3. Run previously failed tests
```bash
cargo nextest run --failed
```

#### 4. Filter by test status
```bash
# After a test run with failures
cargo nextest run --status fail  # Re-run only failed tests
```

#### 5. Generate test list
```bash
cargo nextest list  # Shows all tests without running
```

### Configuration

The project has `.config/nextest.toml` configured with:
- **default profile**: For development with immediate output
- **ci profile**: For CI/CD with retries and JUnit output

### CI/CD Integration

For GitHub Actions:
```yaml
- name: Install nextest
  uses: taiki-e/install-action@nextest
  
- name: Run tests
  run: cargo nextest run --profile ci
  
- name: Upload test results
  uses: actions/upload-artifact@v3
  if: always()
  with:
    name: test-results
    path: target/nextest/ci/junit.xml
```

### Tips

1. **For daily development**: Just use `cargo nextest run`
2. **For debugging**: Add `--no-capture` to see println! output
3. **For CI**: Use `--profile ci` for retries and reports
4. **For speed**: Tests run in parallel by default

### Troubleshooting

**Issue**: "command not found: cargo-nextest"
**Fix**: Run `cargo install cargo-nextest --locked`

**Issue**: Tests seem slower
**Fix**: First run compiles; subsequent runs are fast

**Issue**: Can't see test output
**Fix**: Use `cargo nextest run --no-capture`

### Benefits for cargo-optimize

1. **Clear test counts**: No more confusion about how many tests ran
2. **Performance tracking**: See which tests are slow
3. **Better debugging**: Immediate failure output
4. **CI-ready**: JUnit reports work with all CI systems
5. **Parallel by default**: Tests run faster

---

That's it! cargo-nextest is ready to use. Just run `cargo nextest run` instead of `cargo test` for better output.
