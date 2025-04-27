# Code Bank

Code Bank is a tool to concat code from multiple files into a single markdown file, while contains the relative path of the code and the code. It could also generate separate code summary only contains the public interfaces (data structure, functions, macros, etc.) without the code body, using tools like tree-sitter to parse the code. Generated markdown looks like this (example for Rust):

```markdown
# Code Bank
## src/main.rs
```rust
...
```
## src/lib.rs
```rust
...
```

## Features

- [ ] Support Rust language
- [ ] Support Python language
- [ ] Support TypeScript language
- [ ] Support C language
