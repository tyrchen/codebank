use crate::{BankStrategy, FileUnit, FunctionUnit, ModuleUnit, Result, StructUnit};

pub trait PythonFormatter {
    fn format_python(&self, strategy: BankStrategy) -> Result<String>;
}

impl PythonFormatter for FunctionUnit {
    fn format_python(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Skip test functions (marked with pytest decorators)
                if self.attributes.iter().any(|attr| attr.contains("test")) {
                    return Ok(String::new());
                }
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                // Skip private functions
                if self.visibility == crate::Visibility::Private {
                    return Ok(String::new());
                }
                // Add function signature only (no body)
                if let Some(sig) = &self.signature {
                    output.push_str(sig);
                    output.push_str(" ...");
                } else if let Some(source) = &self.source {
                    // Try to extract just the signature from the source
                    if let Some(idx) = source.find(':') {
                        output.push_str(&source[0..=idx]);
                        output.push_str(" ...");
                    } else {
                        // Fallback: use the whole source
                        output.push_str(source);
                    }
                }
            }
        }

        Ok(output)
    }
}

impl PythonFormatter for StructUnit {
    fn format_python(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
                // Format methods
                for method in &self.methods {
                    output.push_str("\n    ");
                    output.push_str(&method.format_python(strategy)?);
                }
            }
            BankStrategy::NoTests => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
                // Include all non-test methods
                for method in &self.methods {
                    if !method.attributes.iter().any(|attr| attr.contains("test")) {
                        output.push_str("\n    ");
                        output.push_str(&method.format_python(strategy)?);
                    }
                }
            }
            BankStrategy::Summary => {
                // Skip private classes
                if self.visibility == crate::Visibility::Private {
                    return Ok(String::new());
                }
                if let Some(source) = &self.source {
                    // Extract class definition
                    if let Some(idx) = source.find(':') {
                        output.push_str(&source[0..=idx]);
                        output.push('\n');
                    }
                }
                // Format public methods only
                for method in &self.methods {
                    if method.visibility == crate::Visibility::Public {
                        output.push_str("    ");
                        output.push_str(&method.format_python(strategy)?);
                        output.push('\n');
                    }
                }
            }
        }

        Ok(output)
    }
}

impl PythonFormatter for ModuleUnit {
    fn format_python(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                    output.push_str("\n\n");
                }
            }
            BankStrategy::NoTests => {
                // Skip test modules (prefixed with test_)
                if self.attributes.iter().any(|attr| attr.contains("test_"))
                    || self.name.starts_with("test_")
                {
                    return Ok(String::new());
                }
                // Include all declarations
                for decl in &self.declares {
                    output.push_str(&decl.source);
                    output.push('\n');
                }
                // Format all non-test functions and classes
                for function in &self.functions {
                    let formatted = function.format_python(strategy)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push_str("\n\n");
                    }
                }
                for class in &self.structs {
                    let formatted = class.format_python(strategy)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push_str("\n\n");
                    }
                }
            }
            BankStrategy::Summary => {
                // Skip private modules
                if self.visibility == crate::Visibility::Private {
                    return Ok(String::new());
                }
                // Format public functions and classes only
                for function in &self.functions {
                    if function.visibility == crate::Visibility::Public {
                        let formatted = function.format_python(strategy)?;
                        if !formatted.is_empty() {
                            output.push_str(&formatted);
                            output.push('\n');
                        }
                    }
                }
                for class in &self.structs {
                    if class.visibility == crate::Visibility::Public {
                        let formatted = class.format_python(strategy)?;
                        if !formatted.is_empty() {
                            output.push_str(&formatted);
                            output.push('\n');
                        }
                    }
                }
            }
        }

        Ok(output)
    }
}

impl PythonFormatter for FileUnit {
    fn format_python(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests | BankStrategy::Summary => {
                // Add declarations first
                for decl in &self.declares {
                    output.push_str(&decl.source);
                    output.push('\n');
                }

                // Add modules
                for module in &self.modules {
                    let formatted = module.format_python(strategy)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push_str("\n\n");
                    }
                }

                // Add functions
                for function in &self.functions {
                    let formatted = function.format_python(strategy)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push_str("\n\n");
                    }
                }

                // Add classes
                for class in &self.structs {
                    let formatted = class.format_python(strategy)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push_str("\n\n");
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::FieldUnit, *};

    // Helper to create a test function
    fn create_test_function(name: &str, is_public: bool, has_test_attr: bool) -> FunctionUnit {
        let mut attrs = Vec::new();
        if has_test_attr {
            attrs.push("@pytest.mark.test".to_string());
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
            signature: Some(format!("def {}():", name)),
            body: Some("    pass".to_string()),
            source: Some(format!("def {}():\n    pass", name)),
        }
    }

    // Helper to create a test class
    fn create_test_class(name: &str, is_public: bool) -> StructUnit {
        let mut methods = Vec::new();
        methods.push(create_test_function(
            &format!("{}_method", name.to_lowercase()),
            true,
            false,
        ));
        // Add a private method as well
        methods.push(create_test_function(
            &format!("_{}_private_method", name.to_lowercase()),
            false,
            false,
        ));

        StructUnit {
            name: name.to_string(),
            head: format!("class {}", name),
            attributes: Vec::new(),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            doc: Some(format!("Documentation for {}", name)),
            methods,
            source: Some(format!("class {}:\n    pass", name)),
            fields: Vec::new(),
        }
    }

    // Helper to create a test module
    fn create_test_module(name: &str, is_public: bool, is_test: bool) -> ModuleUnit {
        let functions = vec![
            create_test_function("module_function", true, false),
            // Add a private function
            create_test_function("_module_private_function", false, false),
        ];

        let structs = vec![create_test_class("ModuleClass", true)];

        let mut attributes = Vec::new();
        if is_test {
            attributes.push("test_".to_string());
        }

        // Add declarations
        let mut declares = Vec::new();
        declares.push(DeclareStatements {
            source: "from typing import List, Dict".to_string(),
            kind: DeclareKind::Import,
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
            source: Some(format!("# Module {}", name)),
        }
    }

    #[test]
    fn test_function_formatter_default() {
        let function = create_test_function("test_function", true, false);
        let formatted = function
            .format(&BankStrategy::Default, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("def test_function():"));
        assert!(formatted.contains("pass"));
    }

    #[test]
    fn test_function_formatter_no_tests() {
        // Regular function
        let function = create_test_function("regular_function", true, false);
        let formatted = function
            .format(&BankStrategy::NoTests, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("def regular_function():"));
        assert!(formatted.contains("pass"));

        // Test function
        let test_function = create_test_function("test_function", true, true);
        let formatted = test_function
            .format(&BankStrategy::NoTests, LanguageType::Python)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_function_formatter_summary() {
        // Public function
        let public_function = create_test_function("public_function", true, false);
        let formatted = public_function
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("def public_function():"));
        assert!(formatted.contains("..."));
        assert!(!formatted.contains("pass"));

        // Private function
        let private_function = create_test_function("_private_function", false, false);
        let formatted = private_function
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_class_formatter_default() {
        let class_unit = create_test_class("TestClass", true);
        let formatted = class_unit
            .format(&BankStrategy::Default, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("class TestClass:"));
        assert!(formatted.contains("pass"));
    }

    #[test]
    fn test_class_formatter_summary() {
        // Public class
        let mut public_class = create_test_class("PublicClass", true);

        // Add a field to the class
        let field = FieldUnit {
            name: "field".to_string(),
            doc: Some("Field documentation".to_string()),
            attributes: vec![],
            source: Some("field = None".to_string()),
        };
        public_class.fields.push(field);

        let formatted = public_class
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();

        assert!(
            formatted.contains("class PublicClass:"),
            "Should include class definition"
        );
        assert!(formatted.contains("field = None"), "Should include fields");
        assert!(
            formatted.contains("def publicclass_method"),
            "Should include public methods"
        );
        assert!(
            !formatted.contains("def _publicclass_private_method"),
            "Should not include private methods"
        );

        // Private class
        let private_class = create_test_class("_PrivateClass", false);
        let formatted = private_class
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_module_formatter_default() {
        let module = create_test_module("test_module", true, false);
        let formatted = module
            .format(&BankStrategy::Default, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("# Module test_module"));
    }

    #[test]
    fn test_module_formatter_no_tests() {
        // Regular module
        let module = create_test_module("regular_module", true, false);
        let formatted = module
            .format(&BankStrategy::NoTests, LanguageType::Python)
            .unwrap();
        // Check for essential elements
        assert!(formatted.contains("def module_function"));
        assert!(formatted.contains("class ModuleClass"));
        assert!(formatted.contains("from typing import List, Dict"));
        assert!(formatted.contains("def _module_private_function")); // Check private function included

        // Test module - should also be processed by NoTests, skipping inner tests if any
        let test_module = create_test_module("test_module", true, true);
        let formatted_test = test_module
            .format(&BankStrategy::NoTests, LanguageType::Python)
            .unwrap();
        assert!(!formatted_test.is_empty()); // Should not be empty
        assert!(formatted_test.contains("def module_function")); // Check content is present
        assert!(formatted_test.contains("class ModuleClass"));
    }

    #[test]
    fn test_module_formatter_summary() {
        // Public module
        let public_module = create_test_module("public_module", true, false);
        let formatted = public_module
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();
        assert!(formatted.contains("def module_function():"));
        assert!(formatted.contains("..."));
        assert!(!formatted.contains("pass"));

        // Private module
        let private_module = create_test_module("_private_module", false, false);
        let formatted = private_module
            .format(&BankStrategy::Summary, LanguageType::Python)
            .unwrap();
        assert!(formatted.is_empty());
    }
}
