//! Configuration Management for cargo-optimize - Phase 1.1 Implementation
//! 
//! This module provides sophisticated configuration management using:
//! - Figment for layered configuration (defaults -> file -> env vars)
//! - toml_edit for preserving user's TOML formatting
//! - Automatic backup and restore capabilities
//! - Profile support (dev/test/release/bench)
//! - Percentage value parsing

use anyhow::{Context, Result};
use figment::providers::{Env, Format, Toml};
use figment::{Figment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{DocumentMut, Item, Table};
use tracing::{debug, info};
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Profile not found
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    
    /// Backup not found
    #[error("Backup not found: {0}")]
    BackupNotFound(PathBuf),
    
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Other error
    #[error("Configuration error: {0}")]
    Other(#[from] anyhow::Error),
}
