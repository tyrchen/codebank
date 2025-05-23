#![cfg(test)]
use super::*; // Imports items from src/parser/formatter/mod.rs
use crate::parser::{RustParser, LanguageParser, FileUnit, DeclareKind, FieldUnit, ModuleUnit, FunctionUnit, StructUnit, TraitUnit, ImplUnit};
use crate::{BankStrategy, LanguageType, Result, Visibility}; // Added Visibility here
use std::path::PathBuf;

// Helper function to parse a fixture file using RustParser
fn parse_rust_fixture(fixture_name: &str) -> Result<FileUnit> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR should be set during tests");
    let path = PathBuf::from(manifest_dir)
        .join("fixtures")
        .join(fixture_name);
    let mut parser = RustParser::try_new()?;
    parser.parse_file(&path)
}

#[test]
fn test_rust_trait_unit_summary_head_formatting_refined() {
    // This test assumes TraitUnit has a `head` field correctly populated by the parser.
    // The TraitUnit struct in `parser/mod.rs` needs `head: String`.
    // For now, we'll construct a TraitUnit manually as if `head` was populated.
    let trait_unit_with_generics = TraitUnit {
        name: "MyGenericTrait".to_string(),
        visibility: Visibility::Public,
        doc: Some("A generic trait.".to_string()),
        // head: "pub trait MyGenericTrait<T> where T: Clone".to_string(), // THIS IS THE KEY FIELD
        source: Some("pub trait MyGenericTrait<T> where T: Clone { fn method(&self); }".to_string()), // Full source
        attributes: vec![],
        methods: vec![FunctionUnit {
            name: "method".to_string(),
            visibility: Visibility::Public,
            doc: None,
            signature: Some("fn method(&self);".to_string()),
            body: None,
            source: Some("fn method(&self);".to_string()),
            attributes: vec![],
        }],
    };

    // The formatter refinement uses `self.source`'s first line for summary head if `TraitUnit.head` is not available.
    // Let's test that behavior.
    let expected_summary = "/// A generic trait.\npub trait MyGenericTrait<T> where T: Clone { ... }";
    let formatted_summary = trait_unit_with_generics.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    assert_eq!(formatted_summary.trim(), expected_summary.trim());
}


#[test]
fn test_rust_struct_unit_summary_field_comma_refined() {
    let struct_with_fields = StructUnit {
        name: "MyStruct".to_string(),
        head: "pub struct MyStruct".to_string(),
        visibility: Visibility::Public,
        doc: None,
        attributes: vec![],
        fields: vec![
            FieldUnit { name: "field_one".to_string(), source: Some("field_one: i32,".to_string()), ..Default::default() },
            FieldUnit { name: "field_two".to_string(), source: Some("field_two: String".to_string()), ..Default::default() },
            FieldUnit { name: "field_three".to_string(), source: Some("field_three: bool,".to_string()), ..Default::default() },
        ],
        methods: vec![],
        source: Some("pub struct MyStruct { field_one: i32, field_two: String, field_three: bool, }".to_string()),
    };

    // Expected: field_one: i32, (original comma kept, no new one added)
    //           field_two: String, (new comma added)
    //           field_three: bool, (original comma kept, no new one added - this is the last field, so no comma by rule)
    // The refined formatter logic for fields in StructUnit::Summary:
    // output.push_str(field_src_trimmed);
    // if i < self.fields.len() - 1 && rules.field_sep == "," { output.push_str(","); }
    // This means it *always* adds a comma if not the last field. The `field_src_trimmed` already removed its own comma.
    // This should result in single commas.

    let expected_summary = r#"pub struct MyStruct {
    field_one: i32,
    field_two: String,
    field_three: bool
}"#; // Note: Last field does not get a comma from the loop.
    let formatted_summary = struct_with_fields.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    assert_eq!(formatted_summary.trim(), expected_summary.trim());
}

// --- End-to-End Tests ---

#[test]
fn test_e2e_sample_rs_default_strategy() {
    let file_unit = parse_rust_fixture("sample.rs").unwrap();
    let formatted_default = file_unit.format(&BankStrategy::Default, LanguageType::Rust).unwrap();
    // Default strategy should return the raw source content.
    assert_eq!(formatted_default.trim(), file_unit.source.as_ref().unwrap().trim());
}

#[test]
fn test_e2e_sample_rs_no_tests_strategy() {
    let file_unit = parse_rust_fixture("sample.rs").unwrap();
    let formatted_no_tests = file_unit.format(&BankStrategy::NoTests, LanguageType::Rust).unwrap();

    assert!(formatted_no_tests.contains("/// This is a file-level documentation comment."));
    assert!(formatted_no_tests.contains("extern crate proc_macro;"));
    assert!(formatted_no_tests.contains("pub mod public_module {"));
    assert!(formatted_no_tests.contains("pub struct PublicStruct<T: FmtDebug + Clone, U>")); // Full struct def
    assert!(formatted_no_tests.contains("fn method(&self, input: T) -> String {")); // Full method body in impl
    assert!(formatted_no_tests.contains("pub fn public_function() -> String {")); // Full function
    assert!(formatted_no_tests.contains("fn private_function(s: &str) -> String {")); // Private functions included
    assert!(!formatted_no_tests.contains("mod tests {")); // Test module should be excluded
    assert!(!formatted_no_tests.contains("test_public_function_output()"));
}

#[test]
fn test_e2e_sample_rs_summary_strategy() {
    let file_unit = parse_rust_fixture("sample.rs").unwrap();
    let formatted_summary = file_unit.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    
    // File level
    assert!(formatted_summary.contains("/// This is a file-level documentation comment."));
    assert!(formatted_summary.contains("extern crate proc_macro;"));
    assert!(formatted_summary.contains("use crate::public_module::PublicStruct;"));
    assert!(formatted_summary.contains("mod my_other_module;"));

    // Public Module
    assert!(formatted_summary.contains("/// This is a public module."));
    assert!(formatted_summary.contains("#[cfg(feature = \"some_feature\")]"));
    assert!(formatted_summary.contains("pub mod public_module {"));
    // Inside public_module
    assert!(formatted_summary.contains("    /// This is a public struct with documentation."));
    assert!(formatted_summary.contains("    pub struct PublicStruct<T: FmtDebug + Clone, U> { ... }"));
    assert!(formatted_summary.contains("    pub trait PublicTrait<T> { ... }"));
    assert!(formatted_summary.contains("    pub enum PublicEnum { ... }"));
    assert!(!formatted_summary.contains("    crate_visible_function()")); // Not public
    assert!(!formatted_summary.contains("    mod nested_module {")); // nested_module is private

    // Top-level public items
    assert!(formatted_summary.contains("/// A public function with multiple attributes and docs."));
    assert!(formatted_summary.contains("#[inline]"));
    assert!(formatted_summary.contains("pub fn public_function() -> String { ... }"));
    assert!(!formatted_summary.contains("private_function")); // Private

    assert!(formatted_summary.contains("pub type PublicTypeAlias<T> = Result<T, Box<dyn std::error::Error>>;"));
    assert!(formatted_summary.contains("pub const PUBLIC_CONSTANT: &str = \"constant value\";"));
    assert!(formatted_summary.contains("pub static PUBLIC_STATIC_VAR: i32 = 100;"));

    assert!(formatted_summary.contains("pub struct GenericStruct<T> { ... }"));
    assert!(formatted_summary.contains("pub trait GenericTrait<T> { ... }"));

    // Impl blocks
    // Inherent impl for GenericStruct - only shows if it had public methods. `new` is private.
    // The formatter logic for ImplUnit summary: "If no methods to include and strategy is Summary (and not trait impl), return empty"
    // So, the "impl<T> GenericStruct<T>" block might be empty or not present if `new` is its only method and private.
    // The test `test_impl_blocks_details` checks this more closely.
    // Let's ensure the doc and attribute for it are not present if the block itself isn't.
    let generic_struct_impl_head = "impl<T> GenericStruct<T> {";
    if formatted_summary.contains(generic_struct_impl_head) { // If the impl block is rendered (e.g. if it had public methods)
        assert!(formatted_summary.contains("/// Implementation for GenericStruct."));
        assert!(formatted_summary.contains("#[allow(dead_code)]")); // Attribute on impl
    } else { // If the impl block is NOT rendered because it has no public methods
        assert!(!formatted_summary.contains("/// Implementation for GenericStruct."));
    }


    assert!(formatted_summary.contains("/// Implementation of GenericTrait for GenericStruct."));
    assert!(formatted_summary.contains("impl<T> GenericTrait<T> for GenericStruct<T> where T: Clone + FmtDebug {"));
    assert!(formatted_summary.contains("    fn method(&self, value: T) -> T { ... }"));

    // Test module should not be present
    assert!(!formatted_summary.contains("mod tests {"));
}


#[test]
fn test_e2e_sample_advanced_summary_strategy() {
    let file_unit = parse_rust_fixture("sample_advanced.rs").unwrap();
    let formatted_summary = file_unit.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();

    assert!(formatted_summary.contains("/// File for advanced Rust constructs."));
    assert!(formatted_summary.contains("pub mod level1 {"));
    // level2 is private, so its contents (even if pub(in path)) won't be shown via traversing level1.
    assert!(!formatted_summary.contains("mod level2 {"));
    assert!(!formatted_summary.contains("DeepStruct")); 
    assert!(formatted_summary.contains("pub fn complex_generic_function<'a, T, U>(param_t: T, param_u: &'a U) -> Result<T, U::Error> where T: std::fmt::Debug + Clone + Send + 'static, U: std::error::Error + ?Sized, for<'b> &'b U: Send { ... }"));
    
    assert!(formatted_summary.contains("pub struct AdvancedGenericStruct<'a, A, B> where A: AsRef<[u8]> + ?Sized, B: 'a + Send + Sync { ... }"));
    assert!(formatted_summary.contains("pub enum GenericResult<S, E> where S: Send, E: std::fmt::Debug { ... }"));
    assert!(formatted_summary.contains("pub trait AdvancedTrait { ... }"));
    assert!(formatted_summary.contains("impl AdvancedTrait for MyTypeForAdvancedTrait {"));
    assert!(formatted_summary.contains("fn process(&self, item: Self::Item) -> Result<Self::Item, String> { ... }"));
    
    assert!(formatted_summary.contains("pub struct MyUnitStruct;"));
    assert!(formatted_summary.contains("pub struct EmptyStruct { ... }")); // Empty struct with {}
    assert!(!formatted_summary.contains("NoFieldsStruct")); // Private
}

// --- Specific Unit Tests for Coverage ---

#[test]
fn test_visibility_formatting_in_summary() {
    let public_fn = FunctionUnit { name: "public_fn".into(), visibility: Visibility::Public, signature: Some("pub fn public_fn()".into()), ..Default::default() };
    let private_fn = FunctionUnit { name: "private_fn".into(), visibility: Visibility::Private, signature: Some("fn private_fn()".into()), ..Default::default() };
    let crate_fn = FunctionUnit { name: "crate_fn".into(), visibility: Visibility::Crate, signature: Some("pub(crate) fn crate_fn()".into()), ..Default::default() };
    
    let file_unit = FileUnit {
        functions: vec![public_fn, private_fn, crate_fn],
        ..Default::default()
    };

    let summary = file_unit.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    assert!(summary.contains("pub fn public_fn() { ... }"));
    assert!(!summary.contains("private_fn"));
    assert!(!summary.contains("crate_fn")); // Not public
}

#[test]
fn test_attribute_and_doc_formatting() {
    let func = FunctionUnit {
        name: "func_with_attrs_docs".into(),
        visibility: Visibility::Public,
        doc: Some("This is a doc line 1.\nThis is doc line 2.".into()),
        attributes: vec!["#[inline]".into(), "#[must_use]".into()],
        signature: Some("pub fn func_with_attrs_docs()".into()),
        body: Some("{ }".into()),
        ..Default::default()
    };
    let expected_no_tests = "/// This is a doc line 1.\n/// This is doc line 2.\n#[inline]\n#[must_use]\npub fn func_with_attrs_docs() { }";
    let expected_summary = "/// This is a doc line 1.\n/// This is doc line 2.\n#[inline]\n#[must_use]\npub fn func_with_attrs_docs() { ... }";

    assert_eq!(func.format(&BankStrategy::NoTests, LanguageType::Rust).unwrap().trim(), expected_no_tests.trim());
    assert_eq!(func.format(&BankStrategy::Summary, LanguageType::Rust).unwrap().trim(), expected_summary.trim());
}

#[test]
fn test_empty_and_comment_only_files_formatting() {
    let empty_file_unit = parse_rust_fixture("empty.rs").unwrap();
    for strategy in [BankStrategy::Default, BankStrategy::NoTests, BankStrategy::Summary] {
        let formatted = empty_file_unit.format(&strategy, LanguageType::Rust).unwrap();
        assert_eq!(formatted.trim(), "", "Formatted output for empty file should be empty for strategy {:?}", strategy);
    }

    let comments_only_unit = parse_rust_fixture("only_comments.rs").unwrap();
    let default_fmt = comments_only_unit.format(&BankStrategy::Default, LanguageType::Rust).unwrap();
    assert_eq!(default_fmt.trim(), comments_only_unit.source.as_ref().unwrap().trim());

    let expected_doc_summary = "/// This is an inner line comment, often used for module-level docs.\n/// This is an inner block comment.\n/// Also for module-level docs usually.";
    let no_tests_fmt = comments_only_unit.format(&BankStrategy::NoTests, LanguageType::Rust).unwrap();
    assert_eq!(no_tests_fmt.trim(), expected_doc_summary.trim());
    
    let summary_fmt = comments_only_unit.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    assert_eq!(summary_fmt.trim(), expected_doc_summary.trim());
}

#[test]
fn test_trait_unit_head_field_assumption() {
    // This test acknowledges that TraitUnit does not have `head: String` yet.
    // The formatter for TraitUnit (Summary) currently falls back to using the first line of `source`.
    let trait_unit = TraitUnit {
        name: "SimpleTrait".to_string(),
        visibility: Visibility::Public,
        doc: None,
        source: Some("pub trait SimpleTrait<T>: Debug where T: Copy {\n    // ...\n}".to_string()),
        attributes: vec![],
        methods: vec![],
        // head: "pub trait SimpleTrait<T>: Debug where T: Copy".to_string(), // Ideal
    };
    let expected_summary = "pub trait SimpleTrait<T>: Debug where T: Copy { ... }";
    let formatted_summary = trait_unit.format(&BankStrategy::Summary, LanguageType::Rust).unwrap();
    assert_eq!(formatted_summary.trim(), expected_summary.trim());
}
