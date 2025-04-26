use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error types for the CodeBank library
#[derive(Error, Debug)]
pub enum Error {
    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Parse error for syntax parsing failures
    #[error("Parse error: {0}")]
    Parse(String),

    /// Tree-sitter error for tree-sitter specific failures
    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    /// File not found error
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Directory not found error
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Unsupported language error
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Result type alias for CodeBank operations
pub type Result<T> = std::result::Result<T, Error>;
