# Simplify

I've greatly simplified the code unit structure. Please update the code to match the new structure. The parser should be simplified as well. Formatters should be updated too. Please also update test cases.

```rust

/// Represents a file in the code
#[derive(Debug, Default)]
pub struct FileUnit {
    /// The path to the file
    pub path: PathBuf,

    pub document: Option<String>,
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

#[derive(Debug, Default)]
pub struct DeclareStatements {
    pub source: String,
    pub kind: DeclareKind,
}

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
    pub document: Option<String>,

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
#[derive(Debug, Default)]
pub struct FunctionUnit {
    /// The name of the function
    pub name: String,

    /// Attributes applied to the function
    pub attributes: Vec<String>,

    /// The visibility of the function
    pub visibility: Visibility,

    /// The documentation for the function
    pub documentation: Option<String>,

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
    pub documentation: Option<String>,

    /// The methods implemented for the struct
    pub methods: Vec<FunctionUnit>,

    /// The source code of the struct
    pub source: Option<String>,
}

/// Represents a trait or interface in the code
#[derive(Debug, Default)]
pub struct TraitUnit {
    /// The name of the trait
    pub name: String,

    /// Attributes applied to the struct
    pub attributes: Vec<String>,

    /// The visibility of the trait
    pub visibility: Visibility,

    /// The documentation for the trait
    pub documentation: Option<String>,

    /// The methods declared in the trait
    pub methods: Vec<FunctionUnit>,

    /// The source code of the trait
    pub source: Option<String>,
}

/// Represents an implementation block in the code, not all languages need this
#[derive(Debug, Default)]
pub struct ImplUnit {
    /// Attributes applied to the trait
    pub attributes: Vec<String>,

    /// The documentation for the implementation block
    pub documentation: Option<String>,

    /// The methods implemented in this block
    pub methods: Vec<FunctionUnit>,

    /// The source code of the implementation block
    pub source: Option<String>,
}
```
