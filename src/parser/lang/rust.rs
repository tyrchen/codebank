use crate::{Error, Result, RustParser};
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
