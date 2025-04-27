mod python;
mod rules;
mod rust;
use rules::FormatterRules;

use super::{FileUnit, FunctionUnit, ImplUnit, ModuleUnit, StructUnit, TraitUnit, Visibility};
use crate::parser::LanguageType;
use crate::{BankStrategy, Result};

pub trait Formatter {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String>;
}

// Implement Formatter for FileUnit
impl Formatter for FileUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add file documentation if present
                if let Some(doc) = &self.document {
                    output.push_str(&format!("{} {}\n", rules.doc_marker, doc));
                }

                // Add declarations
                for decl in &self.declares {
                    output.push_str(&decl.source);
                    output.push('\n');
                }

                // Format each module
                for module in &self.modules {
                    let formatted = module.format(strategy, language)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push('\n');
                    }
                }

                // Format each function
                for function in &self.functions {
                    let formatted = function.format(strategy, language)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push('\n');
                    }
                }

                // Format each struct
                for struct_unit in &self.structs {
                    let formatted = struct_unit.format(strategy, language)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push('\n');
                    }
                }

                // Format each trait
                for trait_unit in &self.traits {
                    let formatted = trait_unit.format(strategy, language)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push('\n');
                    }
                }

                // Format each impl
                for impl_unit in &self.impls {
                    let formatted = impl_unit.format(strategy, language)?;
                    if !formatted.is_empty() {
                        output.push_str(&formatted);
                        output.push('\n');
                    }
                }
            }
            BankStrategy::Summary => {
                // Add file documentation if present
                if let Some(doc) = &self.document {
                    output.push_str(&format!("{} {}\n", rules.doc_marker, doc));
                }

                // Add declarations
                for decl in &self.declares {
                    output.push_str(&decl.source);
                    output.push('\n');
                }

                for module in &self.modules {
                    if module.visibility == Visibility::Public {
                        let module_formatted = module.format(strategy, language)?;
                        output.push_str(&module_formatted);
                        output.push('\n');
                    }
                }

                // Format public functions
                for function in &self.functions {
                    if function.visibility == Visibility::Public {
                        let function_formatted = function.format(strategy, language)?;
                        output.push_str(&function_formatted);
                        output.push('\n');
                    }
                }

                // Format public structs
                for struct_unit in &self.structs {
                    if struct_unit.visibility == Visibility::Public {
                        let struct_formatted = struct_unit.format(strategy, language)?;
                        output.push_str(&struct_formatted);
                        output.push('\n');
                    }
                }

                // Format public traits
                for trait_unit in &self.traits {
                    if trait_unit.visibility == Visibility::Public {
                        let trait_formatted = trait_unit.format(strategy, language)?;
                        output.push_str(&trait_formatted);
                        output.push('\n');
                    }
                }

                // Format impls (only showing public methods)
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy, language)?;
                    output.push_str(&impl_formatted);
                    output.push('\n');
                }
            }
        }

        Ok(output)
    }
}

// Implement Formatter for ModuleUnit
impl Formatter for ModuleUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        // Skip test modules
        if rules.is_test_module(&self.name, &self.attributes) {
            return Ok(String::new());
        }

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests | BankStrategy::Summary => {
                // Add documentation
                if let Some(doc) = &self.document {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Public modules only for Summary
                if *strategy == BankStrategy::NoTests || self.visibility == Visibility::Public {
                    if language == LanguageType::Rust {
                        // TODO: what about other languages?
                        output.push_str(&format!("pub mod {} {{\n", self.name));
                    }

                    // Add public functions
                    for function in &self.functions {
                        if *strategy == BankStrategy::NoTests
                            || function.visibility == Visibility::Public
                        {
                            let function_formatted = function.format(strategy, language)?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                function_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add public structs
                    for struct_unit in &self.structs {
                        if *strategy == BankStrategy::NoTests
                            || struct_unit.visibility == Visibility::Public
                        {
                            let struct_formatted = struct_unit.format(strategy, language)?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                struct_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add public traits
                    for trait_unit in &self.traits {
                        if *strategy == BankStrategy::NoTests
                            || trait_unit.visibility == Visibility::Public
                        {
                            let trait_formatted = trait_unit.format(strategy, language)?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                trait_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add impls (only public methods)
                    for impl_unit in &self.impls {
                        let impl_formatted = impl_unit.format(strategy, language)?;
                        output.push_str(&format!(
                            "    {}\n\n",
                            impl_formatted.replace("\n", "\n    ")
                        ));
                    }

                    // Add public submodules
                    for submodule in &self.submodules {
                        if *strategy == BankStrategy::NoTests
                            || submodule.visibility == Visibility::Public
                        {
                            let submodule_formatted = submodule.format(strategy, language)?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                submodule_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    if language == LanguageType::Rust {
                        output.push_str(rules.function_body_end_marker);
                        output.push('\n');
                    }
                } else {
                    // Private modules not included in summary
                    return Ok(String::new());
                }
            }
        }

        Ok(output)
    }
}

// Implement Formatter for FunctionUnit
impl Formatter for FunctionUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        // Skip test functions in all modes
        if rules.is_test_function(&self.attributes) {
            return Ok(String::new());
        }

        match strategy {
            BankStrategy::Default | BankStrategy::NoTests => {
                // Add documentation if present
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}", rules.doc_marker, line));
                        output.push('\n');
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(attr);
                    output.push('\n');
                }

                // Add function declaration and body
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                // Only include public functions for summary
                if !matches!(self.visibility, Visibility::Public) {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add function signature only (no body)
                let s = self.source.as_deref().unwrap_or("");
                let sig = self.signature.as_deref();
                output.push_str(&rules.format_signature(s, sig));
                output.push('\n');
            }
        }

        Ok(output)
    }
}

// Implement Formatter for StructUnit
impl Formatter for StructUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        match strategy {
            BankStrategy::Default | BankStrategy::NoTests => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                let public_methods: Vec<&FunctionUnit> = self
                    .methods
                    .iter()
                    .filter(|m| m.visibility == Visibility::Public)
                    .collect();

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                output.push_str(&self.head);
                output.push_str(rules.function_body_start_marker);

                if !public_methods.is_empty() {
                    for method in public_methods {
                        let method_formatted = method.format(strategy, language)?;
                        if !method_formatted.is_empty() {
                            output.push_str("\n    ");
                            output.push_str(&method_formatted.replace("\n", "\n    "));
                        }
                    }
                    output.push_str("\n}");
                }

                output.push_str(rules.function_body_end_marker);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

// Implement Formatter for TraitUnit
impl Formatter for TraitUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        match strategy {
            BankStrategy::Default | BankStrategy::NoTests => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                // Only include public traits for Summary
                if self.visibility != Visibility::Public {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Start trait declaration
                match self.visibility {
                    Visibility::Public => output.push_str(&format!("pub trait {} {{\n", self.name)),
                    _ => output.push_str(&format!("trait {} {{\n", self.name)),
                }

                // Add method signatures
                for method in &self.methods {
                    // Only include public methods for Summary
                    if method.visibility != Visibility::Public {
                        continue;
                    }

                    let method_formatted = method.format(strategy, language)?;

                    if !method_formatted.is_empty() {
                        output.push_str("    ");
                        output.push_str(&method_formatted.replace("\n", "\n    "));
                        output.push('\n');
                    }
                }

                let r = FormatterRules::for_language(language);
                output.push_str(r.function_body_end_marker);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

// Implement Formatter for ImplUnit
impl Formatter for ImplUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        match strategy {
            BankStrategy::Default | BankStrategy::NoTests => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                let methods_to_include = self
                    .methods
                    .iter()
                    .filter(|m| matches!(m.visibility, Visibility::Public))
                    .filter(|m| !m.attributes.iter().any(|attr| attr == "#[test]"))
                    .collect::<Vec<_>>();

                // If no methods to include, return an empty string
                if methods_to_include.is_empty() {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                output.push_str(rules.function_body_start_marker);

                for method in methods_to_include {
                    // Standard method formatting
                    let method_formatted = method.format(strategy, language)?;

                    if !method_formatted.is_empty() {
                        output.push_str("\n    ");
                        output.push_str(&method_formatted.replace("\n", "\n    "));
                    }
                }

                output.push('\n');
                output.push_str(rules.function_body_end_marker);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Visibility;

    #[test]
    fn test_function_unit_format() {
        let function = FunctionUnit {
            name: "test_function".to_string(),
            visibility: Visibility::Public,
            documentation: Some("Test function documentation".to_string()),
            signature: Some("fn test_function()".to_string()),
            body: Some("{ println!(\"test\"); }".to_string()),
            source: Some("fn test_function() { println!(\"test\"); }".to_string()),
            attributes: vec!["#[test]".to_string()],
        };

        // Test function should be skipped in Default mode
        let result = function
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(result, "");

        // Test function should be skipped in NoTests mode
        let result = function
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert_eq!(result, "");

        // Test function should be skipped in Summary mode
        let result = function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert_eq!(result, "");

        // Regular function should be included
        let regular_function = FunctionUnit {
            name: "regular_function".to_string(),
            visibility: Visibility::Public,
            documentation: Some("Regular function documentation".to_string()),
            signature: Some("fn regular_function()".to_string()),
            body: Some("{ println!(\"regular\"); }".to_string()),
            source: Some("fn regular_function() { println!(\"regular\"); }".to_string()),
            attributes: vec![],
        };

        let result = regular_function
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Regular function documentation"));
        assert!(result.contains("fn regular_function()"));

        let result = regular_function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();

        assert!(result.contains("fn regular_function()"));
        assert!(result.contains("{ ... }"));
    }

    #[test]
    fn test_module_unit_format() {
        let module = ModuleUnit {
            name: "test_module".to_string(),
            visibility: Visibility::Public,
            document: Some("Test module documentation".to_string()),
            source: Some(
                "/// Test module documentation\n#[cfg(test)]\nmod test_module {".to_string(),
            ),
            attributes: vec!["#[cfg(test)]".to_string()],
            functions: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
            submodules: vec![],
            declares: vec![],
        };

        // Test module should be skipped
        let result = module
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(result, "");

        let regular_module = ModuleUnit {
            name: "regular_module".to_string(),
            visibility: Visibility::Public,
            document: Some("Regular module documentation".to_string()),
            source: Some("/// Regular module documentation\nmod regular_module {}".to_string()),
            attributes: vec![],
            functions: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
            submodules: vec![],
            declares: vec![],
        };

        let result = regular_module
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Regular module documentation"));
        assert!(result.contains("mod regular_module {}"));

        let result = regular_module
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("mod regular_module"));
    }

    #[test]
    fn test_struct_unit_format() {
        let struct_unit = StructUnit {
            name: "TestStruct".to_string(),
            head: "pub struct TestStruct".to_string(),
            visibility: Visibility::Public,
            documentation: Some("Test struct documentation".to_string()),
            source: Some("/// Test struct documentation\npub struct TestStruct {}".to_string()),
            attributes: vec![],
            methods: vec![],
        };

        let result = struct_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Test struct documentation"));
        assert!(result.contains("pub struct TestStruct"));

        let result = struct_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        println!("{}", result);
        assert!(result.contains("pub struct TestStruct"));
    }

    #[test]
    fn test_trait_unit_format() {
        let trait_unit = TraitUnit {
            name: "TestTrait".to_string(),
            visibility: Visibility::Public,
            documentation: Some("Test trait documentation".to_string()),
            source: Some("/// Test trait documentation\npub trait TestTrait {}".to_string()),
            attributes: vec![],
            methods: vec![],
        };

        let result = trait_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Test trait documentation"));
        assert!(result.contains("pub trait TestTrait"));

        let result = trait_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("pub trait TestTrait"));
    }

    #[test]
    fn test_impl_unit_format() {
        let impl_unit = ImplUnit {
            head: "impl".to_string(),
            documentation: Some("Test impl documentation".to_string()),
            source: Some("/// Test impl documentation\nimpl TestStruct {".to_string()),
            attributes: vec![],
            methods: vec![],
        };

        let result = impl_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        println!("{}", result);
        assert!(result.contains("Test impl documentation"));
        assert!(result.contains("impl TestStruct {"));

        let result = impl_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(!result.contains("impl TestStruct"));
    }

    #[test]
    fn test_file_unit_format() {
        let file_unit = FileUnit {
            path: std::path::PathBuf::from("test.rs"),
            document: Some("Test file documentation".to_string()),
            source: Some("/// Test file documentation".to_string()),
            declares: vec![],
            modules: vec![],
            functions: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
        };

        let result = file_unit
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Test file documentation"));

        let result = file_unit
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(result.contains("Test file documentation"));
    }
}
