//! # cargo-optimize
//!
//! Automatically optimize Rust build times with zero configuration.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [build-dependencies]
//! cargo-optimize = "0.1"
//! ```
//!
//! In your `build.rs`:
//! ```no_run
//! cargo_optimize::auto_configure();
//! ```
//!
//! That's it! Your builds will automatically be optimized.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

/// MVP module providing automatic linker optimization.
/// 
/// This module contains the minimal viable product implementation
/// that focuses solely on detecting and configuring fast linkers.
pub mod mvp;

// Re-export MVP function as the main interface for now
pub use mvp::auto_configure_mvp as auto_configure;

/// Configuration module for build optimization settings.
/// 
/// Provides enhanced configuration management with support for:
/// - Safe merging of existing configurations
/// - Multi-file config support (.cargo/config.toml + cargo-optimize.toml)
/// - Profile system (dev/test/release/bench)
/// - Backup and rollback capabilities
pub mod config;

/// Hardware detection and system information module.
/// 
/// Provides platform-aware hardware detection with:
/// - CPU and memory information
/// - Disk space detection
/// - Percentage-based calculations for resource allocation
/// - Fallback values for failure scenarios
pub mod hardware;

/// Project analysis module using cargo_metadata and guppy.
/// 
/// Provides comprehensive project analysis including:
/// - Workspace structure detection
/// - Dependency graph analysis
/// - Feature analysis and optimization suggestions
/// - Build target detection
/// - Build metrics collection
pub mod analysis;

/// Utility functions for output and common operations.
/// 
/// Provides helper functions for displaying messages to users.
pub mod utils {
    /// Print an error message to stderr.
    /// 
    /// # Arguments
    /// * `_msg` - The error message to display
    pub fn print_error(_msg: &str) { eprintln!("{}", _msg); }
    /// Print an informational message to stdout.
    /// 
    /// # Arguments
    /// * `_msg` - The info message to display
    pub fn print_info(_msg: &str) { println!("{}", _msg); }
    /// Print a success message with a checkmark prefix.
    /// 
    /// # Arguments
    /// * `_msg` - The success message to display
    pub fn print_success(_msg: &str) { println!("✓ {}", _msg); }
}

/// Main optimizer struct for managing build optimizations.
/// 
/// Currently a placeholder that will be expanded with full
/// optimization logic in future versions.
pub struct Optimizer;

/// Build cache configuration and management.
/// 
/// Handles integration with sccache, ccache, and other
/// build caching systems.
pub mod cache {
    /// Configuration for build cache systems.
    /// 
    /// Will manage settings for various caching backends.
    pub struct CacheConfig;
}

/// Get the version of cargo-optimize
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
