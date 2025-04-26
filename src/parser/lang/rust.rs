use crate::{Error, Result, RustParser};
use std::ops::{Deref, DerefMut};
use tree_sitter::Parser;

impl RustParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::UnsupportedLanguage(e.to_string()))?;
        Ok(Self { parser })
    }
}

impl Deref for RustParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for RustParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
