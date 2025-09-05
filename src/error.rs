//! Error types for cargo-optimize

use thiserror::Error;

/// Result type for cargo-optimize operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for cargo-optimize
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    /// Cargo metadata error
    #[error("Cargo metadata error: {0}")]
    CargoMetadata(#[from] cargo_metadata::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Detection error
    #[error("Detection error: {0}")]
    Detection(String),

    /// Optimization error
    #[error("Optimization error: {0}")]
    Optimization(String),

    /// Linker error
    #[error("Linker error: {0}")]
    Linker(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Project analysis error
    #[error("Project analysis error: {0}")]
    Analysis(String),

    /// Unsupported platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Invalid project structure
    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a detection error
    pub fn detection(msg: impl Into<String>) -> Self {
        Self::Detection(msg.into())
    }

    /// Create an optimization error
    pub fn optimization(msg: impl Into<String>) -> Self {
        Self::Optimization(msg.into())
    }

    /// Create a linker error
    pub fn linker(msg: impl Into<String>) -> Self {
        Self::Linker(msg.into())
    }

    /// Create a cache error
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    /// Create an analysis error
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis(msg.into())
    }

    /// Create an unsupported platform error
    pub fn unsupported_platform(msg: impl Into<String>) -> Self {
        Self::UnsupportedPlatform(msg.into())
    }

    /// Create a missing dependency error
    pub fn missing_dependency(msg: impl Into<String>) -> Self {
        Self::MissingDependency(msg.into())
    }

    /// Create an invalid project error
    pub fn invalid_project(msg: impl Into<String>) -> Self {
        Self::InvalidProject(msg.into())
    }

    /// Create a permission denied error
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::MissingDependency(_) | Self::Cache(_) | Self::Linker(_) | Self::Detection(_)
        )
    }

    /// Get a user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            Self::UnsupportedPlatform(_) => {
                format!("{}\nPlease report this issue at https://github.com/yourusername/cargo-optimize/issues", self)
            }
            Self::MissingDependency(dep) => {
                format!(
                    "Missing dependency: {}\nTry installing it with your package manager",
                    dep
                )
            }
            Self::PermissionDenied(path) => {
                format!("Permission denied: {}\nMake sure you have write access to the project directory", path)
            }
            _ => self.to_string(),
        }
    }
}
