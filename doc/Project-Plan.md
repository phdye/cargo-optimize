# Project Plan: cargo-optimize Full Implementation

## Executive Summary
Transform cargo-optimize from MVP (linker optimization only) into a comprehensive, zero-configuration Rust build optimization tool by **leveraging best-in-class crates** rather than building from scratch.

## Current State (MVP - v0.1.0)
- ✅ Linker detection and configuration
- ✅ Platform detection (Windows/Linux)
- ✅ Basic config file management
- ✅ ~30-70% build time improvement via linker alone

## Target State (v1.0.0)
Complete automated optimization system with:
- Hardware-aware configuration (via `sysinfo`/`num_cpus`)
- Build parallelization and caching
- Dependency analysis (via `cargo_metadata`/`guppy`)
- CI/CD integration (via `ci_info`)
- Build analytics and monitoring
- Performance budgets and regression detection

---

## Key Dependencies

### Core Crates We'll Use
```toml
[dependencies]
# System Information
sysinfo = "0.30"          # Comprehensive system/hardware info
num_cpus = "1.16"         # CPU detection (used by Rust itself)

# Configuration
figment = "0.10"          # Rocket's config library (layers, merge, validation)
toml_edit = "0.22"        # Preserve TOML formatting when editing

# Project Analysis  
cargo_metadata = "0.18"   # Official Cargo project parser
guppy = "0.17"           # Facebook's dependency graph analyzer

# CI Detection
ci_info = "0.14"         # Detects 20+ CI environments

# Web Dashboard
axum = "0.7"             # Modern web framework
minijinja = "1.0"        # Lightweight templates
tower-http = "0.5"       # Static file serving

# Utilities
which = "6.0"            # Find executables (already using)
anyhow = "1.0"           # Error handling
tracing = "0.1"          # Logging/diagnostics
```

---

## Development Conventions

[Keep existing conventions section - Branch Strategy, Version Strategy, etc.]

### Crate Integration Principles
- **Prefer established crates** over custom implementations
- **Wrap external crates** in our own modules for abstraction
- **Document why** each crate was chosen
- **Monitor crate health** (maintenance, security advisories)
- **Minimize dependencies** - only add what provides significant value

---

## Phase 1: Core Infrastructure (v0.2.0)
**Timeline: 1 week** (reduced from 2 weeks)  
**Goal: Robust foundation using proven libraries**
**Branch: `implementation/phase-1`**

### 1.1 Configuration Management
```rust
// src/config.rs - Using figment for layered configuration
use figment::{Figment, providers::{Format, Toml, Env}};
use toml_edit::Document;  // Preserve user's TOML formatting

// Features:
- Layer configs: defaults -> file -> env vars
- Preserve user's .cargo/config.toml formatting
- Automatic backup before changes
- Profile support (dev/test/release/bench)
```

**Deliverables:**
- [ ] Figment-based config system with merge logic
- [ ] TOML preservation using toml_edit
- [ ] Backup/restore mechanism
- [ ] Percentage value parsing layer
- [ ] Tests: `tests/config_management.rs`

### 1.2 Hardware Detection
```rust
// src/hardware.rs - Using sysinfo and num_cpus
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use num_cpus;  // For simple CPU count

// Provides:
- CPU cores (logical/physical)
- Memory info (total/available)
- Disk info (size/type/mount points)
- Platform-specific details
```

**Deliverables:**
- [ ] Thin wrapper around sysinfo
- [ ] Percentage calculation helpers
- [ ] Fallback values for failures
- [ ] Tests: `tests/hardware_detection.rs`

### 1.3 Project Analysis
```rust
// src/analysis.rs - Using cargo_metadata and guppy
use cargo_metadata::{MetadataCommand, Package};
use guppy::graph::PackageGraph;

// Provides:
- Workspace structure
- Dependency graph
- Feature analysis
- Build target detection
```

**Deliverables:**
- [ ] cargo_metadata integration
- [ ] Guppy-based dependency analysis
- [ ] Build metrics collection
- [ ] Tests: `tests/project_analysis.rs`

**Incremental Checkpoint 1.0:**
- Verify crate integrations work smoothly
- Test on various project structures
- Validate percentage calculations

---

## Phase 2: Build Optimization (v0.3.0)
**Timeline: 1 week** (reduced from 2 weeks)  
**Goal: Core build improvements**
**Branch: `implementation/phase-2`**

### 2.1 Parallel Build Configuration
```rust
// src/optimize/parallel.rs
use num_cpus;
use sysinfo::{System, SystemExt};

// Smart parallelization:
- Use num_cpus for detection
- Memory-aware limits from sysinfo
- Percentage support ("75%" of cores)
```

### 2.2 Cache Setup
```rust
// src/optimize/cache.rs
use which::which;  // Detect sccache/ccache
use sysinfo::DiskExt;  // For disk space

// Features:
- Detect cache tools via 'which'
- Smart cache sizing based on disk
- Generate CI-appropriate configs
```

### 2.3 Build Timing Analysis
```rust
// src/analyze/timing.rs
// Parse cargo's built-in --timings output
use cargo_metadata::Message;

// Instead of custom timing:
- Use `cargo build --timings`
- Parse the generated HTML/JSON
- Extract bottleneck information
```

**Deliverables:**
- [ ] Parallel optimization with num_cpus
- [ ] Cache tool detection and config
- [ ] Cargo timings parser
- [ ] Tests: Optimization suite

**Incremental Checkpoint 2.0:**
- Measure performance improvements
- Validate cache effectiveness
- Test timing analysis accuracy

---

## Phase 3: Dependency Optimization (v0.4.0)
**Timeline: 1 week** (reduced from 2 weeks)  
**Goal: Optimize dependencies using guppy**
**Branch: `implementation/phase-3`**

### 3.1 Feature Flag Analysis
```rust
// src/optimize/features.rs
use guppy::graph::{PackageGraph, FeatureSet};

// Guppy provides:
- Feature graph analysis
- Unused feature detection
- Feature unification insights
```

### 3.2 Bottleneck Detection
```rust
// src/analyze/bottlenecks.rs
// Combine cargo --timings with guppy

- Parse timing data
- Map to dependency graph
- Identify slow paths
```

### 3.3 Build Order Optimization
```rust
// src/optimize/build_order.rs
use guppy::graph::DependencyDirection;

// Guppy can:
- Analyze build order
- Find parallelization opportunities
- Suggest better ordering
```

**Deliverables:**
- [ ] Guppy-based feature analysis
- [ ] Timing + dependency correlation
- [ ] Build order recommendations
- [ ] Tests: Analysis accuracy

**Incremental Checkpoint 3.0:**
- Validate feature recommendations
- Test bottleneck detection
- Measure ordering improvements

---

## Phase 4: CI/CD Integration (v0.5.0)
**Timeline: 1 week** (reduced from 2 weeks)  
**Goal: CI/CD optimization**
**Branch: `implementation/phase-4`**

### 4.1 CI Environment Detection
```rust
// src/ci.rs
use ci_info::{is_ci, get};

// ci_info detects:
- GitHub Actions, GitLab CI, Jenkins
- CircleCI, Travis, Azure DevOps
- 20+ CI systems automatically
```

### 4.2 CI-Specific Optimization
```rust
// Build on ci_info detection
match ci_info::get().name.as_deref() {
    Some("GitHub Actions") => github_optimizations(),
    Some("GitLab CI") => gitlab_optimizations(),
    _ => generic_ci_optimizations(),
}
```

### 4.3 Metrics Export
```rust
// src/metrics.rs
use serde_json;  // For JSON export
use csv;         // For CSV export

// Simple serialization of:
- Build times (from cargo --timings)
- Cache stats (from sccache)
- System metrics (from sysinfo)
```

**Deliverables:**
- [ ] ci_info integration
- [ ] Platform-specific optimizations
- [ ] Metrics serialization
- [ ] Tests: CI simulation

**Incremental Checkpoint 4.0:**
- Test on real CI systems
- Validate detection accuracy
- Verify metrics format

---

## Phase 5: Monitoring & Analytics (v0.6.0)
**Timeline: 1.5 weeks**  
**Goal: Dashboard and monitoring**
**Branch: `implementation/phase-5`**

### 5.1 Web Dashboard
```rust
// src/dashboard/mod.rs
use axum::{Router, routing::get};
use minijinja::Environment;
use tower_http::services::ServeDir;

// Lightweight dashboard:
- Axum for web server
- Minijinja for templates
- Chart.js (CDN) for graphs
- Local-only by default (127.0.0.1)
```

### 5.2 Performance Budgets
```rust
// src/budget.rs
use figment::Provider;  // Reuse config system

// Simple threshold checking:
- Parse budget config
- Compare with metrics
- Generate warnings
```

### 5.3 Data Persistence
```rust
// src/storage.rs
use rusqlite;  // Embedded SQLite

// Store:
- Build history
- Metrics over time
- Regression baselines
```

**Deliverables:**
- [ ] Axum-based dashboard
- [ ] SQLite metrics storage
- [ ] Budget enforcement
- [ ] Tests: Web routes, storage

**Incremental Checkpoint 5.0:**
- Dashboard usability test
- Performance data accuracy
- Budget enforcement validation

---

## Phase 6: Platform & Polish (v1.0.0)
**Timeline: 1.5 weeks**  
**Goal: Production release**
**Branch: `implementation/phase-6`**

### 6.1 Platform Optimizations
```rust
// src/platform/mod.rs
use sysinfo::System;

// Platform-specific using sysinfo:
#[cfg(windows)]
mod windows;  // Defender, MSVC
#[cfg(unix)]
mod unix;     // tmpfs, linker selection
#[cfg(macos)]
mod macos;    // Universal binaries
```

### 6.2 Documentation
- User guide with examples
- Configuration reference
- Troubleshooting guide
- Crate documentation

### 6.3 Release Engineering
```rust
// Using existing tools:
- cargo-dist for binary releases
- cargo-release for version management
- GitHub Actions for CI/CD
```

**Final Deliverables:**
- [ ] Platform modules
- [ ] Complete documentation
- [ ] Release automation
- [ ] Package manager configs

---

## Dependency Health Monitoring

### Critical Dependencies to Track
| Crate | Purpose | Health Check |
|-------|---------|--------------|
| sysinfo | Hardware detection | Monthly updates, 5M+ downloads |
| figment | Configuration | Rocket maintains it |
| cargo_metadata | Project analysis | Official, stable |
| guppy | Dependency analysis | Facebook maintains it |
| ci_info | CI detection | Active development |
| axum | Web framework | Tokio team maintains it |

### Maintenance Strategy
- Review dependency updates monthly
- Security advisories via `cargo audit`
- Consider alternatives if crate abandoned
- Minimize transitive dependencies

---

## Revised Timeline

| Phase | Version | Duration | Crates Used |
|-------|---------|----------|-------------|
| 1. Core Infrastructure | v0.2.0 | 1 week | figment, sysinfo, cargo_metadata, guppy |
| 2. Build Optimization | v0.3.0 | 1 week | num_cpus, which |
| 3. Dependency Opt | v0.4.0 | 1 week | guppy, cargo --timings |
| 4. CI/CD Integration | v0.5.0 | 1 week | ci_info, serde_json, csv |
| 5. Monitoring | v0.6.0 | 1.5 weeks | axum, minijinja, rusqlite |
| 6. Platform & Polish | v1.0.0 | 1.5 weeks | platform-specific |

**Total: 7 weeks** (reduced from 12 weeks!)

---

## What We Actually Build

### Our Core Value (Not Available in Crates)
1. **Optimization Logic** - The rules for what to optimize when
2. **Integration Layer** - Tying all the crates together
3. **Percentage Calculations** - Our "75%" feature
4. **Profile Templates** - Pre-configured optimization profiles
5. **Rollback System** - Safety mechanism for changes
6. **Auto-configuration** - Zero-config optimization logic

### We DON'T Build
- ❌ System info detection (use sysinfo)
- ❌ Config file parsing (use figment/toml_edit)
- ❌ Dependency graphs (use guppy)
- ❌ CI detection (use ci_info)
- ❌ Web server (use axum)
- ❌ Template engine (use minijinja)

---

## Benefits of This Approach

### Time Savings
- **Original estimate**: 12 weeks
- **Revised estimate**: 7 weeks
- **Time saved**: 5 weeks (42% reduction!)

### Quality Improvements
- **Battle-tested code**: Millions of downloads
- **Cross-platform**: Already solved OS differences
- **Maintained**: Active communities
- **Documented**: Existing docs and examples
- **Secure**: Audited by many users

### Focus Benefits
- More time on our unique value proposition
- Less time debugging low-level issues
- Faster to market
- Easier maintenance

---

## Risk Mitigation

### Dependency Risks
- **Abandoned crates**: Have backup alternatives identified
- **Breaking changes**: Pin major versions
- **Security issues**: Regular `cargo audit`
- **License conflicts**: All chosen crates are MIT/Apache-2.0

### Integration Risks
- **API changes**: Wrap in abstraction layers
- **Performance overhead**: Benchmark each integration
- **Feature gaps**: Contribute upstream or extend locally

---

## Next Steps

1. **Add dependencies to Cargo.toml**:
   ```bash
   cargo add sysinfo num_cpus figment toml_edit cargo_metadata guppy ci_info
   ```
2. **Create abstraction modules** for each external crate
3. **Start Phase 1** with proven libraries
4. **Set up dependency monitoring** (cargo-audit, cargo-outdated)

This approach gets us to market **5 weeks faster** with **higher quality** by standing on the shoulders of giants in the Rust ecosystem!