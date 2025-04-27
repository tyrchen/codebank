use crate::{
    parser::{
        formatter::Formatter, CParser, FileUnit, LanguageParser, LanguageType, PythonParser,
        RustParser, TypeScriptParser,
    },
    Bank, BankStrategy, Error, Result,
};
use ignore::Walk;
use std::{ffi::OsStr, path::Path};

/// The code bank generator implementation
pub struct CodeBank {
    rust_parser: RustParser,
    python_parser: PythonParser,
    typescript_parser: TypeScriptParser,
    c_parser: CParser,
}

impl CodeBank {
    /// Create a new code bank generator
    pub fn try_new() -> Result<Self> {
        let rust_parser = RustParser::try_new()?;
        let python_parser = PythonParser::try_new()?;
        let typescript_parser = TypeScriptParser::try_new()?;
        let c_parser = CParser::try_new()?;

        Ok(Self {
            rust_parser,
            python_parser,
            typescript_parser,
            c_parser,
        })
    }

    /// Detect the language type from a file extension
    fn detect_language(&self, path: &Path) -> Option<LanguageType> {
        match path.extension().and_then(OsStr::to_str) {
            Some("rs") => Some(LanguageType::Rust),
            Some("py") => Some(LanguageType::Python),
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => Some(LanguageType::TypeScript),
            Some("c") | Some("h") => Some(LanguageType::C),
            _ => None,
        }
    }

    /// Get the language name for markdown code blocks
    fn get_language_name(&self, path: &Path) -> &str {
        match path.extension().and_then(OsStr::to_str) {
            Some("rs") => "rust",
            Some("py") => "python",
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => "typescript",
            Some("c") | Some("h") => "c",
            _ => "",
        }
    }

    /// Parse a single file using the appropriate language parser
    fn parse_file(&mut self, file_path: &Path) -> Result<Option<FileUnit>> {
        match self.detect_language(file_path) {
            Some(LanguageType::Rust) => self.rust_parser.parse_file(file_path).map(Some),
            Some(LanguageType::Python) => self.python_parser.parse_file(file_path).map(Some),
            Some(LanguageType::TypeScript) => {
                self.typescript_parser.parse_file(file_path).map(Some)
            }
            Some(LanguageType::C) => self.c_parser.parse_file(file_path).map(Some),
            Some(LanguageType::Unknown) => Ok(None),
            None => Ok(None),
        }
    }
}

impl Bank for CodeBank {
    fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String> {
        // Make sure the root directory exists
        if !root_dir.exists() {
            return Err(Error::DirectoryNotFound(root_dir.to_path_buf()));
        }

        if !root_dir.is_dir() {
            return Err(Error::InvalidConfig(format!(
                "{} is not a directory",
                root_dir.display()
            )));
        }

        // Initialize output
        let mut output = String::new();
        output.push_str("# Code Bank\n\n");

        // Clone self to make it mutable (needed for parsers)
        let mut code_bank = self.try_clone()?;

        // Use a vector to collect all file units so we can sort them
        let mut file_units = Vec::new();

        // Walk through all files in the directory
        for entry in Walk::new(root_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                // Try to parse the file with the appropriate parser
                if let Ok(Some(file_unit)) = code_bank.parse_file(path) {
                    file_units.push(file_unit);
                }
            }
        }

        // Sort file units by path for consistent output
        file_units.sort_by(|a, b| a.path.cmp(&b.path));

        // Format each file unit as markdown using the Formatter trait
        for file_unit in &file_units {
            // Get the relative path of the file
            let relative_path = file_unit
                .path
                .strip_prefix(root_dir)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| file_unit.path.display().to_string());

            // Add the file header
            output.push_str(&format!("## {}\n", relative_path));

            // Add the code block with appropriate language
            let lang = code_bank.get_language_name(&file_unit.path);
            output.push_str(&format!("```{}\n", lang));

            // Format the file unit using the Formatter trait
            let formatted_content = file_unit.format(
                &strategy,
                code_bank
                    .detect_language(&file_unit.path)
                    .unwrap_or(LanguageType::Unknown),
            )?;
            output.push_str(&formatted_content);

            output.push_str("```\n\n");
        }

        Ok(output)
    }
}

impl CodeBank {
    // Helper method to clone the CodeBank for mutability
    fn try_clone(&self) -> Result<Self> {
        CodeBank::try_new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_language() {
        let code_bank = CodeBank::try_new().unwrap();

        // Test Rust files
        let rust_path = PathBuf::from("test.rs");
        assert_eq!(
            code_bank.detect_language(&rust_path),
            Some(LanguageType::Rust)
        );

        // Test Python files
        let python_path = PathBuf::from("test.py");
        assert_eq!(
            code_bank.detect_language(&python_path),
            Some(LanguageType::Python)
        );

        // Test TypeScript files
        let ts_path = PathBuf::from("test.ts");
        assert_eq!(
            code_bank.detect_language(&ts_path),
            Some(LanguageType::TypeScript)
        );

        let tsx_path = PathBuf::from("test.tsx");
        assert_eq!(
            code_bank.detect_language(&tsx_path),
            Some(LanguageType::TypeScript)
        );

        let js_path = PathBuf::from("test.js");
        assert_eq!(
            code_bank.detect_language(&js_path),
            Some(LanguageType::TypeScript)
        );

        let jsx_path = PathBuf::from("test.jsx");
        assert_eq!(
            code_bank.detect_language(&jsx_path),
            Some(LanguageType::TypeScript)
        );

        // Test C files
        let c_path = PathBuf::from("test.c");
        assert_eq!(code_bank.detect_language(&c_path), Some(LanguageType::C));

        let h_path = PathBuf::from("test.h");
        assert_eq!(code_bank.detect_language(&h_path), Some(LanguageType::C));

        // Test unsupported files
        let unsupported_path = PathBuf::from("test.txt");
        assert_eq!(code_bank.detect_language(&unsupported_path), None);
    }

    #[test]
    fn test_get_language_name() {
        let code_bank = CodeBank::try_new().unwrap();

        // Test Rust files
        let rust_path = PathBuf::from("test.rs");
        assert_eq!(code_bank.get_language_name(&rust_path), "rust");

        // Test Python files
        let python_path = PathBuf::from("test.py");
        assert_eq!(code_bank.get_language_name(&python_path), "python");

        // Test TypeScript files
        let ts_path = PathBuf::from("test.ts");
        assert_eq!(code_bank.get_language_name(&ts_path), "typescript");

        // Test C files
        let c_path = PathBuf::from("test.c");
        assert_eq!(code_bank.get_language_name(&c_path), "c");

        // Test unsupported files
        let unsupported_path = PathBuf::from("test.txt");
        assert_eq!(code_bank.get_language_name(&unsupported_path), "");
    }
}
