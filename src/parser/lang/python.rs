use std::ops::{Deref, DerefMut};

use crate::{Error, PythonParser, Result};
use tree_sitter::Parser;

impl PythonParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::UnsupportedLanguage(e.to_string()))?;
        Ok(Self { parser })
    }
}

impl Deref for PythonParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for PythonParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
