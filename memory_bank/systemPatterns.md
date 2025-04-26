# CODE BANK SYSTEM PATTERNS

## Design Principles
- **Modularity**: Each component has a single responsibility
- **Extensibility**: Easy to add support for new languages
- **Flexibility**: Multiple output strategies (full code, summary)
- **Error Handling**: Comprehensive error types with meaningful messages

## Code Patterns

### Trait-Based Architecture
The codebase follows a trait-based architecture pattern:
- Core functionality defined through traits
- Concrete implementations provided for specific use cases
- Clear separation between interface and implementation

```rust
pub trait Bank {
    fn generate(&self, root_dir: &Path, strategy: BankStrategy) -> Result<String>;
}

pub trait LanguageParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit>;
}
```

### Intermediate Representation
Code is parsed into an intermediate representation:
- Language-agnostic data structures
- Hierarchical structure mirroring code organization
- Preserves metadata (visibility, documentation, etc.)

```rust
pub struct FileUnit {
    pub path: PathBuf,
    pub modules: Vec<ModuleUnit>,
    pub functions: Vec<FunctionUnit>,
    // ...
}
```

### Strategy Pattern
Output generation follows the strategy pattern:
- BankStrategy enum defines different output strategies
- Implementation can vary based on the selected strategy

```rust
pub enum BankStrategy {
    Default,
    NoTests,
    Summary,
}
```

### Error Type Hierarchy
Error handling follows Rust best practices:
- Custom Error enum with variant for each error type
- thiserror for deriving Error implementations
- Result type alias for consistent return types

```rust
pub enum Error {
    Io(#[from] io::Error),
    Parse(String),
    // ...
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Naming Conventions
- **Traits**: Descriptive noun (Bank, CodeUnit, LanguageParser)
- **Implementations**: LanguageName + Trait (RustParser, PythonParser)
- **Enums**: Descriptive noun + purpose (BankStrategy, LanguageType)
- **Data Structures**: Descriptive noun + Unit (FileUnit, ModuleUnit)
