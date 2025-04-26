use std::ops::{Deref, DerefMut};

use crate::{Error, Result, TypeScriptParser};
use tree_sitter::Parser;

impl TypeScriptParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::UnsupportedLanguage(e.to_string()))?;
        Ok(Self { parser })
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
