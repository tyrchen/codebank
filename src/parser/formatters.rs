use super::{FileUnit, Formatter, FunctionUnit, ImplUnit, ModuleUnit, StructUnit, TraitUnit};
use crate::{BankStrategy, Result, Visibility};

// Implement Formatter for FileUnit
impl Formatter for FileUnit {
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Skip test modules
                let non_test_modules: Vec<&ModuleUnit> = self
                    .modules
                    .iter()
                    .filter(|m| {
                        !m.attributes
                            .iter()
                            .any(|attr| attr.contains("#[cfg(test)]"))
                            && m.name != "tests"
                    })
                    .collect();

                // Format and include non-test modules
                for module in non_test_modules {
                    let module_formatted = module.format(strategy.clone())?;
                    output.push_str(&module_formatted);
                    output.push_str("\n\n");
                }

                // Skip test functions
                let non_test_functions: Vec<&FunctionUnit> = self
                    .functions
                    .iter()
                    .filter(|f| !f.attributes.iter().any(|attr| attr == "#[test]"))
                    .collect();

                // Format and include non-test functions
                for function in non_test_functions {
                    let function_formatted = function.format(strategy.clone())?;
                    output.push_str(&function_formatted);
                    output.push_str("\n\n");
                }

                // Format and include structs
                for struct_unit in &self.structs {
                    let struct_formatted = struct_unit.format(strategy.clone())?;
                    output.push_str(&struct_formatted);
                    output.push_str("\n\n");
                }

                // Format and include traits
                for trait_unit in &self.traits {
                    let trait_formatted = trait_unit.format(strategy.clone())?;
                    output.push_str(&trait_formatted);
                    output.push_str("\n\n");
                }

                // Format and include impls
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy.clone())?;
                    output.push_str(&impl_formatted);
                    output.push_str("\n\n");
                }
            }
            BankStrategy::Summary => {
                // Only include public interfaces
                // Format public modules
                for module in &self.modules {
                    if matches!(module.visibility, Visibility::Public) {
                        let module_formatted = module.format(strategy.clone())?;
                        output.push_str(&module_formatted);
                        output.push_str("\n\n");
                    }
                }

                // Format public functions
                for function in &self.functions {
                    if matches!(function.visibility, Visibility::Public) {
                        let function_formatted = function.format(strategy.clone())?;
                        output.push_str(&function_formatted);
                        output.push_str("\n\n");
                    }
                }

                // Format public structs
                for struct_unit in &self.structs {
                    if matches!(struct_unit.visibility, Visibility::Public) {
                        let struct_formatted = struct_unit.format(strategy.clone())?;
                        output.push_str(&struct_formatted);
                        output.push_str("\n\n");
                    }
                }

                // Format public traits
                for trait_unit in &self.traits {
                    if matches!(trait_unit.visibility, Visibility::Public) {
                        let trait_formatted = trait_unit.format(strategy.clone())?;
                        output.push_str(&trait_formatted);
                        output.push_str("\n\n");
                    }
                }

                // Format impls (only showing public methods)
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy.clone())?;
                    output.push_str(&impl_formatted);
                    output.push_str("\n\n");
                }
            }
        }

        Ok(output)
    }
}

// Implement Formatter for ModuleUnit
impl Formatter for ModuleUnit {
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add documentation
                if let Some(doc) = &self.document {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add visibility and module name
                match self.visibility {
                    Visibility::Public => output.push_str(&format!("pub mod {} {{\n", self.name)),
                    _ => output.push_str(&format!("mod {} {{\n", self.name)),
                }

                // Add content - skip test functions
                let non_test_functions: Vec<&FunctionUnit> = self
                    .functions
                    .iter()
                    .filter(|f| !f.attributes.iter().any(|attr| attr == "#[test]"))
                    .collect();

                // Format and include non-test functions
                for function in non_test_functions {
                    let function_formatted = function.format(strategy.clone())?;
                    output.push_str(&format!(
                        "    {}\n\n",
                        function_formatted.replace("\n", "\n    ")
                    ));
                }

                // Format and include structs
                for struct_unit in &self.structs {
                    let struct_formatted = struct_unit.format(strategy.clone())?;
                    output.push_str(&format!(
                        "    {}\n\n",
                        struct_formatted.replace("\n", "\n    ")
                    ));
                }

                // Format and include traits
                for trait_unit in &self.traits {
                    let trait_formatted = trait_unit.format(strategy.clone())?;
                    output.push_str(&format!(
                        "    {}\n\n",
                        trait_formatted.replace("\n", "\n    ")
                    ));
                }

                // Format and include impls
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy.clone())?;
                    output.push_str(&format!(
                        "    {}\n\n",
                        impl_formatted.replace("\n", "\n    ")
                    ));
                }

                // Format and include submodules (except test modules)
                let non_test_submodules: Vec<&ModuleUnit> = self
                    .submodules
                    .iter()
                    .filter(|m| {
                        !m.attributes
                            .iter()
                            .any(|attr| attr.contains("#[cfg(test)]"))
                            && m.name != "tests"
                    })
                    .collect();

                for submodule in non_test_submodules {
                    let submodule_formatted = submodule.format(strategy.clone())?;
                    output.push_str(&format!(
                        "    {}\n\n",
                        submodule_formatted.replace("\n", "\n    ")
                    ));
                }

                output.push_str("}\n");
            }
            BankStrategy::Summary => {
                // Add documentation
                if let Some(doc) = &self.document {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Public modules only for Summary
                if matches!(self.visibility, Visibility::Public) {
                    output.push_str(&format!("pub mod {} {{\n", self.name));

                    // Add public functions
                    for function in &self.functions {
                        if matches!(function.visibility, Visibility::Public) {
                            let function_formatted = function.format(strategy.clone())?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                function_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add public structs
                    for struct_unit in &self.structs {
                        if matches!(struct_unit.visibility, Visibility::Public) {
                            let struct_formatted = struct_unit.format(strategy.clone())?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                struct_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add public traits
                    for trait_unit in &self.traits {
                        if matches!(trait_unit.visibility, Visibility::Public) {
                            let trait_formatted = trait_unit.format(strategy.clone())?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                trait_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add impls
                    for impl_unit in &self.impls {
                        let impl_formatted = impl_unit.format(strategy.clone())?;
                        if !impl_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                impl_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }

                    // Add public submodules
                    for submodule in &self.submodules {
                        if matches!(submodule.visibility, Visibility::Public) {
                            let submodule_formatted = submodule.format(strategy.clone())?;
                            output.push_str(&format!(
                                "    {}\n\n",
                                submodule_formatted.replace("\n", "\n    ")
                            ));
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
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Skip test functions
                if self.attributes.iter().any(|attr| attr == "#[test]") {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add function source
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::Summary => {
                // Only include public functions for Summary
                if !matches!(self.visibility, Visibility::Public) {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add signature
                if let Some(signature) = &self.signature {
                    output.push_str(&format!("{} {{ ... }}", signature));
                } else {
                    // Build signature from components
                    output.push_str("pub fn ");
                    output.push_str(&self.name);
                    output.push('(');

                    // Format parameters
                    let params: Vec<String> = self
                        .parameters
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.parameter_type))
                        .collect();
                    output.push_str(&params.join(", "));
                    output.push(')');

                    // Add return type if present
                    if let Some(ret_type) = &self.return_type {
                        output.push_str(&format!(" -> {}", ret_type));
                    }

                    // Add placeholder for function body
                    output.push_str(" { ... }");
                }
            }
        }

        Ok(output)
    }
}

// Implement Formatter for StructUnit
impl Formatter for StructUnit {
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add struct declaration
                match self.visibility {
                    Visibility::Public => {
                        output.push_str(&format!("pub struct {} {{\n", self.name))
                    }
                    _ => output.push_str(&format!("struct {} {{\n", self.name)),
                }

                // Add fields
                for field in &self.fields {
                    // Add field documentation
                    if let Some(doc) = &field.documentation {
                        for line in doc.lines() {
                            output.push_str(&format!("    /// {}\n", line));
                        }
                    }

                    // Add field attributes
                    for attr in &field.attributes {
                        output.push_str(&format!("    {}\n", attr));
                    }

                    // Add field declaration
                    match field.visibility {
                        Visibility::Public => output
                            .push_str(&format!("    pub {}: {},\n", field.name, field.field_type)),
                        _ => {
                            output.push_str(&format!("    {}: {},\n", field.name, field.field_type))
                        }
                    }
                }

                output.push_str("}\n");
            }
            BankStrategy::Summary => {
                // Only include public structs for Summary
                if !matches!(self.visibility, Visibility::Public) {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add struct declaration
                output.push_str(&format!("pub struct {} {{\n", self.name));

                // Add public fields only
                for field in &self.fields {
                    if matches!(field.visibility, Visibility::Public) {
                        // Add field documentation
                        if let Some(doc) = &field.documentation {
                            for line in doc.lines() {
                                output.push_str(&format!("    /// {}\n", line));
                            }
                        }

                        // Add field attributes
                        for attr in &field.attributes {
                            output.push_str(&format!("    {}\n", attr));
                        }

                        // Add field declaration
                        output
                            .push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
                    }
                }

                output.push_str("}\n");
            }
        }

        Ok(output)
    }
}

// Implement Formatter for TraitUnit
impl Formatter for TraitUnit {
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add trait declaration
                match self.visibility {
                    Visibility::Public => output.push_str(&format!("pub trait {} {{\n", self.name)),
                    _ => output.push_str(&format!("trait {} {{\n", self.name)),
                }

                // Add methods (skip test methods)
                for method in &self.methods {
                    if !method.attributes.iter().any(|attr| attr == "#[test]") {
                        let method_formatted = method.format(strategy.clone())?;
                        if !method_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                method_formatted.replace("\n", "\n    ")
                            ));
                        }
                    }
                }

                output.push_str("}\n");
            }
            BankStrategy::Summary => {
                // Only include public traits for Summary
                if !matches!(self.visibility, Visibility::Public) {
                    return Ok(String::new());
                }

                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add trait declaration
                output.push_str(&format!("pub trait {} {{\n", self.name));

                // Add methods (all trait methods are considered public)
                for method in &self.methods {
                    let method_formatted = method.format(strategy.clone())?;
                    if !method_formatted.is_empty() {
                        output.push_str(&format!(
                            "    {}\n\n",
                            method_formatted.replace("\n", "\n    ")
                        ));
                    }
                }

                output.push_str("}\n");
            }
        }

        Ok(output)
    }
}

// Implement Formatter for ImplUnit
impl Formatter for ImplUnit {
    fn format(&self, strategy: BankStrategy) -> Result<String> {
        let mut output = String::new();

        match strategy {
            BankStrategy::Default => {
                // For default strategy, include the full source code
                if let Some(source) = &self.source {
                    output.push_str(source);
                }
            }
            BankStrategy::NoTests => {
                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add impl declaration
                if let Some(trait_name) = &self.trait_name {
                    output.push_str(&format!(
                        "impl {} for {} {{\n",
                        trait_name, self.target_type
                    ));
                } else {
                    output.push_str(&format!("impl {} {{\n", self.target_type));
                }

                // Add methods (skip test methods)
                let mut has_methods = false;
                for method in &self.methods {
                    if !method.attributes.iter().any(|attr| attr == "#[test]") {
                        let method_formatted = method.format(strategy.clone())?;
                        if !method_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                method_formatted.replace("\n", "\n    ")
                            ));
                            has_methods = true;
                        }
                    }
                }

                if has_methods {
                    output.push_str("}\n");
                } else {
                    // If there are no methods after filtering, return empty string
                    return Ok(String::new());
                }
            }
            BankStrategy::Summary => {
                // Add documentation
                if let Some(doc) = &self.documentation {
                    for line in doc.lines() {
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add impl declaration
                if let Some(trait_name) = &self.trait_name {
                    output.push_str(&format!(
                        "impl {} for {} {{\n",
                        trait_name, self.target_type
                    ));
                } else {
                    output.push_str(&format!("impl {} {{\n", self.target_type));
                }

                // Add public methods only
                let mut has_methods = false;
                for method in &self.methods {
                    if matches!(method.visibility, Visibility::Public) {
                        let method_formatted = method.format(strategy.clone())?;
                        if !method_formatted.is_empty() {
                            output.push_str(&format!(
                                "    {}\n\n",
                                method_formatted.replace("\n", "\n    ")
                            ));
                            has_methods = true;
                        }
                    }
                }

                if has_methods {
                    output.push_str("}\n");
                } else {
                    // If there are no public methods, return empty string
                    return Ok(String::new());
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BankStrategy, FieldUnit, FunctionUnit, ParameterUnit, Visibility};
    use std::path::PathBuf;

    // Helper to create a test function unit
    fn create_test_function(name: &str, is_public: bool, has_test_attr: bool) -> FunctionUnit {
        let mut attributes = Vec::new();
        if has_test_attr {
            attributes.push("#[test]".to_string());
        }

        FunctionUnit {
            name: name.to_string(),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            documentation: Some("Test documentation".to_string()),
            parameters: vec![ParameterUnit {
                name: "param1".to_string(),
                parameter_type: "String".to_string(),
                is_self: false,
            }],
            return_type: Some("String".to_string()),
            source: Some(format!(
                "fn {}(param1: String) -> String {{ \"result\".to_string() }}",
                name
            )),
            signature: Some(format!("fn {}(param1: String) -> String", name)),
            body: Some("{ \"result\".to_string() }".to_string()),
            attributes,
        }
    }

    // Helper to create a test struct unit
    fn create_test_struct(name: &str, is_public: bool) -> StructUnit {
        StructUnit {
            name: name.to_string(),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            documentation: Some("Test struct documentation".to_string()),
            fields: vec![
                FieldUnit {
                    name: "field1".to_string(),
                    visibility: Visibility::Public,
                    field_type: "String".to_string(),
                    documentation: Some("Field documentation".to_string()),
                    attributes: vec!["#[derive(Debug)]".to_string()],
                },
                FieldUnit {
                    name: "field2".to_string(),
                    visibility: Visibility::Private,
                    field_type: "i32".to_string(),
                    documentation: None,
                    attributes: Vec::new(),
                },
            ],
            methods: Vec::new(),
            source: Some(format!(
                "struct {} {{\n    pub field1: String,\n    field2: i32,\n}}",
                name
            )),
            attributes: vec!["#[derive(Debug)]".to_string()],
        }
    }

    // Helper to create a test module unit
    fn create_test_module(name: &str, is_public: bool, is_test: bool) -> ModuleUnit {
        let mut attributes = Vec::new();
        if is_test {
            attributes.push("#[cfg(test)]".to_string());
        }

        let document = Some("Test module documentation".to_string());

        ModuleUnit {
            name: name.to_string(),
            document,
            declares: Vec::new(),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            functions: vec![
                create_test_function("module_fn1", true, false),
                create_test_function("module_fn2", false, false),
                create_test_function("test_fn", true, true),
            ],
            structs: vec![create_test_struct("ModuleStruct", true)],
            traits: Vec::new(),
            impls: Vec::new(),
            submodules: Vec::new(),
            source: Some(format!("mod {} {{\n    // Contents\n}}", name)),
            attributes,
        }
    }

    #[test]
    fn test_function_formatter_default() {
        let function = create_test_function("test_fn", true, false);
        let formatted = function.format(BankStrategy::Default).unwrap();

        // Default strategy should include the full source
        assert!(
            formatted.contains("fn test_fn(param1: String) -> String { \"result\".to_string() }")
        );
    }

    #[test]
    fn test_function_formatter_no_tests() {
        // Test regular function
        let function = create_test_function("regular_fn", true, false);
        let formatted = function.format(BankStrategy::NoTests).unwrap();

        // NoTests strategy should include non-test functions
        assert!(formatted.contains("Test documentation"));
        assert!(formatted.contains("fn regular_fn"));

        // Test test function
        let test_function = create_test_function("test_function", true, true);
        let formatted_test = test_function.format(BankStrategy::NoTests).unwrap();

        // NoTests strategy should skip test functions
        assert!(formatted_test.is_empty());
    }

    #[test]
    fn test_function_formatter_summary() {
        // Test public function
        let function = create_test_function("public_fn", true, false);
        let formatted = function.format(BankStrategy::Summary).unwrap();

        // Summary strategy should include public functions with only signature
        assert!(formatted.contains("Test documentation"));
        assert!(formatted.contains("fn public_fn(param1: String) -> String { ... }"));

        // Test private function
        let private_function = create_test_function("private_fn", false, false);
        let formatted_private = private_function.format(BankStrategy::Summary).unwrap();

        // Summary strategy should skip private functions
        assert!(formatted_private.is_empty());
    }

    #[test]
    fn test_struct_formatter_default() {
        let struct_unit = create_test_struct("TestStruct", true);
        let formatted = struct_unit.format(BankStrategy::Default).unwrap();

        // Default strategy should include the full source
        assert!(formatted.contains("struct TestStruct"));
        assert!(formatted.contains("pub field1: String"));
        assert!(formatted.contains("field2: i32"));
    }

    #[test]
    fn test_struct_formatter_summary() {
        // Test public struct
        let struct_unit = create_test_struct("PublicStruct", true);
        let formatted = struct_unit.format(BankStrategy::Summary).unwrap();

        // Summary strategy should include public structs with only public fields
        assert!(formatted.contains("Test struct documentation"));
        assert!(formatted.contains("#[derive(Debug)]"));
        assert!(formatted.contains("pub struct PublicStruct"));
        assert!(formatted.contains("pub field1: String"));
        assert!(!formatted.contains("field2: i32")); // Private field should be skipped

        // Test private struct
        let private_struct = create_test_struct("PrivateStruct", false);
        let formatted_private = private_struct.format(BankStrategy::Summary).unwrap();

        // Summary strategy should skip private structs
        assert!(formatted_private.is_empty());
    }

    #[test]
    fn test_module_formatter_default() {
        let module = create_test_module("test_module", true, false);
        let formatted = module.format(BankStrategy::Default).unwrap();

        // Default strategy should include the full source
        assert!(formatted.contains("mod test_module"));
    }

    #[test]
    fn test_module_formatter_no_tests() {
        // Test regular module
        let module = create_test_module("regular_module", true, false);
        let formatted = module.format(BankStrategy::NoTests).unwrap();

        // NoTests strategy should include non-test modules
        assert!(formatted.contains("Test module documentation"));
        assert!(formatted.contains("pub mod regular_module"));
        assert!(formatted.contains("module_fn1"));
        assert!(formatted.contains("module_fn2"));
        assert!(!formatted.contains("test_fn")); // Test function should be skipped

        // Test test module
        let test_module = create_test_module("tests", true, true);
        let formatted_test = test_module.format(BankStrategy::NoTests).unwrap();

        // NoTests strategy should include the module but skip test functions
        assert!(formatted_test.contains("Test module documentation"));
        assert!(formatted_test.contains("pub mod tests"));
        assert!(!formatted_test.contains("test_fn")); // Test function should be skipped
    }

    #[test]
    fn test_module_formatter_summary() {
        // Test public module
        let module = create_test_module("public_module", true, false);
        let formatted = module.format(BankStrategy::Summary).unwrap();

        // Summary strategy should include public modules with only public items
        assert!(formatted.contains("Test module documentation"));
        assert!(formatted.contains("pub mod public_module"));
        assert!(formatted.contains("module_fn1")); // Public function
        assert!(!formatted.contains("module_fn2")); // Private function should be skipped

        // Test private module
        let private_module = create_test_module("private_module", false, false);
        let formatted_private = private_module.format(BankStrategy::Summary).unwrap();

        // Summary strategy should skip private modules
        assert!(!formatted_private.contains("mod private_module"));
    }

    #[test]
    fn test_file_unit_formatter() {
        let file_unit = FileUnit {
            path: PathBuf::from("test/file.rs"),
            document: None,
            declares: vec![],
            modules: vec![
                create_test_module("public_module", true, false),
                create_test_module("private_module", false, false),
                create_test_module("tests", false, true),
            ],
            functions: vec![
                create_test_function("public_function", true, false),
                create_test_function("private_function", false, false),
                create_test_function("test_function", true, true),
            ],
            structs: vec![
                create_test_struct("PublicStruct", true),
                create_test_struct("PrivateStruct", false),
            ],
            traits: Vec::new(),
            impls: Vec::new(),
            source: Some("// Test source code".to_string()),
        };

        // Test Default strategy
        let formatted_default = file_unit.format(BankStrategy::Default).unwrap();
        assert_eq!(formatted_default, "// Test source code");

        // Test NoTests strategy
        let formatted_no_tests = file_unit.format(BankStrategy::NoTests).unwrap();
        assert!(formatted_no_tests.contains("public_module"));
        assert!(formatted_no_tests.contains("private_module"));
        assert!(!formatted_no_tests.contains("tests")); // Test module should be skipped
        assert!(formatted_no_tests.contains("public_function"));
        assert!(formatted_no_tests.contains("private_function"));
        assert!(!formatted_no_tests.contains("#[test]")); // Test attribute should be skipped

        // Test Summary strategy
        let formatted_summary = file_unit.format(BankStrategy::Summary).unwrap();
        assert!(formatted_summary.contains("public_module"));
        assert!(!formatted_summary.contains("private_module")); // Private module should be skipped
        assert!(!formatted_summary.contains("tests")); // Test module should be skipped
        assert!(formatted_summary.contains("public_function"));
        assert!(!formatted_summary.contains("private_function")); // Private function should be skipped
    }
}
