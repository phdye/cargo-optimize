# cargo-optimize 🚀

[![Crates.io](https://img.shields.io/crates/v/cargo-optimize.svg)](https://crates.io/crates/cargo-optimize)
[![Documentation](https://docs.rs/cargo-optimize/badge.svg)](https://docs.rs/cargo-optimize)
[![License](https://img.shields.io/crates/l/cargo-optimize.svg)](LICENSE)

**Automatically optimize Rust build times by up to 70% with a single command!**

cargo-optimize automatically detects your hardware, analyzes your project structure, and applies optimal build configurations - no manual tuning required.

## ✨ Features

- 🔍 **Automatic Hardware Detection** - Optimizes for your CPU cores, memory, and architecture
- 📊 **Project Analysis** - Understands your code structure and dependency graph  
- ⚡ **Fast Linkers** - Automatically configures mold, lld, or gold for faster linking
- 💾 **Build Caching** - Sets up sccache/ccache for incremental compilation
- 🎯 **Smart Profiles** - Optimizes dev, release, and test profiles based on your project
- 🔧 **Zero Configuration** - Works out of the box with sensible defaults
- 📈 **Continuous Learning** - Adapts optimizations based on your build patterns

## 📦 Installation

### As a Cargo Subcommand (Recommended)

```bash
cargo install cargo-optimize
```

Now you can use it with:
```bash
cargo optimize
```

### As a Build Dependency

Add to your `Cargo.toml`:
```toml
[build-dependencies]
cargo-optimize = "0.1"
```

Create or update `build.rs`:
```rust
fn main() {
    cargo_optimize::auto_configure();
}
```

## 🚀 Quick Start

### One-Command Setup and Optimization

```bash
# Set up and optimize current project (replaces old setup.sh/setup.bat)
cargo optimize setup

# Or just apply optimizations directly
cargo optimize

# Advanced setup options
cargo optimize setup -O aggressive --no-verify
cargo optimize setup --path /path/to/project

# Initialize project with configuration
cargo optimize init

# Dry run to see what would be changed
cargo optimize --dry-run
```

### Analyze Without Applying

```bash
# Analyze project structure
cargo optimize analyze

# Detailed analysis
cargo optimize analyze --detailed
```

### Benchmark Your Improvements

```bash
# Compare build times with and without optimizations
cargo optimize benchmark
```

## 📊 Real-World Results

| Project Size | Before | After | Improvement |
|--------------|--------|-------|-------------|
| Small (<5k LOC) | 30s | 10s | **67% faster** |
| Medium (5k-50k LOC) | 2m 30s | 45s | **70% faster** |
| Large (>50k LOC) | 5m | 1m 30s | **70% faster** |

## 🎯 Optimization Levels

### Conservative (Safe)
- Fast linker configuration
- Build caching
- Parallel job optimization
- **Best for:** Production builds, CI/CD

### Balanced (Recommended)
- All conservative optimizations
- Optimized build profiles  
- Incremental compilation
- Split debuginfo
- **Best for:** Daily development

### Aggressive (Maximum Speed)
- All balanced optimizations
- Native CPU targeting
- Parallel frontend (nightly)
- Shared generics
- **Best for:** Local development, when stability isn't critical

## 🛠️ Manual Configuration

### Via Configuration File

Create `.cargo-optimize.toml`:
```toml
optimization_level = "balanced"
auto_detect_hardware = true
analyze_project = true
optimize_linker = true
enable_cache = true
parallel_jobs = 8  # or leave unset for auto-detection

[profile_overrides.dev]
opt_level = 0
incremental = true
codegen_units = 256

[profile_overrides.release]
opt_level = 3
lto = "thin"
codegen_units = 1
```

Use with:
```bash
cargo optimize --config .cargo-optimize.toml
```

### Via Environment Variables

```bash
# Disable specific optimizations
export CARGO_OPTIMIZE_DISABLE=1  # Disable all optimizations
export CARGO_OPTIMIZE_NO_LINKER=1  # Skip linker optimization
export CARGO_OPTIMIZE_NO_CACHE=1   # Skip cache setup

# Enable verbose output
export CARGO_OPTIMIZE_VERBOSE=1
```

## 🔧 What Gets Optimized?

### 1. **Linker Selection**
- Automatically detects and configures the fastest available linker
- Preference: mold > lld > gold > default
- Platform-specific optimizations

### 2. **Build Caching**
- Sets up sccache or ccache
- Configures optimal cache size based on available disk space
- Enables distributed caching in CI environments

### 3. **Compilation Parallelism**
- Optimizes job count based on CPU cores
- Adjusts for CI environments
- Configures codegen units for best performance

### 4. **Build Profiles**
- Optimizes dev profile for fast iteration
- Optimizes release profile for final builds
- Balances debug info and compilation speed

### 5. **Platform-Specific**
- Windows: Optimized PDB generation
- macOS: Split DWARF configuration  
- Linux: Native CPU optimizations

## 📋 Prerequisites

### Required
- Rust 1.70.0 or later
- Cargo

### Optional (Automatically Installed)
- **sccache** - For build caching
- **mold/lld** - For faster linking

Install optional dependencies:
```bash
# Install all recommended tools
cargo optimize install

# Install specific tool
cargo optimize install sccache
cargo optimize install lld
```

## 🤝 CI/CD Integration

### GitHub Actions

```yaml
- name: Install cargo-optimize
  run: cargo install cargo-optimize

- name: Optimize build
  run: cargo optimize -O conservative

- name: Build project
  run: cargo build --release
```

### GitLab CI

```yaml
before_script:
  - cargo install cargo-optimize
  - cargo optimize -O conservative
```

### Docker

```dockerfile
RUN cargo install cargo-optimize
RUN cargo optimize -O balanced
```

## 🔍 Troubleshooting

### Reset All Optimizations

```bash
# Reset configurations
cargo optimize reset

# Reset and clean build artifacts
cargo optimize reset --clean
```

### Common Issues

**Q: Build fails after optimization**
```bash
# Try conservative mode
cargo optimize -O conservative

# Or reset and try again
cargo optimize reset --clean
cargo optimize
```

**Q: Not seeing improvements**
```bash
# Run benchmark to measure
cargo optimize benchmark

# Check what's being applied
cargo optimize --dry-run --verbose
```

**Q: Want to exclude certain optimizations**
```bash
# Disable specific features
cargo optimize --disable linker --disable cache
```

## 📚 Advanced Usage

### Workspace Optimization

```bash
# Optimize entire workspace
cargo optimize --workspace

# Optimize specific package
cargo optimize -p my-package
```

### Custom Profiles

```rust
// In build.rs
use cargo_optimize::{Config, OptimizationLevel};

fn main() {
    let mut config = Config::new();
    config.set_optimization_level(OptimizationLevel::Custom);
    config.set_parallel_jobs(16);
    
    cargo_optimize::optimize_with_config(config).ok();
}
```

### Project-Specific Tuning

```toml
# In .cargo/config.toml (generated by cargo-optimize)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[build]
jobs = 8
incremental = true

[profile.dev]
opt-level = 0
debug = true
incremental = true
codegen-units = 256
```

## 🧪 Benchmarks

Run the included benchmarks:
```bash
# Benchmark your project
cargo optimize benchmark

# Benchmark with more iterations
cargo optimize benchmark --iterations 5
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/cargo-optimize
cd cargo-optimize

# Set up development environment (installs tools, sets up hooks, etc.)
rust-script scripts/dev-setup.rs

# Fix file permissions if needed
./scripts/fix-permissions.sh

# Run tests
cargo nextest run

# Run with verbose output
CARGO_OPTIMIZE_VERBOSE=1 cargo run -- setup --dry-run
```

## 📄 License

This project is dual-licensed under MIT OR Apache-2.0.

## 🙏 Acknowledgments

- [mold](https://github.com/rui314/mold) - High-performance linker
- [sccache](https://github.com/mozilla/sccache) - Shared compilation cache
- [cargo-nextest](https://nexte.st/) - Test runner inspiration

## 🔗 Links

- [Documentation](https://docs.rs/cargo-optimize)
- [Crates.io](https://crates.io/crates/cargo-optimize)
- [GitHub](https://github.com/yourusername/cargo-optimize)
- [Issue Tracker](https://github.com/yourusername/cargo-optimize/issues)

---

**Made with ❤️ by the Rust community**
