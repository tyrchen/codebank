# CODE BANK PRODUCT CONTEXT

## Product Vision
Code Bank aims to simplify code documentation and sharing by:
1. Generating well-formatted markdown documentation from source code
2. Providing different views of the codebase (full code, public interfaces only)
3. Supporting multiple programming languages with a consistent output format

## User Stories
- As a developer, I want to generate documentation from my codebase to share with my team
- As a reviewer, I want to see only the public interfaces to understand the API without implementation details
- As a maintainer, I want to create consistent documentation across multiple languages
- As a teacher, I want to extract code snippets for educational materials

## Usage Scenarios

### Open Source Documentation
Generate clean documentation for open source projects, showing interfaces to potential users without overwhelming them with implementation details.

### Code Reviews
Extract the architecture and interfaces to focus code reviews on design rather than implementation.

### Knowledge Sharing
Create clean, condensed code examples for sharing in wikis, documentation, or educational materials.

### API Documentation
Generate interface-only documentation for libraries and frameworks to showcase the public API.

## Output Format
Code Bank generates markdown files with:
- Section headers for each file
- Code blocks with appropriate language syntax highlighting
- Optional filtering for public interfaces only
- Hierarchical organization matching the source code structure

Example output:
```markdown
# Code Bank
## src/main.rs
```rust
fn main() {
    // Code here
}
```
## src/lib.rs
```rust
pub struct Example {
    // Fields here
}
```
```

## Future Enhancements
- Web interface for generating documentation
- Integration with documentation systems
- Custom templates for different output formats
- Support for more programming languages
