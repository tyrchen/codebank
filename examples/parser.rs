use std::path::Path;

use anyhow::Result;
use codebank::{CParser, LanguageParser, PythonParser, RustParser, TypeScriptParser};

const C: &str = include_str!("../fixtures/sample.c");
const TS: &str = include_str!("../fixtures/sample.ts");

fn main() -> Result<()> {
    let mut rust_parser = RustParser::try_new()?;
    let mut python_parser = PythonParser::try_new()?;
    let mut c_parser = CParser::try_new()?;
    let mut ts_parser = TypeScriptParser::try_new()?;

    let data = rust_parser
        .parse_file(Path::new("fixtures/sample.rs"))
        .unwrap();

    println!("Rust:\n{:#?}", data);

    let data = python_parser
        .parse_file(Path::new("fixtures/sample.py"))
        .unwrap();

    println!("Python:\n{:#?}", data);

    let tree = c_parser.parse(C, None).unwrap();

    println!("C:\n{:?}", tree);

    let tree = ts_parser.parse(TS, None).unwrap();

    println!("TypeScript:\n{:?}", tree);

    Ok(())
}
