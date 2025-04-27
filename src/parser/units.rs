use super::{FileUnit, ModuleUnit, Visibility};
use std::path::PathBuf;

/// Implementation of ModuleUnit.
///
/// # Examples
///
/// ```
/// use codebank::{ModuleUnit, Visibility};
///
/// // Create a new public module
/// let module = ModuleUnit::new(
///     "example".to_string(),
///     Visibility::Public,
///     Some("Module documentation".to_string()),
/// );
///
/// assert_eq!(module.name, "example");
/// assert!(matches!(module.visibility, Visibility::Public));
/// assert_eq!(module.document, Some("Module documentation".to_string()));
/// assert!(module.functions.is_empty());
/// assert!(module.structs.is_empty());
/// assert!(module.traits.is_empty());
/// assert!(module.impls.is_empty());
/// assert!(module.submodules.is_empty());
/// ```
impl ModuleUnit {
    /// Creates a new module unit with the given name, visibility, and documentation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module
    /// * `visibility` - The visibility level of the module
    /// * `document` - Optional documentation for the module
    ///
    /// # Examples
    ///
    /// ```
    /// use codebank::{ModuleUnit, Visibility};
    ///
    /// let module = ModuleUnit::new(
    ///     "my_module".to_string(),
    ///     Visibility::Public,
    ///     Some("Module docs".to_string()),
    /// );
    ///
    /// assert_eq!(module.name, "my_module");
    /// ```
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

/// Implementation of FileUnit.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use codebank::FileUnit;
///
/// // Create a new file unit
/// let file = FileUnit::new(PathBuf::from("src/lib.rs"));
///
/// assert_eq!(file.path, PathBuf::from("src/lib.rs"));
/// assert!(file.document.is_none());
/// assert!(file.declares.is_empty());
/// assert!(file.modules.is_empty());
/// assert!(file.functions.is_empty());
/// assert!(file.structs.is_empty());
/// assert!(file.traits.is_empty());
/// assert!(file.impls.is_empty());
/// assert!(file.source.is_none());
/// ```
impl FileUnit {
    /// Creates a new file unit with the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use codebank::FileUnit;
    ///
    /// let file = FileUnit::new(PathBuf::from("src/main.rs"));
    /// assert_eq!(file.path, PathBuf::from("src/main.rs"));
    /// ```
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
