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

## Development Conventions

### Branch Strategy
Every phase and section follows this branching pattern:

```
main
└── implementation/phase-N        # Phase branch
    ├── 1-feature-name           # Feature branches
    ├── 2-feature-name
    └── 3-feature-name
```

**Naming Convention:**
- Phase branches: `implementation/phase-1`, `implementation/phase-2`, etc.
- Feature branches: `M-descriptive-name` (from within phase branch)
  - M = section number within phase
  - descriptive-name = kebab-case feature description
  - Example: `1-config-management`, `2-hardware-detection`

### Version Strategy
- **Patch (v0.2.x)**: Bug fixes, documentation updates, minor improvements
- **Minor (v0.x.0)**: New features (typically each phase completion)
- **Major (vx.0.0)**: Breaking changes to config format, CLI, or behavior

### Commit Message Convention
Use conventional commits format:
```bash
# Recommended format
git commit -m "feat(config): implement configuration management"
git commit -m "fix(hardware): handle sysinfo failure gracefully"
git commit -m "test(config): add merge precedence tests"
git commit -m "docs(readme): update installation instructions"
```

### Standard Development Workflow

**For Each Phase:**
1. `git checkout -b implementation/phase-N` from main
2. Complete all sections within the phase
3. Merge back to main when phase is complete
4. Tag with version number (e.g., `v0.2.0`)

**For Each Section within a Phase:**
1. `git checkout -b M-feature-name` from phase branch
2. Implement the feature
3. Write tests according to `doc/Comprehensive-Test-Strategy.md`
4. Complete **Quality Checklist** (see below)
5. `git commit -m "feat(module): description"`
6. `git checkout implementation/phase-N`
7. `git merge M-feature-name`

### Quality Checklist
Before merging any section:

- [ ] All tests written according to `doc/Comprehensive-Test-Strategy.md`
- [ ] Test strategy checkpoints followed (save work every ~100 lines)
- [ ] Minimum test coverage met (80% for new code)
- [ ] `cargo test` passes with no failures
- [ ] `cargo check --tests` shows no errors or warnings
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All code is thoroughly commented
- [ ] Documentation updated (`README.md`, `doc/`, inline docs)
- [ ] Project root is clean (no development artifacts)
- [ ] Benchmark results recorded in `issue/benchmark/`
- [ ] `git add -u` and `git add [new-files]` completed
- [ ] Meaningful commit message following convention

### Rollback Procedure
If a feature needs to be reverted:

```bash
# Git rollback
git checkout implementation/phase-N
git revert -m 1 <merge-commit-hash>
# Document in issue/phase-N/rollback-NNN.md

# User system rollback (to be implemented in Phase 1)
cargo optimize rollback  # Restores original .cargo/config.toml from backup
```

### Phase Completion Checklist
Before merging phase to main:

- [ ] All section branches merged to phase branch
- [ ] Full test suite passes: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Documentation complete: `cargo doc --open` works
- [ ] Benchmarks recorded: `issue/benchmark/phase-N/`
- [ ] Performance regression check completed (compare to previous version)
- [ ] **Phase objectives validated:**
  - [ ] Review checkpoint criteria met (see Incremental Checkpoint in phase)
  - [ ] Performance metrics documented in `issue/phase-N/metrics.md`
  - [ ] Any deviations from plan documented
  - [ ] Risks or blockers identified and addressed
  - [ ] **If objectives not met**: Document remediation plan in `issue/phase-N/remediation.md`
- [ ] Changelog updated with all changes
- [ ] Version bumped in `Cargo.toml`
- [ ] Dependencies audited: `cargo audit`
- [ ] `git checkout main && git merge implementation/phase-N`
- [ ] `git tag vX.Y.Z`
- [ ] Push tags: `git push origin main --tags`

---

## Core Principles

### Error Handling Philosophy
- **Always preserve** user's original configuration (backup before modify)
- **Fail safely** with clear, actionable error messages
- **Never leave** system in broken state
- **Provide rollback** mechanism for all changes

### Backward Compatibility
- **MSRV**: Rust 1.70+ (document in Cargo.toml)
- **Cargo versions**: Handle gracefully from 1.70 onward
- **Config versioning**: Include version field in cargo-optimize.toml
- **Migration support**: Provide upgrade path for config changes

### Testing Strategy Application
Following `doc/Comprehensive-Test-Strategy.md`:
- **Unit tests**: For each module
- **Integration tests**: For feature interactions
- **Boundary tests**: Edge cases and limits
- **Platform tests**: Windows, Linux, macOS specific
- **Regression tests**: For each fixed bug
- **Performance tests**: Benchmark critical paths

### Percentage Value Specification
When implementing percentage support:
```toml
[build]
jobs = "75%"         # 75% of logical CPU cores, rounded down
cache-size = "10%"   # 10% of available disk space, max 10GB
memory-limit = "50%" # 50% of available RAM, rounded to nearest GB
```

### Crate Integration Principles
- **Prefer established crates** over custom implementations
- **Wrap external crates** in our own modules for abstraction
- **Document why** each crate was chosen
- **Monitor crate health** (maintenance, security advisories)
- **Minimize dependencies** - only add what provides significant value

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

# Web Dashboard (Phase 5)
axum = "0.7"             # Modern web framework
minijinja = "1.0"        # Lightweight templates
tower-http = "0.5"       # Static file serving

# Storage (Phase 5)
rusqlite = "0.31"        # Embedded database

# Utilities
which = "6.0"            # Find executables (already using)
anyhow = "1.0"           # Error handling
tracing = "0.1"          # Logging/diagnostics
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"       # JSON serialization
```

---

## Phase 1: Core Infrastructure (v0.2.0)
**Timeline: 1 week**  
**Goal: Robust foundation using proven libraries**
**Branch: `implementation/phase-1`**

### 1.1 Configuration Management Enhancement

**Branch: `1-config-management`**

```rust
// src/config.rs - Using figment for layered configuration
use figment::{Figment, providers::{Format, Toml, Env}};
use toml_edit::Document;  // Preserve user's TOML formatting
```

**Implementation Tasks:**
- [ ] Create config module with figment integration
- [ ] Implement TOML preservation with toml_edit
- [ ] Add backup/restore mechanism (store in `.cargo/config.toml.backup`)
- [ ] Implement percentage value parser
- [ ] Create profile templates (dev/test/release/bench)

**Tests Required** (`tests/config_management.rs`):
- [ ] Unit: Config loading, merging, serialization
- [ ] Integration: Multi-file config precedence
- [ ] Boundary: Invalid percentages, missing files
- [ ] Regression: Preserve user formatting

**Deliverables:**
- [ ] Figment-based config system with merge logic
- [ ] TOML preservation using toml_edit
- [ ] Backup/restore mechanism
- [ ] Percentage value parsing layer
- [ ] Complete test suite

### 1.2 Hardware Detection Module

**Branch: `2-hardware-detection`**

```rust
// src/hardware.rs - Using sysinfo and num_cpus
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use num_cpus;  // For simple CPU count
```

**Implementation Tasks:**
- [ ] Create hardware module wrapping sysinfo
- [ ] Implement percentage calculation helpers
- [ ] Add fallback values for detection failures
- [ ] Create platform-specific optimizations
- [ ] Add caching for expensive operations

**Tests Required** (`tests/hardware_detection.rs`):
- [ ] Unit: CPU/RAM/Disk detection
- [ ] Integration: Percentage calculations
- [ ] Boundary: Zero resources, overflow
- [ ] Platform: Windows/Linux/macOS specific

**Deliverables:**
- [ ] Thin wrapper around sysinfo
- [ ] Percentage calculation helpers
- [ ] Fallback values for failures
- [ ] Complete test suite

### 1.3 Project Analysis Engine

**Branch: `3-project-analysis`**

```rust
// src/analysis.rs - Using cargo_metadata and guppy
use cargo_metadata::{MetadataCommand, Package};
use guppy::graph::PackageGraph;
```

**Implementation Tasks:**
- [ ] Create analysis module with cargo_metadata
- [ ] Integrate guppy for dependency graphs
- [ ] Implement workspace detection
- [ ] Add build target identification
- [ ] Create metrics collection

**Tests Required** (`tests/project_analysis.rs`):
- [ ] Unit: Metadata parsing, graph creation
- [ ] Integration: Multi-workspace projects
- [ ] Boundary: Malformed Cargo.toml
- [ ] Performance: Large dependency graphs

**Deliverables:**
- [ ] cargo_metadata integration
- [ ] Guppy-based dependency analysis
- [ ] Build metrics collection
- [ ] Complete test suite

**Incremental Checkpoint 1.0:**
- [ ] Verify all crate integrations work smoothly
- [ ] Test on 5+ different project structures
- [ ] Validate percentage calculations with edge cases
- [ ] Performance baseline: <100ms for analysis
- [ ] Document any API limitations discovered

---

## Phase 2: Build Optimization (v0.3.0)
**Timeline: 1 week**  
**Goal: Core build improvements**
**Branch: `implementation/phase-2`**

### 2.1 Parallel Build Configuration

**Branch: `1-parallel-builds`**

```rust
// src/optimize/parallel.rs
use num_cpus;
use sysinfo::{System, SystemExt};
```

**Implementation Tasks:**
- [ ] Calculate optimal job count based on CPU/RAM
- [ ] Implement memory-aware limits
- [ ] Add percentage support parsing
- [ ] Create cargo config generation
- [ ] Add validation logic

**Tests Required** (`tests/parallel_optimization.rs`):
- [ ] Unit: Job calculation logic
- [ ] Integration: Config file generation
- [ ] Boundary: 1 core, 1000 cores
- [ ] Performance: Measure actual speedup

### 2.2 Cache Setup

**Branch: `2-cache-setup`**

```rust
// src/optimize/cache.rs
use which::which;  // Detect sccache/ccache
use sysinfo::DiskExt;  // For disk space
```

**Implementation Tasks:**
- [ ] Detect available cache tools
- [ ] Implement smart cache sizing
- [ ] Add CI-specific configurations
- [ ] Create cache statistics collector
- [ ] Add cache cleanup logic

**Tests Required** (`tests/cache_optimization.rs`):
- [ ] Unit: Cache size calculations
- [ ] Integration: Tool detection
- [ ] Boundary: No disk space, no tools
- [ ] CI: GitHub Actions simulation

### 2.3 Build Timing Analysis

**Branch: `3-timing-analysis`**

```rust
// src/analyze/timing.rs
use cargo_metadata::Message;
```

**Implementation Tasks:**
- [ ] Parse cargo --timings output
- [ ] Extract bottleneck information
- [ ] Create timing report generator
- [ ] Add comparison logic
- [ ] Implement trend detection

**Tests Required** (`tests/timing_analysis.rs`):
- [ ] Unit: Parsing logic
- [ ] Integration: Real cargo output
- [ ] Boundary: Empty builds, huge builds
- [ ] Regression: Format changes

**Incremental Checkpoint 2.0:**
- [ ] Measure performance improvements (target: 20-30%)
- [ ] Validate cache hit rates (target: >80%)
- [ ] Test on 10+ real projects
- [ ] Document optimization limits

---

## Phase 3: Dependency Optimization (v0.4.0)
**Timeline: 1 week**  
**Goal: Optimize dependencies using guppy**
**Branch: `implementation/phase-3`**

### 3.1 Feature Flag Analysis

**Branch: `1-feature-analysis`**

```rust
// src/optimize/features.rs
use guppy::graph::{PackageGraph, FeatureSet};
```

**Implementation Tasks:**
- [ ] Implement feature usage detection
- [ ] Create unused feature finder
- [ ] Add feature recommendation engine
- [ ] Generate optimization reports
- [ ] Add safety validation

**Tests Required** (`tests/feature_optimization.rs`):
- [ ] Unit: Feature detection logic
- [ ] Integration: Real project analysis
- [ ] Boundary: No features, all features
- [ ] Safety: Don't break builds

### 3.2 Bottleneck Detection

**Branch: `2-bottleneck-detection`**

**Implementation Tasks:**
- [ ] Combine timing data with dependency graph
- [ ] Identify critical path
- [ ] Create bottleneck scorer
- [ ] Generate actionable reports
- [ ] Add alternative suggestions

**Tests Required** (`tests/bottleneck_detection.rs`):
- [ ] Unit: Scoring algorithm
- [ ] Integration: Real build data
- [ ] Performance: Large graphs
- [ ] Accuracy: Known bottlenecks

### 3.3 Build Order Optimization

**Branch: `3-build-order`**

**Implementation Tasks:**
- [ ] Analyze current build order
- [ ] Find parallelization opportunities
- [ ] Create order optimizer
- [ ] Validate correctness
- [ ] Measure improvements

**Tests Required** (`tests/build_order.rs`):
- [ ] Unit: Order algorithm
- [ ] Integration: Workspace builds
- [ ] Correctness: No circular deps
- [ ] Performance: Actual speedup

**Incremental Checkpoint 3.0:**
- [ ] Validate recommendations don't break builds
- [ ] Measure feature reduction impact (target: 10-20% faster)
- [ ] Test bottleneck accuracy (>90% correct)
- [ ] Document found limitations

---

## Phase 4: CI/CD Integration (v0.5.0)
**Timeline: 1 week**  
**Goal: CI/CD optimization**
**Branch: `implementation/phase-4`**

**Prerequisites:**
- [ ] GitHub Actions workflow created
- [ ] Test repositories prepared
- [ ] CI access tokens configured

### 4.1 CI Environment Detection

**Branch: `1-ci-detection`**

```rust
// src/ci.rs
use ci_info::{is_ci, get};
```

**Implementation Tasks:**
- [ ] Integrate ci_info crate
- [ ] Create CI abstraction layer
- [ ] Add platform-specific logic
- [ ] Implement cache key generation
- [ ] Add environment validation

**Tests Required** (`tests/ci_detection.rs`):
- [ ] Unit: Detection logic
- [ ] Integration: Simulated environments
- [ ] Coverage: All major CI systems
- [ ] Edge cases: Unknown CI

### 4.2 CI-Specific Optimization

**Branch: `2-ci-optimization`**

**Implementation Tasks:**
- [ ] GitHub Actions optimization
- [ ] GitLab CI optimization
- [ ] Jenkins optimization
- [ ] Generic CI fallback
- [ ] Matrix build support

**Tests Required** (`tests/ci_optimization.rs`):
- [ ] Unit: Optimization rules
- [ ] Integration: Real CI configs
- [ ] Matrix: Multiple targets
- [ ] Performance: Cache effectiveness

### 4.3 Metrics Export

**Branch: `3-metrics-export`**

**Implementation Tasks:**
- [ ] JSON export format
- [ ] CSV export format
- [ ] CI-specific formats
- [ ] Metrics aggregation
- [ ] Trend analysis

**Tests Required** (`tests/metrics_export.rs`):
- [ ] Unit: Serialization
- [ ] Integration: Real metrics
- [ ] Format: Valid JSON/CSV
- [ ] Performance: Large datasets

**Incremental Checkpoint 4.0:**
- [ ] Test on real CI (GitHub Actions minimum)
- [ ] Validate cache keys (no collisions)
- [ ] Verify metrics accuracy
- [ ] Measure CI speedup (target: 30-50%)

---

## Phase 5: Monitoring & Analytics (v0.6.0)
**Timeline: 1.5 weeks**  
**Goal: Dashboard and monitoring**
**Branch: `implementation/phase-5`**

### 5.1 Web Dashboard

**Branch: `1-dashboard`**

```rust
// src/dashboard/mod.rs
use axum::{Router, routing::get};
use minijinja::Environment;
use tower_http::services::ServeDir;
```

**Implementation Tasks:**
- [ ] Create axum web server
- [ ] Design dashboard UI
- [ ] Implement API endpoints
- [ ] Add real-time updates
- [ ] Ensure local-only by default

**Tests Required** (`tests/dashboard.rs`):
- [ ] Unit: Route handlers
- [ ] Integration: Full server
- [ ] Security: Local-only binding
- [ ] UI: Template rendering

### 5.2 Performance Budgets

**Branch: `2-budgets`**

**Implementation Tasks:**
- [ ] Budget configuration parser
- [ ] Threshold checking logic
- [ ] Alert generation
- [ ] Trend analysis
- [ ] Report generation

**Tests Required** (`tests/budgets.rs`):
- [ ] Unit: Threshold logic
- [ ] Integration: Real builds
- [ ] Boundary: Edge values
- [ ] Alerts: Proper triggering

### 5.3 Data Persistence

**Branch: `3-persistence`**

```rust
// src/storage.rs
use rusqlite;
```

**Implementation Tasks:**
- [ ] Database schema design
- [ ] Migration system
- [ ] Data insertion logic
- [ ] Query optimization
- [ ] Cleanup/rotation

**Tests Required** (`tests/storage.rs`):
- [ ] Unit: CRUD operations
- [ ] Integration: Full workflow
- [ ] Performance: Large datasets
- [ ] Migration: Schema updates

**Incremental Checkpoint 5.0:**
- [ ] Dashboard usability testing
- [ ] Performance data accuracy validation
- [ ] Budget enforcement verification
- [ ] Load testing (1000+ builds)

---

## Phase 6: Platform & Polish (v1.0.0)
**Timeline: 1.5 weeks**  
**Goal: Production release**
**Branch: `implementation/phase-6`**

### 6.1 Platform-Specific Optimizations

**Branch: `1-platform-specific`**

```rust
// src/platform/mod.rs
#[cfg(windows)]
mod windows;
#[cfg(unix)]
mod unix;
#[cfg(macos)]
mod macos;
```

**Implementation Tasks:**
- [ ] Windows: Defender exclusions
- [ ] Linux: tmpfs optimization
- [ ] macOS: Universal binary support
- [ ] Platform detection logic
- [ ] Fallback strategies

**Tests Required** (`tests/platform.rs`):
- [ ] Platform-specific tests
- [ ] Cross-platform compatibility
- [ ] Fallback testing
- [ ] Performance validation

### 6.2 Documentation

**Branch: `2-documentation`**

**Deliverables:**
- [ ] User guide (doc/user-guide.md)
- [ ] Configuration reference (doc/config-reference.md)
- [ ] Troubleshooting guide (doc/troubleshooting.md)
- [ ] Migration guide (doc/migration.md)
- [ ] API documentation (rustdoc)

### 6.3 Release Engineering

**Branch: `3-release`**

**Implementation Tasks:**
- [ ] cargo-dist configuration
- [ ] GitHub Actions release workflow
- [ ] Binary packaging scripts
- [ ] Installation scripts
- [ ] Package manager configs

**Final Checklist:**
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Benchmarks documented
- [ ] CHANGELOG updated
- [ ] Version tagged
- [ ] Binaries built
- [ ] crates.io published

---

## Testing Strategy Application

### Test Coverage Requirements
- **New code**: Minimum 100% coverage
- **Overall**: Minimum 100% coverage
- **Critical paths**: 100% coverage

### Test Execution Per Phase
```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Run clippy
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Platform Testing Matrix
- **Windows**: Windows 10/11, MSVC toolchain
- **Linux**: Ubuntu 22.04, Debian 12, Arch
- **macOS**: Latest two versions
- **CI**: All platforms via GitHub Actions

---

## Performance Baselines & Targets

### Baseline Project
```bash
cargo new bench_project
cd bench_project
# Add dependencies
cargo add tokio serde clap
```

### Success Metrics
- **Build Time**: 50-70% reduction vs baseline
- **Cache Hit Rate**: >80% in development
- **Analysis Time**: <100ms for most projects
- **Dashboard Load**: <500ms

---

## Risk Management

### Dependency Monitoring
```bash
# Weekly checks
cargo outdated
cargo audit
cargo tree --duplicates
```

### Performance Regression Prevention
- Benchmark every merge
- Compare to previous version
- Block merge if >10% regression

---

## Timeline Summary

| Phase | Version | Duration | Key Deliverables |
|-------|---------|----------|------------------|
| 1. Core | v0.2.0 | 1 week | Config, Hardware, Analysis |
| 2. Build | v0.3.0 | 1 week | Parallel, Cache, Timing |
| 3. Dependencies | v0.4.0 | 1 week | Features, Bottlenecks, Order |
| 4. CI/CD | v0.5.0 | 1 week | Detection, Optimization, Metrics |
| 5. Monitoring | v0.6.0 | 1.5 weeks | Dashboard, Budgets, Storage |
| 6. Polish | v1.0.0 | 1.5 weeks | Platform, Docs, Release |

**Total: 7 weeks**

---

## Next Immediate Steps

1. **Verify prerequisites**:
   ```bash
   # Check test strategy exists
   test -f doc/Comprehensive-Test-Strategy.md || echo "Create test strategy"
   
   # Install initial dependencies
   cargo add sysinfo num_cpus figment toml_edit cargo_metadata guppy ci_info
   ```

2. **Create Phase 1 branch**:
   ```bash
   git checkout -b implementation/phase-1
   ```

3. **Start Section 1.1**:
   ```bash
   git checkout -b 1-config-management
   ```

4. **Set up test infrastructure**:
   ```bash
   mkdir -p tests issue/benchmark/001
   ```

5. **Begin implementation** following the workflow above