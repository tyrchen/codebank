pub mod formatter;
mod lang;
mod units;

use crate::Result;
use std::path::{Path, PathBuf};

pub use formatter::Formatter;
pub use lang::{CppParser, PythonParser, RustParser, TypeScriptParser};

/// Represents visibility levels for code elements.
///
/// This enum is used to track the visibility of various code elements
/// such as functions, structs, and modules.
///
/// # Examples
///
/// ```
/// use codebank::Visibility;
///
/// // Public visibility
/// let vis = Visibility::Public;
/// assert!(matches!(vis, Visibility::Public));
///
/// // Private visibility
/// let vis = Visibility::Private;
/// assert!(matches!(vis, Visibility::Private));
///
/// // Crate visibility
/// let vis = Visibility::Crate;
/// assert!(matches!(vis, Visibility::Crate));
///
/// // Restricted visibility
/// let vis = Visibility::Restricted("super::module".to_string());
/// assert!(matches!(vis, Visibility::Restricted(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Public visibility (accessible from outside the module)
    #[default]
    Public,

    /// Private visibility (accessible only within the module)
    Private,

    /// Protected visibility (accessible within the module and its descendants)
    Protected,

    /// Crate visibility (accessible within the crate only)
    Crate,

    /// Visibility restricted to a specific path
    Restricted(String),
}

/// The language type supported by the parser.
///
/// # Examples
///
/// ```
/// use codebank::LanguageType;
///
/// // Check Rust files
/// assert!(matches!(LanguageType::Rust, LanguageType::Rust));
///
/// // Check Python files
/// assert!(matches!(LanguageType::Python, LanguageType::Python));
///
/// // Check TypeScript files
/// assert!(matches!(LanguageType::TypeScript, LanguageType::TypeScript));
///
/// // Check C files
/// assert!(matches!(LanguageType::Cpp, LanguageType::Cpp));
///
/// // Handle unknown types
/// assert!(matches!(LanguageType::Unknown, LanguageType::Unknown));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageType {
    /// Rust language
    Rust,
    /// Python language
    Python,
    /// TypeScript language
    TypeScript,
    /// C/C++ language
    Cpp,
    /// Unknown language (used for unsupported extensions)
    Unknown,
}

/// Trait for language-specific parsers.
///
/// This trait is implemented by parsers for different programming languages
/// to provide consistent parsing behavior.
///
/// # Examples
///
/// ```
/// use codebank::{LanguageParser, FileUnit, Result};
/// use std::path::{Path, PathBuf};
///
/// struct MyParser;
///
/// impl LanguageParser for MyParser {
///     fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
///         // Simple implementation that creates an empty FileUnit
///         Ok(FileUnit::new(file_path.to_path_buf()))
///     }
/// }
///
/// # fn main() -> Result<()> {
/// let mut parser = MyParser;
/// let file_unit = parser.parse_file(Path::new("example.rs"))?;
/// assert_eq!(file_unit.path, PathBuf::from("example.rs"));
/// # Ok(())
/// # }
/// ```
pub trait LanguageParser {
    /// Parse a file into a FileUnit
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit>;
}

/// Represents a file in the code.
///
/// This struct contains all the parsed information about a source code file,
/// including its structure, contents, and metadata.
///
/// # Examples
///
/// ```
/// use codebank::{FileUnit, Visibility, FunctionUnit};
/// use std::path::PathBuf;
///
/// // Create a new file unit
/// let mut file = FileUnit::new(PathBuf::from("example.rs"));
///
/// // Add documentation
/// file.doc = Some("Example file documentation".to_string());
///
/// // Add a function
/// let function = FunctionUnit {
///     name: "example_function".to_string(),
///     visibility: Visibility::Public,
///     doc: Some("Function documentation".to_string()),
///     signature: Some("fn example_function()".to_string()),
///     body: Some("{ println!(\"Hello\"); }".to_string()),
///     source: Some("fn example_function() { println!(\"Hello\"); }".to_string()),
///     attributes: vec![],
/// };
/// file.functions.push(function);
///
/// assert_eq!(file.path, PathBuf::from("example.rs"));
/// assert!(file.doc.is_some());
/// assert!(!file.functions.is_empty());
/// ```
#[derive(Debug, Default)]
pub struct FileUnit {
    /// The path to the file
    pub path: PathBuf,

    /// File-level documentation
    pub doc: Option<String>,

    /// The declares in the file, e.g. imports, use statements, mod statements, c includes, python/js imports, etc.
    pub declares: Vec<DeclareStatements>,

    /// The modules contained in the file
    pub modules: Vec<ModuleUnit>,

    /// Top-level functions not in a module
    pub functions: Vec<FunctionUnit>,

    /// Top-level structs not in a module
    pub structs: Vec<StructUnit>,

    /// Top-level traits not in a module
    pub traits: Vec<TraitUnit>,

    /// Top-level implementation blocks
    pub impls: Vec<ImplUnit>,

    /// Source code of the entire file
    pub source: Option<String>,
}

/// Represents declarations in source code.
///
/// This struct is used to store various types of declarations found in source files,
/// such as imports, use statements, and module declarations.
///
/// # Examples
///
/// ```
/// use codebank::{DeclareStatements, DeclareKind};
///
/// // Create an import declaration
/// let import = DeclareStatements {
///     source: "use std::io;".to_string(),
///     kind: DeclareKind::Import,
/// };
/// assert!(matches!(import.kind, DeclareKind::Import));
///
/// // Create a module declaration
/// let module = DeclareStatements {
///     source: "mod example;".to_string(),
///     kind: DeclareKind::Mod,
/// };
/// assert!(matches!(module.kind, DeclareKind::Mod));
/// ```
#[derive(Debug, Default)]
pub struct DeclareStatements {
    /// The source code of the declaration
    pub source: String,
    /// The kind of declaration
    pub kind: DeclareKind,
}

/// The kind of declaration statement.
///
/// # Examples
///
/// ```
/// use codebank::DeclareKind;
///
/// // Import declaration
/// let kind = DeclareKind::Import;
/// assert!(matches!(kind, DeclareKind::Import));
///
/// // Use declaration
/// let kind = DeclareKind::Use;
/// assert!(matches!(kind, DeclareKind::Use));
///
/// // Module declaration
/// let kind = DeclareKind::Mod;
/// assert!(matches!(kind, DeclareKind::Mod));
///
/// // Other declaration types
/// let kind = DeclareKind::Other("macro_rules".to_string());
/// assert!(matches!(kind, DeclareKind::Other(_)));
/// ```
#[derive(Debug, Default, PartialEq)]
pub enum DeclareKind {
    #[default]
    Import,
    Use,
    Mod,
    Other(String),
}

/// Represents a module in the code
#[derive(Debug, Default)]
pub struct ModuleUnit {
    /// The name of the module
    pub name: String,

    /// Attributes applied to the module
    pub attributes: Vec<String>,

    /// The document for the module
    pub doc: Option<String>,

    /// The declares in the module, e.g. imports, use statements, mod statements, c includes, python/js imports, etc.
    pub declares: Vec<DeclareStatements>,

    /// The visibility of the module
    pub visibility: Visibility,

    /// Functions defined in the module
    pub functions: Vec<FunctionUnit>,

    /// Structs defined in the module
    pub structs: Vec<StructUnit>,

    /// Traits defined in the module
    pub traits: Vec<TraitUnit>,

    /// Implementation blocks defined in the module
    pub impls: Vec<ImplUnit>,

    /// Sub-modules defined in the module
    pub submodules: Vec<ModuleUnit>,

    /// Source code of the module declaration
    pub source: Option<String>,
}

/// Represents a function or method in the code
#[derive(Debug, Default, Clone)]
pub struct FunctionUnit {
    /// The name of the function
    pub name: String,

    /// Attributes applied to the function
    pub attributes: Vec<String>,

    /// The visibility of the function
    pub visibility: Visibility,

    /// The documentation for the function
    pub doc: Option<String>,

    /// The function signature (without body)
    pub signature: Option<String>,

    /// The function body
    pub body: Option<String>,

    /// The source code of the function
    pub source: Option<String>,
}

/// Represents a struct or class in the code
#[derive(Debug, Default)]
pub struct StructUnit {
    /// The name of the struct
    pub name: String,

    /// Attributes applied to the struct
    pub attributes: Vec<String>,

    /// The visibility of the struct
    pub visibility: Visibility,

    /// The documentation for the struct
    pub doc: Option<String>,

    /// struct head, e.g. struct Type, class Type, etc.
    pub head: String,

    /// The fields of the struct
    pub fields: Vec<FieldUnit>,

    /// The methods implemented for the struct
    pub methods: Vec<FunctionUnit>,

    /// The source code of the struct
    pub source: Option<String>,
}

/// Represents a field in a struct
#[derive(Debug, Default, Clone)]
pub struct FieldUnit {
    /// The name of the field
    pub name: String,
    /// documentation for the field
    pub doc: Option<String>,
    /// attributes applied to the field
    pub attributes: Vec<String>,
    /// the source code of the field
    pub source: Option<String>,
}

/// Represents a trait or interface in the code
#[derive(Debug, Default, Clone)]
pub struct TraitUnit {
    /// The name of the trait
    pub name: String,

    /// Attributes applied to the struct
    pub attributes: Vec<String>,

    /// The visibility of the trait
    pub visibility: Visibility,

    /// The documentation for the trait
    pub doc: Option<String>,

    /// The methods declared in the trait
    pub methods: Vec<FunctionUnit>,

    /// The source code of the trait
    pub source: Option<String>,
}

/// Represents an implementation block in the code, not all languages need this
#[derive(Debug, Default, Clone)]
pub struct ImplUnit {
    /// Attributes applied to the trait
    pub attributes: Vec<String>,

    /// The documentation for the implementation block
    pub doc: Option<String>,

    /// impl head, e.g. impl Trait for Type or impl Type
    pub head: String,

    /// The methods implemented in this block
    pub methods: Vec<FunctionUnit>,

    /// The source code of the implementation block
    pub source: Option<String>,
}

impl Visibility {
    pub fn as_str(&self, language: LanguageType) -> &str {
        match (self, language) {
            (Visibility::Public, LanguageType::Rust) => "pub",
            (Visibility::Crate, LanguageType::Rust) => "pub(crate)",
            (_, LanguageType::Rust) => "",
            (_, LanguageType::Python) => "",
            (_, LanguageType::TypeScript) => "",
            (_, LanguageType::Cpp) => "",
            (_, LanguageType::Unknown) => "",
        }
    }
}
