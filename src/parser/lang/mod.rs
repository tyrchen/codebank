use tree_sitter::Parser;

mod c;
mod python;
mod rust;
mod ts;

pub struct RustParser {
    parser: Parser,
}

pub struct PythonParser {
    parser: Parser,
}

pub struct CParser {
    parser: Parser,
}

pub struct TypeScriptParser {
    parser: Parser,
}
