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
                if let Some(doc) = &self.doc {
                    output.push_str(&format!("{} {}\n", rules.doc_marker, doc));
                }

                // Add declarations
                for decl in &self.declares {
                    output.push_str(&decl.source);
                    output.push('\n');
                }

                // Format each module (skip test modules)
                for module in &self.modules {
                    if !rules.is_test_module(&module.name, &module.attributes) {
                        let formatted = module.format(strategy, language)?;
                        if !formatted.is_empty() {
                            output.push_str(&formatted);
                            output.push('\n');
                        }
                    }
                }

                // Format each function (skip test functions)
                for function in &self.functions {
                    if !rules.is_test_function(&function.attributes) {
                        let formatted = function.format(strategy, language)?;
                        if !formatted.is_empty() {
                            output.push_str(&formatted);
                            output.push('\n');
                        }
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
                if let Some(doc) = &self.doc {
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

        // Skip test modules entirely for Summary strategy
        if *strategy == BankStrategy::Summary && rules.is_test_module(&self.name, &self.attributes)
        {
            return Ok(String::new());
        }

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add documentation
                if let Some(doc) = &self.doc {
                    for line in doc.lines() {
                        output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                    }
                }

                // Add attributes (including test attributes for NoTests)
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Write module head
                output.push_str(&format!(
                    "{} mod {} {{\n",
                    self.visibility.as_str(language),
                    self.name
                ));

                // Add declarations
                for decl in &self.declares {
                    output.push_str(&format!("    {}\n", decl.source));
                }

                // Format all functions (skip test functions)
                for function in &self.functions {
                    if !rules.is_test_function(&function.attributes) {
                        let function_formatted = function.format(strategy, language)?;
                        if !function_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                function_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }
                }

                // Format all structs
                for struct_unit in &self.structs {
                    let struct_formatted = struct_unit.format(strategy, language)?;
                    if !struct_formatted.is_empty() {
                        output.push_str(&format!(
                            "    {}\n\n",
                            struct_formatted.replace("\n", "\n    ")
                        ));
                    }
                }

                // Format all traits
                for trait_unit in &self.traits {
                    let trait_formatted = trait_unit.format(strategy, language)?;
                    if !trait_formatted.is_empty() {
                        output.push_str(&format!(
                            "    {}\n\n",
                            trait_formatted.replace("\n", "\n    ")
                        ));
                    }
                }

                // Format all impls
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy, language)?;
                    if !impl_formatted.is_empty() {
                        output.push_str(&format!(
                            "    {}\n\n",
                            impl_formatted.replace("\n", "\n    ")
                        ));
                    }
                }

                // Format submodules
                for submodule in &self.submodules {
                    let sub_formatted = submodule.format(strategy, language)?;
                    if !sub_formatted.is_empty() {
                        output.push_str(&format!(
                            "    {}\n\n",
                            sub_formatted.replace("\n", "\n    ")
                        ));
                    }
                }

                output.push_str("}\n");
            }
            BankStrategy::Summary => {
                // Public modules only
                if self.visibility == Visibility::Public {
                    // Add documentation
                    if let Some(doc) = &self.doc {
                        for line in doc.lines() {
                            output.push_str(&format!("{} {}\n", rules.doc_marker, line));
                        }
                    }
                    // Add attributes (except test attributes)
                    for attr in &self.attributes {
                        if !rules.test_module_markers.contains(&attr.as_str()) {
                            output.push_str(&format!("{}\n", attr));
                        }
                    }

                    output.push_str(&format!("pub mod {} {{\n", self.name));

                    // Add declarations
                    for decl in &self.declares {
                        output.push_str(&format!("    {}\n", decl.source));
                    }

                    // Format public functions
                    for function in &self.functions {
                        if function.visibility == Visibility::Public
                            && !rules.is_test_function(&function.attributes)
                        {
                            let function_formatted = function.format(strategy, language)?;
                            if !function_formatted.is_empty() {
                                output.push_str(&format!(
                                    "    {}\n\n",
                                    function_formatted.replace("\n", "\n    ")
                                ));
                            }
                        }
                    }

                    // Format public structs
                    for struct_unit in &self.structs {
                        if struct_unit.visibility == Visibility::Public {
                            let struct_formatted = struct_unit.format(strategy, language)?;
                            if !struct_formatted.is_empty() {
                                output.push_str(&format!(
                                    "    {}\n\n",
                                    struct_formatted.replace("\n", "\n    ")
                                ));
                            }
                        }
                    }

                    // Format public traits
                    for trait_unit in &self.traits {
                        if trait_unit.visibility == Visibility::Public {
                            let trait_formatted = trait_unit.format(strategy, language)?;
                            if !trait_formatted.is_empty() {
                                output.push_str(&format!(
                                    "    {}\n\n",
                                    trait_formatted.replace("\n", "\n    ")
                                ));
                            }
                        }
                    }

                    // Format impls (showing public methods)
                    for impl_unit in &self.impls {
                        let impl_formatted = impl_unit.format(strategy, language)?;
                        if !impl_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                impl_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Format public submodules
                    for submodule in &self.submodules {
                        if submodule.visibility == Visibility::Public {
                            let sub_formatted = submodule.format(strategy, language)?;
                            if !sub_formatted.is_empty() {
                                output.push_str(&format!(
                                    "    {}\n\n",
                                    sub_formatted.replace("\n", "\n    ")
                                ));
                            }
                        }
                    }

                    output.push_str("}\n");
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

        // Handle Default strategy separately: just return source
        if *strategy == BankStrategy::Default {
            return Ok(self.source.clone().unwrap_or_default());
        }

        // Skip test functions for NoTests and Summary
        if rules.is_test_function(&self.attributes) {
            return Ok(String::new());
        }

        // Skip private functions for Summary
        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public {
            return Ok(String::new());
        }

        // Add documentation (for NoTests and Summary of non-test, non-private functions)
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                output.push_str(&format!("{} {}\n", rules.doc_marker, line));
            }
        }

        // Add attributes (except test attributes)
        for attr in &self.attributes {
            if !rules.test_markers.contains(&attr.as_str()) {
                output.push_str(&format!("{}\n", attr));
            }
        }

        match strategy {
            BankStrategy::Default => { /* Already handled above */ }
            BankStrategy::NoTests => {
                // For NoTests, append the signature and body (if available)
                // This assumes docs/attrs were added above.
                if let Some(sig) = &self.signature {
                    output.push_str(sig);
                }
                if let Some(body) = &self.body {
                    // Ensure space before body if signature exists and doesn't end with space
                    if self.signature.is_some()
                        && !output.ends_with(' ')
                        && !body.starts_with('{')
                        && !body.starts_with(':')
                    {
                        output.push(' ');
                    }
                    output.push_str(body);
                } else if self.signature.is_none() {
                    // Fallback to source if no signature/body
                    if let Some(src) = &self.source {
                        output.push_str(src);
                    }
                }
            }
            BankStrategy::Summary => {
                // For Summary, append only the formatted signature
                // Assumes docs/attrs were added above.
                if let Some(signature) = &self.signature {
                    let formatted_sig = rules.format_signature(signature, Some(signature));
                    output.push_str(&formatted_sig);
                } else if let Some(source) = &self.source {
                    // Fallback if no explicit signature? Format source as signature.
                    let formatted_sig = rules.format_signature(source, None);
                    output.push_str(&formatted_sig);
                }
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

        // Skip private structs for Summary
        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public {
            return Ok(String::new());
        }

        // Add documentation
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                output.push_str(&format!("{} {}\n", rules.doc_marker, line));
            }
        }

        // Add attributes
        for attr in &self.attributes {
            output.push_str(&format!("{}\n", attr));
        }

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests | BankStrategy::Summary => {
                // Add head (struct definition line)
                output.push_str(&self.head);

                // Handle body/methods based on language and strategy
                if *strategy == BankStrategy::Summary {
                    // Check if it looks like a Rust enum based on source or head
                    let is_rust_enum = language == LanguageType::Rust
                        && (self.head.contains(" enum ")
                            || self.source.as_ref().is_some_and(|s| s.contains(" enum ")));

                    if is_rust_enum {
                        if let Some(source) = &self.source {
                            // Construct the full enum output including docs/attrs and source
                            let mut enum_output = String::new();
                            if let Some(doc) = &self.doc {
                                for line in doc.lines() {
                                    enum_output
                                        .push_str(&format!("{} {}\n", rules.doc_marker, line));
                                }
                            }
                            for attr in &self.attributes {
                                enum_output.push_str(&format!("{}\n", attr));
                            }
                            enum_output.push_str(source);
                            return Ok(enum_output); // Return the full enum source with context
                        } else {
                            // Fallback if no source: just head + ellipsis
                            output.push_str(rules.summary_ellipsis);
                        }
                    } else {
                        // Default summary behavior for non-enum structs
                        output.push_str(rules.summary_ellipsis);
                    }
                } else {
                    // BankStrategy::NoTests
                    // NoTests Mode: Include methods
                    let body_start = if language == LanguageType::Python {
                        ":\n"
                    } else {
                        " {\n"
                    };
                    let body_end = if language == LanguageType::Python {
                        ""
                    } else {
                        "}"
                    };
                    output.push_str(body_start);

                    for method in &self.methods {
                        if !rules.is_test_function(&method.attributes) {
                            let method_formatted = method.format(strategy, language)?;
                            if !method_formatted.is_empty() {
                                output.push_str("    ");
                                output.push_str(&method_formatted.replace("\n", "\n    "));
                                output.push('\n');
                            }
                        }
                    }
                    output.push_str(body_end);
                }
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

        // Skip private traits for Summary
        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public {
            return Ok(String::new());
        }

        // Add documentation
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                output.push_str(&format!("{} {}\n", rules.doc_marker, line));
            }
        }

        // Add attributes
        for attr in &self.attributes {
            output.push_str(&format!("{}\n", attr));
        }

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests | BankStrategy::Summary => {
                let head = format!("{} trait {}", self.visibility.as_str(language), self.name);
                output.push_str(&head);

                // Include body only for NoTests
                if *strategy == BankStrategy::NoTests {
                    output.push_str(" {\n");
                    for method in &self.methods {
                        if !rules.is_test_function(&method.attributes) {
                            let method_formatted = method.format(strategy, language)?;
                            if !method_formatted.is_empty() {
                                output.push_str("    ");
                                output.push_str(&method_formatted.replace("\n", "\n    "));
                                output.push('\n');
                            }
                        }
                    }
                    output.push_str(rules.function_body_end_marker);
                } else {
                    // Summary mode
                    output.push_str(rules.summary_ellipsis);
                }
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
        let is_trait_impl = self.head.contains(" for ");

        // Filter methods based on strategy
        let methods_to_include: Vec<&FunctionUnit> = match strategy {
            BankStrategy::Default => self.methods.iter().collect(),
            BankStrategy::NoTests => self
                .methods
                .iter()
                .filter(|m| !rules.is_test_function(&m.attributes))
                .collect(),
            BankStrategy::Summary => {
                if is_trait_impl {
                    // Include all non-test methods for trait impls in Summary
                    self.methods
                        .iter()
                        .filter(|m| !rules.is_test_function(&m.attributes))
                        .collect()
                } else {
                    // Include only public, non-test methods for regular impls in Summary
                    self.methods
                        .iter()
                        .filter(|m| {
                            m.visibility == Visibility::Public
                                && !rules.is_test_function(&m.attributes)
                        })
                        .collect()
                }
            }
        };

        // If no methods to include and strategy is Summary (and not trait impl), return empty
        // Trait impls should show head even if empty
        if methods_to_include.is_empty() && *strategy == BankStrategy::Summary && !is_trait_impl {
            return Ok(String::new());
        }

        // Add documentation
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                output.push_str(&format!("{} {}\n", rules.doc_marker, line));
            }
        }

        // Add attributes
        for attr in &self.attributes {
            output.push_str(&format!("{}\n", attr));
        }

        match strategy {
            BankStrategy::Default => {
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests | BankStrategy::Summary => {
                output.push_str(&self.head);
                output.push_str(" {\n");

                for method in methods_to_include {
                    // Format method using the current strategy (Summary will summarize bodies)
                    let method_formatted = method.format(strategy, language)?;

                    if !method_formatted.is_empty() {
                        output.push_str("    ");
                        output.push_str(&method_formatted.replace("\n", "\n    "));
                        output.push('\n');
                    }
                }
                output.push_str(rules.function_body_end_marker);
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
            doc: Some("Test function documentation".to_string()),
            signature: Some("fn test_function()".to_string()),
            body: Some("{ println!(\"test\"); }".to_string()),
            source: Some("fn test_function() { println!(\"test\"); }".to_string()),
            attributes: vec!["#[test]".to_string()],
        };
        let expected_source = function.source.clone().unwrap();

        // Default: should return full source for test functions
        let result_default = function
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_default, expected_source);

        // NoTests: Test function should be skipped
        let result_no_tests = function
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_no_tests, "");

        // Summary: Test function should be skipped
        let result_summary = function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_summary, "");

        // Regular function should be included
        let regular_function = FunctionUnit {
            name: "regular_function".to_string(),
            visibility: Visibility::Public,
            doc: Some("Regular function documentation".to_string()),
            signature: Some("pub fn regular_function() -> bool".to_string()),
            body: Some("{ true }".to_string()),
            source: Some("pub fn regular_function() -> bool { true }".to_string()),
            attributes: vec![],
        };
        let regular_source = regular_function.source.clone().unwrap();
        let regular_sig = regular_function.signature.clone().unwrap();
        let rules = FormatterRules::for_language(LanguageType::Rust);

        // Default: should return full source
        let result_default_regular = regular_function
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_default_regular, regular_source);

        // NoTests: should return docs + attrs + signature + body
        let result_no_tests_regular = regular_function
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(result_no_tests_regular.contains("Regular function documentation"));
        assert!(result_no_tests_regular.contains("pub fn regular_function() -> bool"));
        assert!(result_no_tests_regular.contains("{ true }"));

        // Summary: should return docs + attrs + formatted signature
        let result_summary_regular = regular_function
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert!(result_summary_regular.contains("Regular function documentation"));
        assert!(result_summary_regular
            .contains(&rules.format_signature(&regular_sig, Some(&regular_sig))));
        assert!(!result_summary_regular.contains("{ true }")); // Should not contain body
    }

    #[test]
    fn test_module_unit_format() {
        let test_module = ModuleUnit {
            name: "test_module".to_string(),
            visibility: Visibility::Public,
            doc: Some("Test module documentation".to_string()),
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
        let expected_test_source = test_module.source.clone().unwrap();

        // Default: should return full source for test modules
        let result_default_test = test_module
            .format(&BankStrategy::Default, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_default_test, expected_test_source);

        // NoTests: Test module should be processed (but inner tests skipped)
        let result_no_tests_test = test_module
            .format(&BankStrategy::NoTests, LanguageType::Rust)
            .unwrap();
        assert!(result_no_tests_test.contains("mod test_module")); // Check if module definition is present
        assert!(result_no_tests_test.contains("#[cfg(test)]"));

        // Summary: Test module should be skipped
        let result_summary_test = test_module
            .format(&BankStrategy::Summary, LanguageType::Rust)
            .unwrap();
        assert_eq!(result_summary_test, "");

        let regular_module = ModuleUnit {
            name: "regular_module".to_string(),
            visibility: Visibility::Public,
            doc: Some("Regular module documentation".to_string()),
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
            doc: Some("Test struct documentation".to_string()),
            attributes: vec![],
            methods: vec![],
            fields: Vec::new(),
            source: Some("/// Test struct documentation\npub struct TestStruct {}".to_string()),
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
            doc: Some("Test trait documentation".to_string()),
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
            doc: Some("Test impl documentation".to_string()),
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
            doc: Some("Test file documentation".to_string()),
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
