use crate::{Error, FileUnit, LanguageParser, Result, TypeScriptParser};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::Parser;

impl TypeScriptParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for TypeScriptParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        // Basic implementation for now - just reads the file and returns an empty FileUnit
        // In a production implementation, this would parse TypeScript code using tree-sitter
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;

        Ok(FileUnit {
            path: file_path.to_path_buf(),
            source: Some(source_code),
            ..Default::default()
        })
    }
}

impl Deref for TypeScriptParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for TypeScriptParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
