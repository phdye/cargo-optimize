//! # cargo-optimize
//!
//! Automatically optimize Rust build times with zero configuration.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod config;
pub mod detector;
pub mod optimizer;
pub mod analyzer;
pub mod linker;
pub mod cache;
pub mod error;
pub mod profile;
pub mod utils;

pub use config::{Config, OptimizationLevel, OptimizationFeature};
pub use error::{Error, Result};
pub use optimizer::Optimizer;

use std::env;
use std::path::PathBuf;
use tracing::{info, warn};

/// Automatically configure and apply all optimizations.
pub fn auto_configure() {
    if let Err(e) = auto_configure_impl() {
        eprintln!("cargo-optimize: Failed to apply optimizations: {}", e);
        eprintln!("cargo-optimize: Continuing with default settings...");
    }
}

fn auto_configure_impl() -> Result<()> {
    // Initialize tracing
    init_logging();
    
    info!("Starting cargo-optimize auto-configuration");
    
    // Skip if disabled
    if env::var("CARGO_OPTIMIZE_DISABLE").is_ok() {
        info!("cargo-optimize disabled via CARGO_OPTIMIZE_DISABLE environment variable");
        return Ok(());
    }
    
    // Get project root
    let project_root = get_project_root()?;
    
    // Create optimizer
    let mut optimizer = Optimizer::new(project_root)?;
    
    // Run optimization
    optimizer.optimize()?;
    
    info!("cargo-optimize configuration complete");
    Ok(())
}

/// Initialize logging system
fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("cargo_optimize=info"));
    
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init()
        .ok();
}

/// Get the project root directory
fn get_project_root() -> Result<PathBuf> {
    // Try CARGO_MANIFEST_DIR first (set during build)
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(manifest_dir));
    }
    
    // Fall back to current directory
    env::current_dir()
        .map_err(|e| Error::Io(e))
}

/// Apply optimizations with custom configuration.
pub fn optimize_with_config(config: Config) -> Result<()> {
    init_logging();
    
    let project_root = get_project_root()?;
    let mut optimizer = Optimizer::with_config(project_root, config)?;
    optimizer.optimize()?;
    
    Ok(())
}

/// Check if optimizations are currently active
pub fn is_optimized() -> bool {
    env::var("CARGO_OPTIMIZE_ACTIVE").is_ok()
}

/// Get the version of cargo-optimize
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
