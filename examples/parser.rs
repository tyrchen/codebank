use std::path::Path;

use anyhow::Result;
use codebank::{CppParser, LanguageParser, PythonParser, RustParser, TypeScriptParser};

fn main() -> Result<()> {
    let mut rust_parser = RustParser::try_new()?;
    let mut python_parser = PythonParser::try_new()?;
    let mut cpp_parser = CppParser::try_new()?;
    let mut ts_parser = TypeScriptParser::try_new()?;

    let data = rust_parser
        .parse_file(Path::new("fixtures/sample.rs"))
        .unwrap();

    println!("Rust:\n{:#?}", data);

    let data = python_parser
        .parse_file(Path::new("fixtures/sample.py"))
        .unwrap();

    println!("Python:\n{:#?}", data);

    let data = ts_parser
        .parse_file(Path::new("fixtures/sample.ts"))
        .unwrap();

    println!("TypeScript:\n{:#?}", data);

    let data = cpp_parser
        .parse_file(Path::new("fixtures/sample.cpp"))
        .unwrap();

    println!("C:\n{:#?}", data);
    Ok(())
}
