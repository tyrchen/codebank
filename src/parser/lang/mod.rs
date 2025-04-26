use tree_sitter::Parser;

mod python;
mod rust;

pub struct RustParser {
    parser: Parser,
}

pub struct PythonParser {
    parser: Parser,
}
