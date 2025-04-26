use crate::{CParser, Error, FileUnit, LanguageParser, Result};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::Parser;

impl CParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_c::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for CParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        // Basic implementation for now - just reads the file and returns an empty FileUnit
        // In a production implementation, this would parse C code using tree-sitter
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;

        // Parse the file with tree-sitter
        let tree = self
            .parse(source_code.as_bytes(), None)
            .ok_or_else(|| Error::Parse("Failed to parse file".to_string()))?;

        let root_node = tree.root_node();

        // Extract file-level documentation if present (comments at the beginning)
        let mut document = None;
        let mut cursor = root_node.walk();
        let mut first_comments = Vec::new();

        for node in root_node.children(&mut cursor) {
            if node.kind() == "comment" {
                if let Ok(comment) = node.utf8_text(source_code.as_bytes()) {
                    let cleaned = comment
                        .trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim()
                        .to_string();
                    first_comments.push(cleaned);
                }
            } else {
                break; // Stop at first non-comment
            }
        }

        if !first_comments.is_empty() {
            document = Some(first_comments.join("\n"));
        }

        // Extract declares (includes, defines, etc.)
        let mut declares = Vec::new();
        cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            match node.kind() {
                "preproc_include" => {
                    if let Ok(include_text) = node.utf8_text(source_code.as_bytes()) {
                        declares.push(crate::DeclareStatements {
                            source: include_text.to_string(),
                            kind: crate::DeclareKind::Import,
                        });
                    }
                }
                "preproc_def" | "preproc_function_def" => {
                    if let Ok(def_text) = node.utf8_text(source_code.as_bytes()) {
                        declares.push(crate::DeclareStatements {
                            source: def_text.to_string(),
                            kind: crate::DeclareKind::Other("define".to_string()),
                        });
                    }
                }
                _ => continue,
            }
        }

        Ok(FileUnit {
            path: file_path.to_path_buf(),
            document,
            declares,
            source: Some(source_code),
            ..Default::default()
        })
    }
}

impl Deref for CParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for CParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
