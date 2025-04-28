#[cfg(test)]
mod tests {
    use crate::*;

    // Helper to create a test function
    fn create_test_function(name: &str, is_public: bool, has_test_attr: bool) -> FunctionUnit {
        let mut attrs = Vec::new();
        if has_test_attr {
            attrs.push("#[test]".to_string());
        }

        FunctionUnit {
            name: name.to_string(),
            attributes: attrs,
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            doc: Some(format!("Documentation for {}", name)),
            signature: Some(format!("fn {}()", name)),
            body: Some("{ /* function body */ }".to_string()),
            source: Some(format!("fn {}() {{ /* function body */ }}", name)),
        }
    }

    // Helper to create a test struct
    fn create_test_struct(name: &str, is_public: bool) -> StructUnit {
        let mut methods = Vec::new();
        methods.push(create_test_function(
            &format!("{}_method", name.to_lowercase()),
            true,
            false,
        ));
        // Add a private method as well
        methods.push(create_test_function(
            &format!("{}_private_method", name.to_lowercase()),
            false,
            false,
        ));

        let visibility = if is_public {
            Visibility::Public
        } else {
            Visibility::Private
        };
        StructUnit {
            name: name.to_string(),
            head: format!("{} struct {}", visibility.as_str(LanguageType::Rust), name),
            attributes: Vec::new(),
            visibility,
            doc: Some(format!("Documentation for {}", name)),
            fields: Vec::new(),
            methods,
            source: Some(format!("struct {} {{ field: i32 }}", name)),
        }
    }

    // Helper to create a test module
    fn create_test_module(name: &str, is_public: bool, is_test: bool) -> ModuleUnit {
        let functions = vec![
            create_test_function("module_function", true, false),
            // Add a private function
            create_test_function("module_private_function", false, false),
        ];

        let structs = vec![create_test_struct("ModuleStruct", true)];

        let mut attributes = Vec::new();
        if is_test {
            attributes.push("#[cfg(test)]".to_string());
        }

        // Add declarations
        let mut declares = Vec::new();
        declares.push(DeclareStatements {
            source: "use std::io;".to_string(),
            kind: DeclareKind::Use,
        });

        ModuleUnit {
            name: name.to_string(),
            attributes,
            doc: Some(format!("Documentation for module {}", name)),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            functions,
            structs,
            traits: Vec::new(),
            impls: Vec::new(),
            submodules: Vec::new(),
            declares,
            source: Some(format!("mod {} {{ /* module contents */ }}", name)),
        }
    }

    // Helper to create a test impl block, with option for trait implementation
    fn create_test_impl(is_trait_impl: bool) -> ImplUnit {
        let methods = vec![
            // Add both public and private methods
            create_test_function("public_method", true, false),
            create_test_function("private_method", false, false),
        ];

        let (head, source) = if is_trait_impl {
            (
                "impl SomeTrait for SomeStruct".to_string(),
                "impl SomeTrait for SomeStruct { /* impl body */ }".to_string(),
            )
        } else {
            (
                "impl SomeStruct".to_string(),
                "impl SomeStruct { /* impl body */ }".to_string(),
            )
        };

        ImplUnit {
            attributes: Vec::new(),
            doc: Some("Documentation for implementation".to_string()),
            head,
            methods,
            source: Some(source),
        }
    }

    // Helper to create a test impl block with only private methods
    fn create_private_methods_impl() -> ImplUnit {
        ImplUnit {
            attributes: Vec::new(),
            doc: Some("Documentation for implementation with private methods".to_string()),
            head: "impl StructWithPrivateMethods".to_string(),
            methods: vec![
                create_test_function("private_method1", false, false),
                create_test_function("private_method2", false, false),
            ],
            source: Some("impl StructWithPrivateMethods { /* impl body */ }".to_string()),
        }
    }

    // Helper to create a test enum
    fn create_test_enum(name: &str, is_public: bool) -> StructUnit {
        let visibility = if is_public {
            Visibility::Public
        } else {
            Visibility::Private
        };
        let head = format!("{} enum {}", visibility.as_str(LanguageType::Rust), name);
        let source = format!(
            "/// Docs for {}\n{} {{
    VariantA,
    VariantB(String),
}}",
            name, head
        );
        StructUnit {
            name: name.to_string(),
            head,
            visibility,
            doc: Some(format!("Docs for {}", name)),
            attributes: vec![],
            fields: vec![], // Variants aren't parsed as fields currently
            methods: vec![],
            source: Some(source),
        }
    }

    #[test]
    fn test_function_formatter_default() {
        let function = create_test_function("test_function", true, false);
        let formatted = function
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("fn test_function()"));
        assert!(formatted.contains("/* function body */"));
    }

    #[test]
    fn test_function_formatter_no_tests() {
        // Regular function
        let function = create_test_function("regular_function", true, false);
        let formatted = function
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("fn regular_function()"));
        assert!(formatted.contains("/* function body */"));

        // Test function
        let test_function = create_test_function("test_function", true, true);
        let formatted = test_function
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_function_formatter_summary() {
        // Public function
        let public_function = create_test_function("public_function", true, false);
        let formatted = public_function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("fn public_function()"));
        assert!(!formatted.contains("/* function body */"));
        assert!(formatted.contains("{ ... }"));

        // Private function
        let private_function = create_test_function("private_function", false, false);
        let formatted = private_function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_struct_formatter_default() {
        let struct_unit = create_test_struct("TestStruct", true);
        let formatted = struct_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("struct TestStruct"));
        assert!(formatted.contains("field: i32"));
    }

    #[test]
    fn test_struct_formatter_summary() {
        // Public struct
        let mut public_struct = create_test_struct("PublicStruct", true);

        // Add a field to the struct
        let field = FieldUnit {
            name: "field".to_string(),
            doc: Some("Field documentation".to_string()),
            attributes: vec![],
            source: Some("pub field: i32".to_string()),
        };
        public_struct.fields.push(field);

        let formatted = public_struct
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        assert!(formatted.contains("struct PublicStruct"));
        assert!(
            formatted.contains("pub field: i32"),
            "Summary should include fields"
        );
        assert!(
            formatted.contains("fn publicstruct_method"),
            "Summary should include public methods"
        );
        assert!(
            !formatted.contains("fn publicstruct_private_method"),
            "Summary should not include private methods"
        );

        // Private struct should be skipped
        let private_struct = create_test_struct("PrivateStruct", false);
        let formatted = private_struct
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(
            formatted.is_empty(),
            "Private structs should be skipped in summary mode"
        );
    }

    #[test]
    fn test_module_formatter_default() {
        let module = create_test_module("test_module", true, false);
        let formatted = module
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("mod test_module"));
        assert!(formatted.contains("/* module contents */"));
    }

    #[test]
    fn test_module_formatter_no_tests() {
        // Regular module
        let module = create_test_module("regular_module", true, false);
        let formatted = module
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("pub mod regular_module"));
        assert!(formatted.contains("fn module_function"));
        assert!(formatted.contains("fn module_private_function"));
        assert!(formatted.contains("struct ModuleStruct"));
        assert!(formatted.contains("use std::io;"));

        // Test module
        let test_module = create_test_module("test_module", true, true);
        let formatted = test_module
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("#[cfg(test)]"));
        assert!(formatted.contains("pub mod test_module"));
    }

    #[test]
    fn test_module_formatter_summary() {
        // Public module
        let public_module = create_test_module("public_module", true, false);
        let formatted = public_module
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("pub mod public_module"));
        assert!(formatted.contains("fn module_function()"));
        // Functions should only show signatures in summary
        assert!(!formatted.contains("/* function body */"));

        // Private module
        let private_module = create_test_module("private_module", false, false);
        let formatted = private_module
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_struct_formatter_no_tests() {
        // Test struct with private methods
        let struct_unit = create_test_struct("TestStruct", true);
        let formatted = struct_unit
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();

        // Should now just return the source for NoTests mode
        assert!(formatted.contains("struct TestStruct { field: i32 }"));
        // Should not contain methods as we're just using the source
        assert!(!formatted.contains("fn teststruct_method()"));
        assert!(!formatted.contains("fn teststruct_private_method()"));
    }

    #[test]
    fn test_regular_impl_formatter_summary() {
        // Regular (non-trait) implementation
        let impl_unit = create_test_impl(false);
        let formatted = impl_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        // Only public methods should be included in regular impls
        // Check the head extracted by the parser
        assert!(formatted.contains("impl SomeStruct"));
        assert!(formatted.contains("fn public_method"));
        assert!(!formatted.contains("fn private_method"));
    }

    #[test]
    fn test_trait_impl_formatter_summary() {
        // Trait implementation
        let impl_unit = create_test_impl(true);
        let formatted = impl_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        // Both public and private methods should be included in trait impls
        // Check the head extracted by the parser
        assert!(formatted.contains("impl SomeTrait for SomeStruct"));
        assert!(formatted.contains("fn public_method"));
        assert!(
            !formatted.contains("fn private_method"),
            "Private method should be excluded in trait impl summary"
        );
        // Check that bodies are summarized
        assert!(
            formatted.contains("public_method() { ... }"),
            "Public method body not summarized"
        );
        assert!(
            !formatted.contains("/* function body */"),
            "Full function body should not be present"
        );
    }

    #[test]
    fn test_impl_formatter_no_tests() {
        // Both regular and trait implementation should include all non-test methods in NoTests mode
        let regular_impl = create_test_impl(false);
        let formatted = regular_impl
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("fn public_method"));
        assert!(formatted.contains("fn private_method"));

        let trait_impl = create_test_impl(true);
        let formatted = trait_impl
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("fn public_method"));
        assert!(formatted.contains("fn private_method"));
    }

    #[test]
    fn test_impl_with_only_private_methods_summary() {
        // Regular impl with only private methods should return empty string in Summary mode
        let impl_unit = create_private_methods_impl();
        let formatted = impl_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        // Should be empty since there are no public methods
        assert!(formatted.is_empty());

        // But in NoTests mode, it should include the private methods
        let formatted = impl_unit
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(!formatted.is_empty());
        assert!(formatted.contains("fn private_method1"));
        assert!(formatted.contains("fn private_method2"));
    }

    #[test]
    fn test_file_unit_formatter() {
        let mut file_unit = FileUnit {
            path: std::path::PathBuf::from("test_file.rs"),
            ..Default::default()
        };

        // Add modules
        file_unit
            .modules
            .push(create_test_module("public_module", true, false));
        file_unit
            .modules
            .push(create_test_module("test_module", true, true));

        // Add functions
        file_unit
            .functions
            .push(create_test_function("public_function", true, false));
        file_unit
            .functions
            .push(create_test_function("private_function", false, false));
        file_unit
            .functions
            .push(create_test_function("test_function", true, true));

        // Add structs
        file_unit
            .structs
            .push(create_test_struct("PublicStruct", true));
        file_unit
            .structs
            .push(create_test_struct("PrivateStruct", false));

        // Test Default strategy
        file_unit.source = Some("// This is the entire file content".to_string());
        let formatted = file_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(formatted, "// This is the entire file content");

        // Test NoTests strategy - test modules and functions should be excluded
        let formatted = file_unit
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("pub mod public_module"));
        assert!(!formatted.contains("fn test_function"));
        assert!(formatted.contains("fn public_function"));
        assert!(formatted.contains("fn private_function"));
        assert!(formatted.contains("struct PublicStruct"));
        assert!(formatted.contains("struct PrivateStruct"));

        // Test Summary strategy - only public items should be included
        let formatted = file_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(formatted.contains("pub mod public_module"));
        assert!(!formatted.contains("mod private_module"));
        assert!(formatted.contains("fn public_function()"));
        assert!(!formatted.contains("fn private_function"));
        assert!(formatted.contains("struct PublicStruct"));
        assert!(!formatted.contains("struct PrivateStruct"));
    }

    #[test]
    fn test_file_unit_no_tests_includes_all() {
        let mut file_unit = FileUnit {
            path: std::path::PathBuf::from("test_file.rs"),
            ..Default::default()
        };

        // Add modules
        file_unit
            .modules
            .push(create_test_module("public_module", true, false));
        file_unit
            .modules
            .push(create_test_module("private_module", false, false));
        file_unit
            .modules
            .push(create_test_module("test_module", true, true));

        // Add functions
        file_unit
            .functions
            .push(create_test_function("public_function", true, false));
        file_unit
            .functions
            .push(create_test_function("private_function", false, false));
        file_unit
            .functions
            .push(create_test_function("test_function", true, true));

        // Add structs
        file_unit
            .structs
            .push(create_test_struct("PublicStruct", true));
        file_unit
            .structs
            .push(create_test_struct("PrivateStruct", false));

        // Add declarations
        file_unit.declares.push(DeclareStatements {
            source: "use std::collections::HashMap;".to_string(),
            kind: DeclareKind::Use,
        });

        // Test NoTests strategy
        let formatted = file_unit
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();

        // Should include all non-test items regardless of visibility
        assert!(formatted.contains("pub mod public_module"));
        assert!(formatted.contains("mod private_module"));
        assert!(!formatted.contains("fn test_function"));
        assert!(formatted.contains("fn public_function"));
        assert!(formatted.contains("fn private_function"));
        assert!(formatted.contains("struct PublicStruct"));
        assert!(formatted.contains("struct PrivateStruct"));
        assert!(formatted.contains("use std::collections::HashMap;"));

        // We now just display struct source in NoTests, not individual methods anymore
        assert!(!formatted.contains("fn publicstruct_private_method()"));
    }

    #[test]
    fn test_enum_formatter_summary() {
        let public_enum = create_test_enum("PublicEnum", true);
        let formatted = public_enum
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        // Summary for enums now follows the same pattern as structs
        assert!(formatted.contains("/// Docs for PublicEnum"));
        assert!(formatted.contains("pub enum PublicEnum"));
        // No fields/variants in the enum
        assert!(!formatted.contains("VariantA,"));
        assert!(!formatted.contains("VariantB(String),"));

        let private_enum = create_test_enum("PrivateEnum", false);
        let formatted = private_enum
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        // Private enums should be omitted entirely in summary
        assert!(formatted.is_empty());
    }
}
