# cargo-optimize Customization Guide

## Overview
While cargo-optimize aims for **zero-configuration optimization**, power users should have full control over every aspect. Here's a comprehensive breakdown of all customizable features and how users can configure them.

## 🎯 Percentage Support
Many numeric configurations support **percentage notation** where a maximum value can be determined:

```toml
# Examples of percentage support
[parallel]
jobs = "75%"  # 75% of CPU cores (e.g., 12 cores on 16-core system)

[cache]
max_size = "10%"  # 10% of available disk space

[budget.memory]
max_build_memory = "80%"  # 80% of system RAM
```

## Configuration Hierarchy

```toml
# 1. Project-level: cargo-optimize.toml (highest priority)
# 2. User-level: ~/.cargo-optimize/config.toml
# 3. System-level: /etc/cargo-optimize/config.toml
# 4. Defaults: Built-in optimizations (lowest priority)
```

---

## 1. Linker Configuration (Current MVP) ✅ 

### What Users Might Customize
- **Linker choice** - Override auto-detection
- **Linker flags** - Additional optimization flags
- **Per-target linkers** - Different linkers for different targets
- **Fallback behavior** - What to do if preferred linker unavailable

### How to Customize
```toml
# cargo-optimize.toml
[linker]
# Force specific linker (override auto-detection)
preferred = "mold"  # Options: "mold", "lld", "gold", "default", "auto"

# Per-platform overrides
[linker.windows]
preferred = "lld"
flags = ["/DEBUG:NONE", "/OPT:REF"]

[linker.linux]
preferred = "mold"
flags = ["--threads=8", "--compress-debug-sections=zlib"]

[linker.macos]
preferred = "lld"
flags = ["-Wl,-dead_strip"]

# Fallback strategy
[linker.fallback]
strategy = "next-best"  # or "error", "default"
order = ["mold", "lld", "gold", "default"]
```

---

## 2. Build Parallelization ✅

### What Users Might Customize
- **Job count** - Override CPU core detection
- **Memory limits** - Prevent OOM on large projects
- **Per-profile settings** - Different parallelization for dev/release

### How to Customize
```toml
[parallel]
# Percentage support!
jobs = "75%"  # 75% of CPU cores
# OR absolute values
jobs = 16  # Fixed count
# OR presets
jobs = "auto"  # 100% of cores
jobs = "conservative"  # 50% of cores
jobs = "aggressive"  # 150% of cores (hyperthreading)

# Memory management (percentage of system RAM)
max_memory_per_job = "10%"  # 10% of RAM per job
# OR absolute values
max_memory_per_job = "2GB"

# Per-profile overrides
[parallel.profile.dev]
jobs = "25%"  # Light load for development

[parallel.profile.release]
jobs = "100%"  # Use all cores for release builds

[parallel.profile.test]
jobs = "50%"  # Leave cores for test execution
```

---

## 3. Caching Strategy ✅

### What Users Might Customize
- **Cache backend** - sccache vs ccache
- **Cache location** - Local paths
- **Cache size limits** - Disk space management

### How to Customize
```toml
[cache]
backend = "sccache"  # or "ccache", "none", "auto"
location = "/fast/nvme/rust-cache"  # Override default

# Size management (percentage support!)
max_size = "5%"  # 5% of disk space
# OR absolute values
max_size = "50GB"

cleanup_threshold = "90%"  # Clean when 90% full
keep_recent = "7d"  # Keep items accessed within 7 days

# Basic CI cache settings
[cache.ci]
enabled = true
key_prefix = "${CI_COMMIT_REF_NAME}"
```

> **Note**: Advanced features like remote cache encryption and distributed caching are tracked in Future-Features.md

---

## 4. Optimization Profiles ✅

### What Users Might Customize
- **Optimization level** - Balance speed vs size
- **Debug info** - Development vs production needs
- **LTO settings** - Link-time optimization
- **Codegen options** - Basic optimizations

### How to Customize
```toml
[optimize.profile.dev]
level = 1  # 0-3, overrides Cargo.toml
lto = false
debug = 2  # Full debug info
incremental = true
split_debuginfo = "packed"

[optimize.profile.release]
level = 3
lto = "thin"  # or "fat", false
debug = 0
strip = "symbols"
panic = "abort"
codegen_units = 1  # Maximum optimization

[optimize.profile.bench]
inherit = "release"
debug = 1  # Some debug info for profiling

# Custom profiles
[optimize.profile.production]
inherit = "release"
strip = "all"
opt_level = "z"  # Size optimization
```

---

## 5. Platform-Specific Optimizations ✅

### What Users Might Customize
- **Target CPU** - Native vs generic
- **Basic platform tweaks** - Simple OS optimizations

### How to Customize
```toml
[platform]
# CPU targeting
target_cpu = "native"  # or "generic", "x86-64-v3"

[platform.windows]
exclude_from_defender = true  # Auto-configure Windows Defender

[platform.linux]
use_tmpfs = true  # Build on tmpfs if available
tmpfs_size = "50%"  # 50% of available RAM
# OR absolute value
tmpfs_size = "8GB"

[platform.macos]
deployment_target = "11.0"
```

> **Note**: Advanced platform features like fat binary optimization are in Future-Features.md

---

## 6. Dependency Management ✅

### What Users Might Customize
- **Feature detection** - Analyze unused features
- **Bottleneck warnings** - Alert on slow dependencies

### How to Customize
```toml
[dependencies]
# Feature optimization
auto_minimize_features = true
feature_detection_depth = 2  # How deep to analyze

# Bottleneck handling
[dependencies.bottlenecks]
warn_threshold = "30s"  # Warn if dependency takes >30s
error_threshold = "2m"   # Error if >2 minutes
```

> **Note**: Automatic dependency replacement and complex refactoring suggestions are in Future-Features.md

---

## 7. CI/CD Integration ✅

### What Users Might Customize
- **CI detection** - Override auto-detection
- **Basic cache strategies** - Simple CI caching

### How to Customize
```toml
[ci]
provider = "github"  # or "gitlab", "jenkins", "auto"
enabled = true

[ci.github]
cache_key = "${GITHUB_SHA}"
restore_keys = ["${GITHUB_REF}", "main"]

[ci.matrix]
# Per-matrix-job optimization
"ubuntu-latest" = { linker = "mold" }
"windows-latest" = { linker = "lld" }
"macos-latest" = { linker = "default" }
```

---

## 8. Build Analytics & Monitoring ✅

### What Users Might Customize
- **Metrics collection** - What to track
- **Basic reporting** - Simple metrics output
- **Regression detection** - Thresholds and alerts

### How to Customize
```toml
[analytics]
enabled = true
track_history = true
history_retention = "90d"

[analytics.metrics]
track = ["build_time", "cache_hits", "dependency_time"]
export_format = "json"  # or "csv"
export_path = "target/metrics/"

[analytics.regression]
enabled = true
threshold = "20%"  # Warn if 20% slower than baseline
# OR absolute time
threshold = "30s"  # Warn if 30s slower
baseline = "main"  # Compare against main branch

[analytics.dashboard]
enabled = true
port = 3000
```

---

## 9. Performance Budgets ✅

### What Users Might Customize
- **Build time limits** - Maximum acceptable times
- **Binary size limits** - Output size constraints
- **Memory usage** - Build memory limits

### How to Customize
```toml
[budget]
enforce = true  # Fail if budgets exceeded

[budget.time]
clean_build = "5m"
incremental = "30s"
test = "2m"
action = "warn"  # or "error"

[budget.size]
release_binary = "10MB"
debug_binary = "100MB"

[budget.memory]
max_build_memory = "80%"  # 80% of system RAM
# OR absolute value
max_build_memory = "8GB"
max_per_crate = "25%"  # 25% of max_build_memory
```

---

## 10. Logging & Diagnostics ✅

### What Users Might Customize
- **Log level** - Verbosity
- **Output format** - Human vs machine readable

### How to Customize
```toml
[logging]
level = "info"  # or "debug", "warn", "error", "trace"
format = "pretty"  # or "json", "compact"
file = "cargo-optimize.log"
color = true

[logging.filters]
# Fine-grained logging control
"cargo_optimize::linker" = "debug"
"cargo_optimize::cache" = "warn"
```

---

## Global Enable/Disable ✅

```toml
# Quick disable without removing config
[global]
enabled = true  # Set to false to disable all optimizations
dry_run = false  # Show what would be done without doing it
backup_before_change = true
```

---

## Environment Variables ✅

All settings can be overridden via environment variables:

```bash
# Override any setting
CARGO_OPTIMIZE_LINKER_PREFERRED=mold
CARGO_OPTIMIZE_PARALLEL_JOBS=75%  # Percentage support!
CARGO_OPTIMIZE_CACHE_MAX_SIZE=10%  # 10% of disk
CARGO_OPTIMIZE_ENABLED=false  # Disable entirely

# Special variables
CARGO_OPTIMIZE_CONFIG=/path/to/config.toml
CARGO_OPTIMIZE_PROFILE=production
```

---

## Command Line Overrides ✅

```bash
# Override any configuration
cargo optimize --linker=mold --jobs=75% --cache-size=10%

# Profiles
cargo optimize --profile=ci

# Disable features
cargo optimize --no-cache --no-parallel

# Dry run
cargo optimize --dry-run
```

---

## Advanced Features

The following advanced customization options are planned but require significant implementation effort. They are tracked in Future-Features.md:

### 🔮 Future Customizations
- **Profile-Guided Optimization (PGO)** - Automatic profiling with test suites
- **Binary Optimization (BOLT)** - Post-link binary optimization
- **Speculative Compilation** - ML-based predictive building
- **Workspace Splitting** - Automatic monolith detection and refactoring
- **Dependency Replacement** - Smart dependency substitution
- **Remote Cache Encryption** - Secure team cache sharing
- **Organization Profile Syncing** - Git-based config distribution
- **Editor Background Compilation** - IDE integration
- **Corruption Auto-Detection** - Integrity checking and recovery
- **Docker Multi-Stage Optimization** - cargo-chef patterns
- **Universal Binary Optimization** - macOS fat binary handling

See Future-Features.md for details on these advanced features.

---

## Summary of Percentage Support

### Where Percentages Work
✅ **CPU/Cores**: `jobs = "75%"` (75% of available cores)  
✅ **Memory/RAM**: `max_memory = "80%"` (80% of system RAM)  
✅ **Disk Space**: `cache_size = "5%"` (5% of available disk)  
✅ **Thresholds**: `cleanup_threshold = "90%"` (90% full)  
✅ **Performance**: `regression_threshold = "20%"` (20% slower)  

### Where Percentages Don't Apply
❌ **Time values**: Use durations like "30s", "5m"  
❌ **Counts**: Use absolute numbers  
❌ **Flags/Booleans**: Use true/false  
❌ **Strings**: Use literal values  

---

## Configuration Examples

### Minimal Configuration (Most users)
```toml
# cargo-optimize.toml
# Empty file - use all defaults!
```

### Development Machine
```toml
[parallel]
jobs = "75%"  # Leave some CPU for other tasks

[cache]
backend = "sccache"
max_size = "10%"  # 10% of disk

[optimize.profile.dev]
incremental = true
debug = 2
```

### CI Environment
```toml
[ci]
provider = "github"

[parallel]
jobs = "100%"  # Use all available cores

[cache]
backend = "sccache"
max_size = "50GB"

[optimize.profile.release]
lto = "fat"
codegen_units = 1
```

### Constrained Environment
```toml
[parallel]
jobs = 2
max_memory_per_job = "1GB"

[cache]
max_size = "5GB"

[budget.memory]
max_build_memory = "4GB"
```

### Performance-Critical Project
```toml
[linker]
preferred = "mold"

[parallel]
jobs = "100%"

[optimize.profile.release]
lto = "fat"
codegen_units = 1
strip = "all"
opt_level = 3

[platform]
target_cpu = "native"
```

This comprehensive customization system allows users to have fine-grained control while maintaining the zero-configuration philosophy for those who just want things to work out of the box.