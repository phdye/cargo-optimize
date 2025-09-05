#!/usr/bin/env rust-script
//! Development environment setup for cargo-optimize contributors
//! 
//! This script sets up the development environment with all necessary tools
//! for contributing to the cargo-optimize project.
//!
//! Usage:
//!   rust-script scripts/dev-setup.rs
//!   # OR compile and run:
//!   rustc scripts/dev-setup.rs -o dev-setup && ./dev-setup
//!
//! ```cargo
//! [dependencies]
//! clap = "4.0"
//! ```

use std::process::{Command, exit};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("🛠️  cargo-optimize Development Environment Setup");
    println!("Setting up tools and configurations for contributors...");
    println!();
    
    // Check we're in the right directory
    if !Path::new("Cargo.toml").exists() || !Path::new("src").exists() {
        error("Please run this script from the cargo-optimize project root directory");
    }
    
    // Check Rust installation
    info("🦀 Checking Rust installation...");
    if !check_command("cargo", &["--version"]) {
        error("Cargo not found. Install Rust from https://rustup.rs/");
    }
    if !check_command("rustc", &["--version"]) {
        error("rustc not found. Install Rust from https://rustup.rs/");
    }
    success("Rust toolchain found");
    
    // Install rustup components
    info("🔧 Installing Rust components...");
    install_rustup_component("clippy");
    install_rustup_component("rustfmt");
    install_rustup_component("rust-src");
    install_rustup_component("rust-docs");
    
    // Install development tools
    info("📦 Installing development tools...");
    install_cargo_tool("cargo-nextest", "Fast test runner");
    install_cargo_tool("cargo-llvm-cov", "Code coverage");
    install_cargo_tool("cargo-audit", "Security auditing");
    install_cargo_tool("cargo-deny", "Dependency checking");
    install_cargo_tool("cargo-watch", "File watching");
    install_cargo_tool("cargo-expand", "Macro expansion");
    install_cargo_tool("cargo-edit", "Dependency management");
    install_cargo_tool("rust-script", "Run Rust scripts");
    
    // Install optional but useful tools
    info("⚡ Installing performance tools...");
    install_cargo_tool("sccache", "Build caching");
    install_cargo_tool("cargo-workspaces", "Workspace management");
    
    // Set up git hooks (if git repository)
    if Path::new(".git").exists() {
        info("📝 Setting up git hooks...");
        setup_git_hooks();
    }
    
    // Create .vscode settings if directory exists
    if Path::new(".vscode").exists() {
        info("💻 Setting up VS Code configuration...");
        setup_vscode_settings();
    } else {
        info("💡 VS Code directory not found. Create .vscode/ and re-run for IDE setup");
    }
    
    // Run initial checks
    info("🧪 Running initial project checks...");
    run_command_ignore_fail("cargo", &["check"]);
    run_command_ignore_fail("cargo", &["clippy", "--", "-D", "warnings"]);
    run_command_ignore_fail("cargo", &["fmt", "--check"]);
    
    // Create development aliases
    info("🚀 Setting up development shortcuts...");
    create_dev_aliases();
    
    println!();
    success("✅ Development environment setup complete!");
    println!();
    println!("🎯 Quick start for contributors:");
    println!("  cargo check          # Basic compile check");
    println!("  cargo clippy         # Linting");
    println!("  cargo fmt            # Format code");
    println!("  cargo nextest run    # Run tests (fast)");
    println!("  cargo llvm-cov       # Code coverage");
    println!("  cargo audit          # Security audit");
    println!("  cargo watch -x test  # Watch and test");
    println!();
    println!("📚 Development workflows:");
    println!("  ./scripts/fix-permissions.sh     # Fix file permissions");
    println!("  ./scripts/apply-cleanup.sh       # Apply project cleanup");
    println!("  cargo run -- setup --dry-run     # Test CLI changes");
    println!("  cargo run --example basic        # Test library changes");
    println!();
    println!("🔗 Useful links:");
    println!("  Docs: https://docs.rs/cargo-optimize");
    println!("  Contributing: ./CONTRIBUTING.md");
    println!("  Issues: https://github.com/your-repo/cargo-optimize/issues");
    println!();
    println!("Happy contributing! 🎉");
}

fn check_command(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd).args(args).output().is_ok()
}

fn run_command(cmd: &str, args: &[&str]) -> bool {
    let status = Command::new(cmd).args(args).status();
    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn run_command_ignore_fail(cmd: &str, args: &[&str]) {
    let _ = Command::new(cmd).args(args).status();
}

fn install_rustup_component(component: &str) {
    print!("  Installing {}... ", component);
    if run_command("rustup", &["component", "add", component]) {
        println!("✅");
    } else {
        println!("⚠️  Failed (may already be installed)");
    }
}

fn install_cargo_tool(tool: &str, description: &str) {
    if check_command(tool, &["--version"]) {
        println!("  {} - ✅ Already installed", description);
        return;
    }
    
    print!("  Installing {} ({})... ", tool, description);
    if run_command("cargo", &["install", tool, "--locked"]) {
        println!("✅");
    } else {
        println!("⚠️  Failed");
    }
}

fn setup_git_hooks() {
    let pre_commit_hook = r#"#!/bin/sh
# Pre-commit hook for cargo-optimize
set -e

echo "🧪 Running pre-commit checks..."

# Format check
echo "📝 Checking formatting..."
cargo fmt --check

# Clippy check
echo "🔍 Running clippy..."
cargo clippy -- -D warnings

# Test check
echo "🧪 Running tests..."
cargo nextest run --no-capture

echo "✅ All checks passed!"
"#;
    
    let hook_path = ".git/hooks/pre-commit";
    if let Err(e) = fs::write(hook_path, pre_commit_hook) {
        warn(&format!("Failed to create pre-commit hook: {}", e));
    } else {
        // Make executable (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(mut perms) = fs::metadata(hook_path).map(|m| m.permissions()) {
                perms.set_mode(0o755);
                let _ = fs::set_permissions(hook_path, perms);
            }
        }
        println!("  Created git pre-commit hook");
    }
}

fn setup_vscode_settings() {
    let settings = r#"{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "editor.formatOnSave": true,
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true,
    "rust-analyzer.lens.enable": true,
    "rust-analyzer.lens.run.enable": true,
    "rust-analyzer.lens.debug.enable": true,
    "rust-analyzer.cargo.features": "all"
}
"#;
    
    if let Err(e) = fs::write(".vscode/settings.json", settings) {
        warn(&format!("Failed to create VS Code settings: {}", e));
    } else {
        println!("  Created .vscode/settings.json");
    }
    
    let extensions = r#"{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb"
    ]
}
"#;
    
    if let Err(e) = fs::write(".vscode/extensions.json", extensions) {
        warn(&format!("Failed to create VS Code extensions: {}", e));
    } else {
        println!("  Created .vscode/extensions.json");
    }
}

fn create_dev_aliases() {
    let aliases = r#"# cargo-optimize development aliases
# Add these to your shell profile (~/.bashrc, ~/.zshrc, etc.)

# Quick commands
alias co-check='cargo check'
alias co-test='cargo nextest run'
alias co-fmt='cargo fmt'
alias co-lint='cargo clippy -- -D warnings'
alias co-cov='cargo llvm-cov --html'
alias co-audit='cargo audit'

# Development workflows  
alias co-watch='cargo watch -x test'
alias co-fix='./scripts/fix-permissions.sh'
alias co-clean='./scripts/apply-cleanup.sh'

# Testing the CLI
alias co-dry='cargo run -- setup --dry-run'
alias co-example='cargo run --example'
"#;
    
    if let Err(e) = fs::write("dev-aliases.sh", aliases) {
        warn(&format!("Failed to create dev aliases: {}", e));
    } else {
        println!("  Created dev-aliases.sh (source this in your shell)");
    }
}

fn info(msg: &str) {
    println!("[INFO] {}", msg);
}

fn warn(msg: &str) {
    println!("[WARN] {}", msg);
}

fn success(msg: &str) {
    println!("[SUCCESS] {}", msg);
}

fn error(msg: &str) -> ! {
    eprintln!("[ERROR] {}", msg);
    exit(1);
}
