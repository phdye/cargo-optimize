# Scripts Directory

This directory contains **contributor development tools** for the cargo-optimize project. These scripts are for developers working on cargo-optimize itself, not for end users of the crate.

## 🎯 **For End Users** (Don't use these scripts!)

If you want to optimize your Rust project with cargo-optimize, **don't use the scripts in this directory.** Instead, use the CLI:

```bash
# Install cargo-optimize
cargo install cargo-optimize

# Set up optimizations for your project
cargo optimize setup

# Or just apply optimizations directly
cargo optimize
```

## 🛠️ **For Contributors** (These scripts are for you!)

### **Development Environment Setup**
- **`dev-setup.rs`** - Complete development environment setup
  ```bash
  # Install rust-script first
  cargo install rust-script
  
  # Run the setup
  rust-script scripts/dev-setup.rs
  ```

### **Project Maintenance**
- **`fix-permissions.sh`** - Fix file permissions (removes execute bits from non-executable files)
- **`apply-cleanup.sh`** - Apply all fixes and commit to git
- **`remove-execute-permissions.sh`** - Legacy permission cleanup script

## 🚀 **Quick Start for Contributors**

### **First-time setup:**
```bash
# 1. Clone the repository
git clone https://github.com/your-username/cargo-optimize
cd cargo-optimize

# 2. Set up development environment
rust-script scripts/dev-setup.rs

# 3. Fix permissions if needed
./scripts/fix-permissions.sh

# 4. Run tests to verify everything works
cargo nextest run
```

### **Daily development:**
```bash
# Check your changes
cargo check
cargo clippy
cargo fmt

# Run tests
cargo nextest run

# Test CLI changes
cargo run -- setup --dry-run

# Apply project fixes
./scripts/apply-cleanup.sh
```

## 📁 **Why These Scripts Are Here**

Scripts are organized by target audience:

| Audience | Location | Examples |
|----------|----------|-----------|
| **End Users** | Built into CLI | `cargo optimize setup` |
| **Contributors** | `./scripts/` | `dev-setup.rs`, `fix-permissions.sh` |

### **✅ What SHOULD be in ./scripts/:**
- Development environment setup
- Project maintenance tools
- Build/release automation
- Testing utilities
- Code quality tools

### **❌ What should NOT be in ./scripts/:**
- End-user setup scripts
- Installation helpers for crate users
- Project usage examples

## 🔧 **CLI Commands (Replaced Old Scripts)**

The old `setup.sh` and `setup.bat` scripts have been **replaced** by CLI subcommands:

| Old Script | New CLI Command | Purpose |
|------------|-----------------|---------|
| `setup.sh` | `cargo optimize setup` | Set up optimizations for a project |
| `setup.bat` | `cargo optimize setup` | (Same command, cross-platform) |
| N/A | `cargo optimize init` | Initialize project with config files |
| N/A | `cargo optimize install` | Install recommended tools |

## 🎯 **End User Documentation**

For end users, the proper workflow is:

```bash
# 1. Install the tool
cargo install cargo-optimize

# 2. Set up your project
cd my-rust-project
cargo optimize setup

# 3. Enjoy faster builds!
cargo build  # Now optimized!
```

**No scripts needed!** Everything is built into the CLI.

## 📊 **File Permission Rules**

This directory follows proper Unix file permissions:

| File Type | Permission | Example |
|-----------|------------|---------|
| `*.rs` | `644` (rw-r--r--) | `dev-setup.rs` |
| `*.sh` | `755` (rwxr-xr-x) | `fix-permissions.sh` |
| `*.md` | `644` (rw-r--r--) | `README.md` |
| `*.bat` | `644` (rw-r--r--) | No execute on Unix |

The `fix-permissions.sh` script automatically handles all these cases.

## 🤝 **Contributing Workflow**

1. **Setup:** `rust-script scripts/dev-setup.rs`
2. **Develop:** Make your changes
3. **Test:** `cargo nextest run`
4. **Quality:** `cargo clippy && cargo fmt`
5. **Fix:** `./scripts/fix-permissions.sh`
6. **Commit:** Your changes are ready!

For detailed contributing guidelines, see [CONTRIBUTING.md](../CONTRIBUTING.md).
