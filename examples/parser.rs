use anyhow::Result;
use codebank::{CParser, PythonParser, RustParser, TypeScriptParser};

const RUST: &str = include_str!("../fixtures/sample.rs");
const PYTHON: &str = include_str!("../fixtures/sample.py");
const C: &str = include_str!("../fixtures/sample.c");
const TS: &str = include_str!("../fixtures/sample.ts");

fn main() -> Result<()> {
    let mut rust_parser = RustParser::try_new()?;
    let mut python_parser = PythonParser::try_new()?;
    let mut c_parser = CParser::try_new()?;
    let mut ts_parser = TypeScriptParser::try_new()?;

    let tree = rust_parser.parse(RUST, None).unwrap();

    println!("Rust:\n{:?}", tree);

    let tree = python_parser.parse(PYTHON, None).unwrap();

    println!("Python:\n{:?}", tree);

    let tree = c_parser.parse(C, None).unwrap();

    println!("C:\n{:?}", tree);

    let tree = ts_parser.parse(TS, None).unwrap();

    println!("TypeScript:\n{:?}", tree);

    Ok(())
}
