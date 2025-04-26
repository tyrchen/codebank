# Code Bank

Code Bank is a tool to comcat code from multiple files into a single markdown file, while contains the relative path of the code and the code. It could also generate separate code summary only contains the public interfaces (data structure, functions, macros, etc.) without the code body, using tools like tree-sitter to parse the code.

Initially, it should support Rust language. Later on it could be extended to other languages. So make sure we define proper trait for the code bank.

## Code Bank Trait

```rust
pub struct SummaryStrategy {
    pub include_private: bool,
    pub include_macro: bool,
    pub include_doc: bool,
    pub include_comment: bool,
    pub include_code: bool,
    pub include_path: bool,
}

pub trait Bank {
    fn generate(&self, root_dir: &Path) -> Result<String>;
    fn generate_summary(&self, root_dir: &Path, strategy: SummaryStrategy) -> Result<String>;
}
```
