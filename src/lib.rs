//! # CodeBank
//!
//! `codebank` is a powerful documentation generator that creates structured markdown documentation
//! from your codebase. It supports multiple programming languages and provides flexible strategies
//! for documentation generation.
//!
//! ## Features
//!
//! - **Multi-language Support**: Parse and document Rust, Python, TypeScript, and C code
//! - **Flexible Strategies**: Choose between different documentation strategies:
//!   - `Default`: Complete code representation with full implementations
//!   - `NoTests`: Exclude test-related code for cleaner documentation
//!   - `Summary`: Generate public interface documentation only
//! - **Intelligent Parsing**: Uses tree-sitter for accurate code parsing and analysis
//! - **Customizable Output**: Control what gets included in your documentation
//!
//! ## Quick Start
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, CodeBank, Result};
//! use std::path::Path;
//!
//! fn main() -> Result<()> {
//!     // Create a new code bank generator
//!     let code_bank = CodeBank::try_new()?;
//!
//!     // Generate documentation for your source directory
//!     let content = code_bank.generate(
//!         Path::new("src"),
//!         BankStrategy::Default
//!     )?;
//!
//!     println!("Generated documentation:\n{}", content);
//!     Ok(())
//! }
//! ```
//!
//! ## Documentation Strategies
//!
//! ### Default Strategy
//!
//! The default strategy includes all code elements with their complete implementations:
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, CodeBank, Result};
//! use std::path::Path;
//!
//! # fn main() -> Result<()> {
//! let code_bank = CodeBank::try_new()?;
//! let content = code_bank.generate(Path::new("src"), BankStrategy::Default)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### NoTests Strategy
//!
//! Exclude test-related code for cleaner documentation:
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, CodeBank, Result};
//! use std::path::Path;
//!
//! # fn main() -> Result<()> {
//! let code_bank = CodeBank::try_new()?;
//! let content = code_bank.generate(Path::new("src"), BankStrategy::NoTests)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Summary Strategy
//!
//! Generate documentation for public interfaces only:
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, CodeBank, Result};
//! use std::path::Path;
//!
//! # fn main() -> Result<()> {
//! let code_bank = CodeBank::try_new()?;
//! let content = code_bank.generate(Path::new("src"), BankStrategy::Summary)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Implementation
//!
//! You can implement the `Bank` trait for your own code bank generator:
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, Result};
//! use std::path::Path;
//!
//! struct MyCodeBank;
//!
//! impl Bank for MyCodeBank {
//!     fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String> {
//!         // Your implementation here
//!         Ok("# Code Bank\n\nCustom implementation".to_string())
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! The crate uses a custom `Result` type that wraps common error cases:
//!
//! ```rust
//! use codebank::{Bank, BankStrategy, CodeBank, Result, Error};
//!
//! # fn main() -> Result<()> {
//! let result = CodeBank::try_new()?.generate(
//!     std::path::Path::new("nonexistent"),
//!     BankStrategy::Default
//! );
//!
//! if let Err(err) = result {
//!     eprintln!("Failed to generate documentation: {}", err);
//! }
//! # Ok(())
//! # }
//! ```

mod bank;
mod error;
pub mod mcp;
mod parser;

use std::path::Path;

pub use bank::CodeBank;
pub use error::{Error, Result};
pub use parser::*;

/// Strategy for generating code bank documentation.
///
/// This enum controls how the code bank generator processes and formats the code.
///
/// # Examples
///
/// ```
/// use codebank::BankStrategy;
///
/// // Use default strategy for complete code representation
/// let strategy = BankStrategy::Default;
///
/// // Use NoTests strategy to exclude test code
/// let strategy = BankStrategy::NoTests;
///
/// // Use Summary strategy for public interface only
/// let strategy = BankStrategy::Summary;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BankStrategy {
    /// Generate the full code bank for the given directory using default settings.
    /// This includes all code elements with their complete implementations.
    ///
    /// # Examples
    ///
    /// ```
    /// use codebank::{Bank, BankStrategy, CodeBank};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let code_bank = CodeBank::try_new()?;
    ///
    /// // Generate complete documentation
    /// let content = code_bank.generate(
    ///     Path::new("src"),  // Use your source directory
    ///     BankStrategy::Default
    /// )?;
    ///
    /// assert!(content.contains("# Code Bank"));
    /// # Ok(())
    /// # }
    /// ```
    #[default]
    Default,

    /// Generate the code bank without tests.
    /// This excludes test modules, test functions, and other test-related code.
    ///
    /// # Examples
    ///
    /// ```
    /// use codebank::{Bank, BankStrategy, CodeBank, Result};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<()> {
    /// let code_bank = CodeBank::try_new()?;
    ///
    /// // Generate documentation excluding tests
    /// let content = code_bank.generate(
    ///     Path::new("src"),  // Use your source directory
    ///     BankStrategy::NoTests
    /// )?;
    ///
    /// // Content should not contain test-related code
    /// let lines = content.lines().collect::<Vec<_>>();
    /// assert!(!lines.iter().any(|line| line.starts_with(&"#[cfg(test)]")));
    /// assert!(!lines.iter().any(|line| line.starts_with(&"#[test]")));
    /// assert!(!lines.iter().any(|line| line.starts_with(&"mod tests {")));
    /// # Ok(())
    /// # }
    /// ```
    NoTests,

    /// Generate a summary, skip all non public units.
    /// For functions, only contain signature and skip the body.
    ///
    /// # Examples
    ///
    /// ```
    /// use codebank::{Bank, BankStrategy, CodeBank};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let code_bank = CodeBank::try_new()?;
    ///
    /// // Generate public interface summary
    /// let content = code_bank.generate(
    ///     Path::new("src"),  // Use your source directory
    ///     BankStrategy::Summary
    /// )?;
    ///
    /// // Content should contain function signatures but not implementations
    /// assert!(content.contains("{ ... }"));
    /// # Ok(())
    /// # }
    /// ```
    Summary,
}

/// Trait to generate a code bank for a given directory.
///
/// This trait is implemented by code bank generators to process source code
/// and generate documentation in a structured format. If the language for a
/// given file is not supported, it will be skipped.
///
/// # Examples
///
/// ```
/// use codebank::{Bank, BankStrategy, CodeBank};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new code bank generator
/// let code_bank = CodeBank::try_new()?;
///
/// // Generate documentation using the Bank trait
/// let content = code_bank.generate(
///     Path::new("src"),  // Use your source directory
///     BankStrategy::Default
/// )?;
///
/// // The generated content should be markdown
/// assert!(content.starts_with("# Code Bank"));
/// # Ok(())
/// # }
/// ```
///
/// # Custom Implementation
///
/// You can implement this trait for your own code bank generator:
///
/// ```
/// use codebank::{Bank, BankStrategy, Result};
/// use std::path::Path;
///
/// struct MyCodeBank;
///
/// impl Bank for MyCodeBank {
///     fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String> {
///         // Your implementation here
///         Ok("# Code Bank\n\nCustom implementation".to_string())
///     }
/// }
///
/// # fn main() -> Result<()> {
/// let my_bank = MyCodeBank;
/// let content = my_bank.generate(Path::new("."), BankStrategy::Default)?;
/// assert!(content.starts_with("# Code Bank"));
/// # Ok(())
/// # }
/// ```
pub trait Bank {
    /// Generate a summary for the given directory using the specified strategy.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The root directory to process
    /// * `strategy` - The strategy to use for generation
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the generated documentation as a string,
    /// or an error if the generation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The root directory does not exist
    /// * The root directory is not actually a directory
    /// * File reading or parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use codebank::{Bank, BankStrategy, CodeBank};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let code_bank = CodeBank::try_new()?;
    ///
    /// // Generate documentation for the src directory
    /// let content = code_bank.generate(
    ///     Path::new("src"),
    ///     BankStrategy::Default
    /// )?;
    ///
    /// println!("Generated documentation:\n{}", content);
    /// # Ok(())
    /// # }
    /// ```
    fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String>;
}
