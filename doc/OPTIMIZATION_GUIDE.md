# Rust Build Optimization Guide

This document explains the optimization techniques used by cargo-optimize and how they work.

## Table of Contents

1. [Linker Optimizations](#linker-optimizations)
2. [Build Caching](#build-caching)
3. [Compilation Parallelism](#compilation-parallelism)
4. [Build Profiles](#build-profiles)
5. [Platform-Specific Optimizations](#platform-specific-optimizations)
6. [CI/CD Optimizations](#cicd-optimizations)
7. [Manual Tuning Guide](#manual-tuning-guide)

## Linker Optimizations

The linker is often the bottleneck in Rust builds, especially for large projects. cargo-optimize automatically selects and configures the fastest available linker.

### Linker Comparison

| Linker | Platform | Speed | Notes |
|--------|----------|-------|-------|
| **mold** | Linux | Fastest (10-50x faster) | Best choice for Linux |
| **lld** | All | Very fast (5-10x faster) | Good cross-platform option |
| **gold** | Linux | Fast (2-3x faster) | Older but reliable |
| **link.exe** | Windows | Default | Standard MSVC linker |
| **ld** | Unix | Default | Standard system linker |

### How It Works

1. **Detection**: cargo-optimize checks which linkers are installed
2. **Selection**: Chooses the fastest available linker
3. **Configuration**: Sets up rustc flags and cargo configuration
4. **Verification**: Tests that the linker works correctly

### Manual Configuration

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

## Build Caching

Build caching can dramatically reduce compilation time by reusing previously compiled artifacts.

### Cache Systems

#### sccache (Recommended)
- Rust-aware caching
- Supports distributed caching
- Works with multiple compilers
- Cloud storage backend support

#### ccache
- General-purpose compiler cache
- Mature and stable
- Good for mixed C/C++/Rust projects

### Cache Configuration

```bash
# Environment variables set by cargo-optimize
export RUSTC_WRAPPER=sccache
export SCCACHE_CACHE_SIZE=10G
export SCCACHE_DIR=~/.cache/sccache
```

### Cache Statistics

Monitor cache effectiveness:
```bash
cargo optimize stats
# Shows hit rate, cache size, etc.
```

## Compilation Parallelism

Rust can compile multiple crates and modules in parallel. cargo-optimize optimizes parallelism based on your hardware.

### Key Settings

#### Codegen Units
- **Dev builds**: 256 (maximum parallelism)
- **Release builds**: 1-16 (better optimization)
- Trade-off: Compilation speed vs. runtime performance

#### Parallel Jobs
- Formula: `physical_cores * 0.75`
- Leaves headroom for system responsiveness
- CI environments: Limited to 2-4 jobs

### Configuration

```toml
[profile.dev]
codegen-units = 256  # Fast compilation

[profile.release]
codegen-units = 1    # Maximum optimization
```

## Build Profiles

cargo-optimize creates optimized profiles for different scenarios:

### Development Profile
Optimized for fast iteration:
```toml
[profile.dev]
opt-level = 0           # No optimization
debug = true            # Include debug info
incremental = true      # Enable incremental compilation
codegen-units = 256     # Maximum parallelism
split-debuginfo = "unpacked"  # Faster linking
```

### Release Profile
Optimized for runtime performance:
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "thin"           # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Remove symbols
panic = "abort"        # Smaller binary
```

### Test Profile
Balanced for test execution:
```toml
[profile.test]
opt-level = 0          # Fast compilation
debug = true           # Debugging support
incremental = true     # Reuse artifacts
```

### Custom Profiles
Create profiles for specific needs:
```toml
[profile.ci]
inherits = "release"
lto = false            # Skip LTO in CI
incremental = false    # Clean builds
```

## Platform-Specific Optimizations

### Linux
- **Linker**: mold > lld > gold
- **Allocator**: jemalloc for better performance
- **Debuginfo**: Split into separate files
- **Native CPU**: `-C target-cpu=native`

### macOS
- **Linker**: lld (mold not yet supported)
- **Debuginfo**: Unpacked DWARF
- **Framework linking**: Optimized for system frameworks
- **Universal binaries**: Separate arch compilation

### Windows
- **Linker**: lld or link.exe
- **PDB generation**: Optimized settings
- **MSVC flags**: `/OPT:REF /OPT:ICF`
- **Debug symbols**: Separate PDB files

## CI/CD Optimizations

### GitHub Actions
```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

- run: cargo optimize -O conservative
```

### Docker
```dockerfile
# Multi-stage build with caching
FROM rust:latest as builder
RUN cargo install cargo-optimize sccache
ENV RUSTC_WRAPPER=sccache
COPY . .
RUN cargo optimize && cargo build --release
```

### Best Practices
1. Use conservative optimization level
2. Disable incremental compilation
3. Cache dependencies separately
4. Limit parallel jobs (2-4)
5. Use consistent toolchain versions

## Manual Tuning Guide

### Measuring Performance

1. **Baseline measurement**:
```bash
cargo clean
time cargo build
```

2. **With optimizations**:
```bash
cargo optimize
cargo clean
time cargo build
```

3. **Detailed analysis**:
```bash
cargo optimize benchmark --iterations 5
```

### Profiling Build Times

Use `cargo build --timings` to generate a build timeline:
```bash
cargo clean
cargo build --timings
# Opens target/cargo-timings/cargo-timing.html
```

### Common Issues and Solutions

#### Out of Memory
- Reduce codegen units
- Limit parallel jobs
- Disable LTO

#### Linker Errors
- Try a different linker
- Check for missing dependencies
- Verify toolchain compatibility

#### Cache Misses
- Check cache configuration
- Verify RUSTC_WRAPPER is set
- Monitor cache statistics

### Advanced Techniques

#### Custom RUSTFLAGS
```bash
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=thin"
```

#### Per-crate optimization
```toml
[profile.release.package."my-hot-crate"]
opt-level = 3
codegen-units = 1
```

#### Build pipelining (Nightly)
```bash
CARGO_BUILD_PIPELINING=true cargo +nightly build -Z timings
```

#### Parallel frontend (Nightly)
```bash
RUSTFLAGS="-Z threads=0" cargo +nightly build
```

## Optimization Decision Tree

```
Start
│
├─ Project Size?
│  ├─ Small (<5k LOC)
│  │  └─ Conservative optimizations
│  │     - Fast linker
│  │     - Basic caching
│  │     - High parallelism
│  │
│  ├─ Medium (5k-50k LOC)
│  │  └─ Balanced optimizations
│  │     - Fast linker
│  │     - Full caching
│  │     - Optimized profiles
│  │     - Split debuginfo
│  │
│  └─ Large (>50k LOC)
│     └─ Aggressive optimizations
│        - Fastest linker
│        - Distributed caching
│        - Native CPU targeting
│        - Workspace splitting
│
├─ Environment?
│  ├─ Local Development
│  │  └─ Maximum parallelism
│  │
│  ├─ CI/CD
│  │  └─ Limited parallelism
│  │     - No incremental
│  │     - Conservative settings
│  │
│  └─ Production Build
│     └─ Maximum optimization
│        - Full LTO
│        - Single codegen unit
│        - Strip symbols
│
└─ Platform-specific adjustments
```

## Performance Metrics

Typical improvements with cargo-optimize:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Clean build | 150s | 45s | 70% faster |
| Incremental build | 30s | 8s | 73% faster |
| Test execution | 45s | 12s | 73% faster |
| Link time | 20s | 2s | 90% faster |
| Cache hit rate | 0% | 85% | N/A |

## Further Reading

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [mold linker](https://github.com/rui314/mold)
- [sccache](https://github.com/mozilla/sccache)
- [Rust Compilation Model](https://rustc-dev-guide.rust-lang.org/overview.html)

## Contributing

Have a new optimization technique? Please contribute! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
