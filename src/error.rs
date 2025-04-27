use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Error types for the CodeBank library.
///
/// This enum represents all possible errors that can occur in the CodeBank library.
///
/// # Examples
///
/// ```
/// use codebank::Error;
/// use std::path::PathBuf;
///
/// // Create an IO error
/// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
/// let error = Error::Io(io_err);
/// assert!(matches!(error, Error::Io(_)));
///
/// // Create a parse error
/// let error = Error::Parse("invalid syntax".to_string());
/// assert!(matches!(error, Error::Parse(_)));
///
/// // Create a file not found error
/// let error = Error::FileNotFound(PathBuf::from("missing.rs"));
/// assert!(matches!(error, Error::FileNotFound(_)));
/// ```
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

/// Result type alias for CodeBank operations.
///
/// This type is used throughout the CodeBank library to handle operations
/// that can fail with a [`Error`].
///
/// # Examples
///
/// ```
/// use codebank::{Result, Error};
/// use std::path::PathBuf;
///
/// fn example_operation() -> Result<String> {
///     // Simulate a failing operation
///     Err(Error::FileNotFound(PathBuf::from("missing.rs")))
/// }
///
/// // Handle the result
/// match example_operation() {
///     Ok(content) => println!("Success: {}", content),
///     Err(e) => println!("Operation failed: {}", e),
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;
