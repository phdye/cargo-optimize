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
/// Currently contains placeholder implementations that will be
/// replaced with full functionality in future versions.
pub mod config {
    /// Main configuration structure for cargo-optimize.
    /// 
    /// This will eventually hold all optimization settings.
    #[derive(Debug, Default)]
    pub struct Config;
    
    /// Optimization level for build configuration.
    /// 
    /// Determines how aggressively optimizations are applied.
    #[derive(Debug)]
    pub enum OptimizationLevel {
        /// Conservative optimization - minimal changes, maximum compatibility.
        Conservative,
        /// Balanced optimization - good performance with reasonable safety.
        Balanced,
        /// Aggressive optimization - maximum performance, may affect stability.
        Aggressive,
    }


}

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

/// Project analysis module for understanding build complexity.
/// 
/// Will eventually analyze project structure and dependencies
/// to make intelligent optimization decisions.
pub mod analyzer {
    /// Analysis results for a Rust project.
    /// 
    /// Will contain metrics about project size, dependencies,
    /// and build complexity.
    pub struct ProjectAnalysis;
}

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
