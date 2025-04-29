# CodeBank

![](https://github.com/tyrchen/rust-lib-template/workflows/build/badge.svg)

CodeBank is a powerful code analysis and documentation tool that parses source code from multiple programming languages and generates structured documentation in markdown format. It provides deep insights into your codebase's structure, making it easier to understand and maintain large projects.

## Features

- **Multi-Language Support**:
  - Rust (fully supported with comprehensive parsing)
  - Python (fully supported with function, class, and module parsing)
  - TypeScript/JavaScript (fully supported with function, class, interface, and export parsing)
  - C (TODO)
  - Go (fully supported with package, function, struct, interface, and method parsing)

- **Code Structure Analysis**:
  - Parses functions, modules, structs/classes, traits/interfaces
  - Extracts documentation and comments
  - Analyzes visibility and scope
  - Handles declarations and imports

- **Flexible Output Strategies**:
  - Default: Complete code representation
  - NoTests: Code representation excluding test code
  - Summary: Public interface documentation only

- **Tree-sitter Integration**:
  - Robust parsing using tree-sitter
  - Accurate syntax analysis
  - Language-specific parsing rules

## Installation

You can install CodeBank using Cargo:

```bash
cargo install codebank
```

There are two executable binaries: `cb` and `cb-mcp`.

- `cb` is the command line interface for CodeBank.
- `cb-mcp` is the MCP server for CodeBank.

## Usage

### Command Line Interface

```bash
# Generate complete code documentation
cb /path/to/source --output docs.md

# Generate documentation excluding tests
cb /path/to/source --strategy no-tests --output docs.md

# Generate public interface summary
cb /path/to/source --strategy summary --output docs.md
```

### MCP Usage

Please refer to [README_MCP.md](README_MCP.md) for details.

### Library Usage

Add CodeBank to your project's dependencies:

```toml
[dependencies]
codebank = { version = "0.4.1", default-features = false }  # Replace with actual version
```

Then use it in your Rust code:

```rust
use codebank::{Bank, BankStrategy, CodeBank};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new code bank generator
    let code_bank = CodeBank::try_new()?;

    // Generate documentation
    let content = code_bank.generate(
        Path::new("/path/to/source"),
        BankStrategy::Default
    )?;

    // Use the generated content
    println!("{}", content);
    Ok(())
}
```

## Development Status

### Current Implementation

- ✅ Comprehensive Rust parsing
- ✅ Comprehensive Python parsing with support for functions, classes, and modules
- ✅ Comprehensive TypeScript/JavaScript parsing with support for functions, classes, interfaces, and exports
- ✅ Basic C file parsing (includes/defines)
- ✅ Markdown output generation
- ✅ Multiple output strategies

### Planned Improvements

1. **Parser Enhancements**:
   - Enhanced C parser with full language support
   - Support for more languages

2. **Feature Additions**:
   - Parallel file processing
   - Caching support
   - Custom output formats
   - Incremental parsing

3. **Performance Optimizations**:
   - Streaming parsing for large files
   - Memory usage optimization
   - Parser result caching

4. **Documentation and Testing**:
   - Expanded test coverage
   - More usage examples
   - Comprehensive API documentation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is distributed under the terms of MIT.

See [LICENSE](LICENSE.md) for details.

Copyright 2025 Tyr Chen
