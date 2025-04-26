use std::ops::{Deref, DerefMut};

use crate::{CParser, Error, Result};
use tree_sitter::Parser;

impl CParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_c::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::UnsupportedLanguage(e.to_string()))?;
        Ok(Self { parser })
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
