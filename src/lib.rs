mod error;
mod parser;

use std::path::Path;

pub use error::{Error, Result};
pub use parser::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BankStrategy {
    /// Generate the full code bank for the given directory using default settings
    #[default]
    Default,
    /// Generate the code bank without tests
    NoTests,
    /// Generate a summary, skip all non public units, for functions, only contain signature and skip the body
    Summary,
}

/// Trait to generate a code bank for a given directory. If the language for a given file is not supported, it will be skipped.
pub trait Bank {
    /// Generate a summary for the given directory using default settings
    fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String>;
}
