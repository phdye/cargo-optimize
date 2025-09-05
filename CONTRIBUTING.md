# Contributing to cargo-optimize

Thank you for your interest in contributing to cargo-optimize! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct:
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing opinions and experiences

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

1. **Clear title and description**
2. **Steps to reproduce**
3. **Expected behavior**
4. **Actual behavior**
5. **System information** (OS, Rust version, cargo-optimize version)
6. **Relevant logs or error messages**

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

1. **Use case** - Why is this enhancement needed?
2. **Proposed solution** - How should it work?
3. **Alternatives considered** - What other solutions did you consider?
4. **Additional context** - Any other relevant information

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the code style** - Run `cargo fmt` and `cargo clippy`
3. **Add tests** - Ensure your code has appropriate test coverage
4. **Update documentation** - Keep README and docs in sync with code changes
5. **Write clear commit messages** - Use conventional commit format

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- Git

### Building

```bash
# Clone your fork
git clone https://github.com/your-username/cargo-optimize
cd cargo-optimize

# On Unix/Linux: Set correct file permissions
chmod +x fix-permissions.sh && ./fix-permissions.sh

# Build the project
cargo build

# Run tests
cargo test

# Run with verbose output for debugging
RUST_LOG=debug cargo run -- optimize --dry-run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_default_config

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test

# Run examples
cargo run --example basic
cargo run --example analyze
cargo run --example detect
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Test optimization on a real project
cargo run -- benchmark --iterations 3
```

## Code Style

### Rust Guidelines

- Follow standard Rust naming conventions
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy -- -D warnings`
- Prefer explicit error handling over `unwrap()`
- Add documentation comments for public APIs
- Include examples in documentation when helpful

### Commit Message Format

We use conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks

Example:
```
feat(linker): add support for mold linker on Linux

- Detect mold installation automatically
- Prefer mold over lld when available
- Add installation instructions

Closes #123
```

## Project Structure

```
cargo-optimize/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── bin/
│   │   └── cargo-optimize.rs  # CLI binary
│   ├── analyzer.rs      # Project analysis
│   ├── cache.rs         # Build cache management
│   ├── config.rs        # Configuration structures
│   ├── detector.rs      # Hardware/environment detection
│   ├── error.rs         # Error types
│   ├── linker.rs        # Linker configuration
│   ├── optimizer.rs     # Main optimization logic
│   ├── profile.rs       # Build profile management
│   └── utils.rs         # Utility functions
├── examples/            # Example usage
├── tests/              # Integration tests
└── doc/               # Additional documentation
```

## Adding New Features

### 1. New Optimization Technique

To add a new optimization:

1. Update `OptimizationFeature` enum in `src/config.rs`
2. Implement detection in `src/detector.rs` if needed
3. Add application logic in `src/optimizer.rs`
4. Update tests and documentation

### 2. New Platform Support

To add support for a new platform:

1. Update `OperatingSystem` enum in `src/detector.rs`
2. Add platform-specific logic in relevant modules
3. Test on the target platform
4. Update compatibility matrix in README

### 3. New Tool Integration

To integrate a new build tool:

1. Add detection logic in appropriate module
2. Implement configuration in `src/optimizer.rs`
3. Add installation instructions
4. Document the integration

## Testing Guidelines

### Unit Tests

- Place unit tests in the same file as the code
- Use `#[cfg(test)]` module
- Test edge cases and error conditions

### Integration Tests

- Place in `tests/` directory
- Test complete workflows
- Use temporary directories for file operations

### Manual Testing

Before submitting PR:

1. Test on a small project
2. Test on a medium workspace
3. Test with different optimization levels
4. Test dry-run mode
5. Test reset functionality

## Documentation

### Code Documentation

- All public APIs must have doc comments
- Include examples in doc comments
- Use `///` for item documentation
- Use `//!` for module documentation

### README Updates

Update README.md when:
- Adding new features
- Changing CLI interface
- Updating requirements
- Adding platform support

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create release PR
4. After merge, create GitHub release
5. Publish to crates.io: `cargo publish`

## Questions?

Feel free to:
- Open an issue for questions
- Start a discussion
- Reach out to maintainers

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project README

Thank you for contributing to cargo-optimize! 🚀
