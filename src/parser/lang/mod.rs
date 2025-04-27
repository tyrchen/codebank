use tree_sitter::Parser;

mod cpp;
mod python;
mod rust;
mod ts;

pub struct RustParser {
    parser: Parser,
}

pub struct PythonParser {
    parser: Parser,
}

pub struct CppParser {
    parser: Parser,
}

pub struct TypeScriptParser {
    parser: Parser,
}
