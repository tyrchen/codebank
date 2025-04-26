# CODE BANK PROJECT BRIEF

## Project Overview
Code Bank is a tool to concatenate code from multiple files into a single markdown file, preserving the relative path of each code snippet. The tool can also generate a summary containing only the public interfaces (data structures, functions, macros, etc.) without the code body, using tree-sitter to parse code.

## Core Features
- Concatenate code from multiple files into a single markdown file
- Generate code summaries with public interfaces only
- Support multiple programming languages (Rust, Python, TypeScript, C)
- Parse code using tree-sitter for accurate interface extraction

## Technical Infrastructure
- Rust-based command-line tool
- Uses tree-sitter for parsing different languages
- Modular architecture with language-specific parsers
- Support for different output strategies (full code, summary only)

## Implementation Status
- Basic data structures and traits defined
- Parser infrastructure established
- Language-specific parsers created but not fully implemented
- Command-line interface skeleton established

## Dependencies
- tree-sitter and language-specific grammar libraries
- anyhow and thiserror for error handling
- walkdir for filesystem traversal
- clap for command-line argument parsing

## Next Steps
- Complete language-specific parsers
- Implement the command-line interface
- Add configuration options
- Write tests and examples
