# CODE BANK TECHNICAL CONTEXT

## Technology Stack
- **Programming Language**: Rust (edition 2021)
- **Parser Engine**: tree-sitter with language-specific grammars
- **Error Handling**: thiserror for error definitions, anyhow for error propagation
- **CLI Framework**: clap with derive features
- **File System**: walkdir for directory traversal
- **Build System**: Cargo

## Architecture Overview
Code Bank follows a modular architecture organized around these key components:

1. **Core Traits**
   - `Bank` - Main trait for generating code banks
   - `CodeUnit` - Base trait for code units in the intermediate representation
   - `LanguageParser` - Trait for language-specific parsers

2. **Data Models**
   - Various code unit structs (FileUnit, ModuleUnit, FunctionUnit, etc.)
   - BankStrategy enum for different output strategies

3. **Parsers**
   - Language-specific parser implementations
   - Each parser translates source code into the intermediate representation

4. **Command Line Interface**
   - Entry point (src/bin/codebank.rs)
   - Command-line argument parsing
   - Output formatting

## Source Code Organization
- `src/lib.rs` - Core traits and types
- `src/error.rs` - Error types and result definitions
- `src/parser/mod.rs` - Parser infrastructure and intermediate representation
- `src/parser/lang/` - Language-specific parser implementations
- `src/bin/codebank.rs` - Command-line interface

## Current Implementation Status
The codebase has established the core data structures and traits but requires:
- Complete implementation of language-specific parsers
- Finalization of the CLI interface
- Additional error handling
- Tests and examples
