use super::{FileUnit, ModuleUnit, Visibility};
use std::path::PathBuf;

impl ModuleUnit {
    pub fn new(name: String, visibility: Visibility, document: Option<String>) -> Self {
        Self {
            name,
            declares: Vec::new(),
            visibility,
            document,
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            submodules: Vec::new(),
            source: None,
            attributes: Vec::new(),
        }
    }
}

impl FileUnit {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            document: None,
            declares: Vec::new(),
            modules: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            source: None,
        }
    }
}
