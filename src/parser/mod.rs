mod lang;

use crate::Result;
use std::path::{Path, PathBuf};

pub use lang::{CParser, PythonParser, RustParser, TypeScriptParser};
/// Base trait for all code units in the intermediate representation
pub trait CodeUnit {
    /// Get the name of the code unit
    fn name(&self) -> &str;

    /// Get the visibility of the code unit
    fn visibility(&self) -> &Visibility;

    /// Get the documentation for the code unit, if any
    fn documentation(&self) -> Option<&str>;

    /// Get the source code for the code unit, if available
    fn source_code(&self) -> Option<&str>;

    /// Get the type of the code unit as a string
    fn unit_type(&self) -> &str;
}

/// Represents visibility levels for code elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Public visibility (accessible from outside the module)
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

/// The language type supported by the parser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageType {
    /// Rust language
    Rust,
    /// Python language
    Python,
    /// Unknown language (used for unsupported extensions)
    Unknown,
}

/// Trait for language-specific parsers
pub trait LanguageParser {
    /// Parse a file into a FileUnit
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit>;
}

/// Represents a file in the code
#[derive(Debug)]
pub struct FileUnit {
    /// The path to the file
    pub path: PathBuf,

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

/// Represents a module in the code
#[derive(Debug)]
pub struct ModuleUnit {
    /// The name of the module
    pub name: String,

    /// The visibility of the module
    pub visibility: Visibility,

    /// The documentation for the module
    pub documentation: Option<String>,

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
#[derive(Debug)]
pub struct FunctionUnit {
    /// The name of the function
    pub name: String,

    /// The visibility of the function
    pub visibility: Visibility,

    /// The documentation for the function
    pub documentation: Option<String>,

    /// The parameters of the function
    pub parameters: Vec<ParameterUnit>,

    /// The return type of the function
    pub return_type: Option<String>,

    /// The source code of the function
    pub source: Option<String>,
}

/// Represents a struct or class in the code
#[derive(Debug)]
pub struct StructUnit {
    /// The name of the struct
    pub name: String,

    /// The visibility of the struct
    pub visibility: Visibility,

    /// The documentation for the struct
    pub documentation: Option<String>,

    /// The fields in the struct
    pub fields: Vec<FieldUnit>,

    /// The methods implemented for the struct
    pub methods: Vec<FunctionUnit>,

    /// The source code of the struct
    pub source: Option<String>,
}

/// Represents a field in a struct
#[derive(Debug)]
pub struct FieldUnit {
    /// The name of the field
    pub name: String,

    /// The visibility of the field
    pub visibility: Visibility,

    /// The type of the field
    pub field_type: String,

    /// The documentation for the field
    pub documentation: Option<String>,
}

/// Represents a trait or interface in the code
#[derive(Debug)]
pub struct TraitUnit {
    /// The name of the trait
    pub name: String,

    /// The visibility of the trait
    pub visibility: Visibility,

    /// The documentation for the trait
    pub documentation: Option<String>,

    /// The methods declared in the trait
    pub methods: Vec<FunctionUnit>,

    /// The source code of the trait
    pub source: Option<String>,
}

/// Represents an implementation block in the code
#[derive(Debug)]
pub struct ImplUnit {
    /// The name of the type being implemented
    pub target_type: String,

    /// The trait being implemented, if any
    pub trait_name: Option<String>,

    /// The documentation for the implementation block
    pub documentation: Option<String>,

    /// The methods implemented in this block
    pub methods: Vec<FunctionUnit>,

    /// The source code of the implementation block
    pub source: Option<String>,
}

/// Represents a parameter in a function
#[derive(Debug)]
pub struct ParameterUnit {
    /// The name of the parameter
    pub name: String,

    /// The type of the parameter
    pub parameter_type: String,

    /// Whether the parameter is self (in Rust methods)
    pub is_self: bool,
}
