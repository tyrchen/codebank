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

                // Include all declare statements
                for declare in &self.declares {
                    output.push_str(&declare.source);
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

                // Format and include impls
                for impl_unit in &self.impls {
                    let impl_formatted = impl_unit.format(strategy.clone())?;
                    output.push_str(&impl_formatted);
                    output.push_str("\n\n");
                }

                // Format and include non-test modules
                for module in non_test_modules {
                    let module_formatted = module.format(strategy.clone())?;
                    output.push_str(&module_formatted);
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

                // Include all declare statements
                for declare in &self.declares {
                    output.push_str(&format!("    {}\n\n", declare.source));
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

                    // Add impls (only public methods)
                    for impl_unit in &self.impls {
                        let impl_formatted = impl_unit.format(strategy.clone())?;
                        output.push_str(&format!(
                            "    {}\n\n",
                            impl_formatted.replace("\n", "\n    ")
                        ));
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
                // Skip functions with #[test] attribute
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

                // Add function signature and body
                if let Some(sig) = &self.signature {
                    output.push_str(sig);
                    if let Some(body) = &self.body {
                        output.push(' ');
                        output.push_str(body);
                    } else {
                        output.push(';');
                    }
                } else if let Some(source) = &self.source {
                    // Fallback to source if signature/body splitting failed
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
                        output.push_str(&format!("/// {}\n", line));
                    }
                }

                // Add attributes
                for attr in &self.attributes {
                    output.push_str(&format!("{}\n", attr));
                }

                // Add function signature only (no body)
                if let Some(sig) = &self.signature {
                    output.push_str(sig);
                    output.push_str("{ ... }");
                } else if let Some(source) = &self.source {
                    // Try to extract just the signature from the source
                    if let Some(idx) = source.find('{') {
                        output.push_str(source[0..idx].trim());
                        output.push_str("{ ... }");
                    } else {
                        // Fallback: use the whole source (likely already a signature-only item)
                        output.push_str(source);
                    }
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
            BankStrategy::NoTests | BankStrategy::Summary => {
                // Only include public structs for Summary
                if strategy == BankStrategy::Summary
                    && !matches!(self.visibility, Visibility::Public)
                {
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

                // Two possibilities for formatting:
                // 1. If we have source code, we can try to extract just the declaration
                // 2. Otherwise, generate a basic declaration

                if let Some(source) = &self.source {
                    // Try to extract just the struct declaration (without methods)
                    if let Some(end_idx) = source.find('{') {
                        // Include the opening brace and get ending brace
                        let mut brace_count = 1;
                        let mut end_pos = end_idx + 1;

                        for (i, c) in source[end_idx + 1..].char_indices() {
                            if c == '{' {
                                brace_count += 1;
                            } else if c == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    end_pos = end_idx + 1 + i + 1; // +1 to include the closing brace
                                    break;
                                }
                            }
                        }

                        if end_pos > end_idx + 1 {
                            output.push_str(&source[0..end_pos]);
                        } else {
                            // Fallback if we couldn't find the closing brace
                            output.push_str(source);
                        }
                    } else {
                        // Entire source might be a single-line declaration
                        output.push_str(source);
                    }
                } else {
                    // Generate a basic struct declaration
                    match self.visibility {
                        Visibility::Public => {
                            output.push_str(&format!("pub struct {} {{}}", self.name))
                        }
                        _ => output.push_str(&format!("struct {} {{}}", self.name)),
                    }
                }

                // For NoTests, show all methods except test methods
                if strategy == BankStrategy::NoTests {
                    // Skip test methods but include all others regardless of visibility
                    let non_test_methods: Vec<&FunctionUnit> = self
                        .methods
                        .iter()
                        .filter(|m| !m.attributes.iter().any(|attr| attr == "#[test]"))
                        .collect();

                    if !non_test_methods.is_empty() {
                        output.push_str("\n\n");
                        output.push_str(&format!("impl {} {{", self.name));
                        for method in non_test_methods {
                            let method_formatted = method.format(strategy.clone())?;
                            if !method_formatted.is_empty() {
                                output.push_str("\n    ");
                                output.push_str(&method_formatted.replace("\n", "\n    "));
                            }
                        }
                        output.push_str("\n}");
                    }
                }

                // For Summary, show only public methods
                if strategy == BankStrategy::Summary {
                    let public_methods: Vec<&FunctionUnit> = self
                        .methods
                        .iter()
                        .filter(|m| matches!(m.visibility, Visibility::Public))
                        .collect();

                    if !public_methods.is_empty() {
                        output.push_str("\n\n");
                        output.push_str(&format!("impl {} {{", self.name));
                        for method in public_methods {
                            let method_formatted = method.format(strategy.clone())?;
                            if !method_formatted.is_empty() {
                                output.push_str("\n    ");
                                output.push_str(&method_formatted.replace("\n", "\n    "));
                            }
                        }
                        output.push_str("\n}");
                    }
                }
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
            BankStrategy::NoTests | BankStrategy::Summary => {
                // Only include public traits for Summary
                if strategy == BankStrategy::Summary
                    && !matches!(self.visibility, Visibility::Public)
                {
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

                // Start trait declaration
                match self.visibility {
                    Visibility::Public => output.push_str(&format!("pub trait {} {{\n", self.name)),
                    _ => output.push_str(&format!("trait {} {{\n", self.name)),
                }

                // Add method signatures
                for method in &self.methods {
                    // Skip test methods
                    if strategy == BankStrategy::NoTests
                        && method.attributes.iter().any(|attr| attr == "#[test]")
                    {
                        continue;
                    }

                    // Only include public methods for Summary
                    if strategy == BankStrategy::Summary
                        && !matches!(method.visibility, Visibility::Public)
                    {
                        continue;
                    }

                    let method_formatted = method.format(strategy.clone())?;
                    if !method_formatted.is_empty() {
                        output.push_str("    ");
                        output.push_str(&method_formatted.replace("\n", "\n    "));
                        output.push('\n');
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
            BankStrategy::NoTests | BankStrategy::Summary => {
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

                // Check if this is a trait implementation (contains 'impl SomeTrait for')
                let is_trait_impl = if let Some(source) = &self.source {
                    source.contains(" for ") && source.contains("impl ")
                } else {
                    false
                };

                // Try to generate a reasonable impl block even if we don't have source
                // We'll have to infer the target type from the source or use placeholder
                if let Some(source) = &self.source {
                    // Try to extract just the impl declaration (without method bodies for Summary)
                    if let Some(idx) = source.find('{') {
                        output.push_str(&source[0..=idx]);
                    } else {
                        // If no opening brace is found, use the whole declaration
                        output.push_str(source);
                        output.push_str(" {");
                    }
                } else {
                    // Fallback if we don't have source
                    output.push_str("impl /* unnamed type */ {");
                }

                // Add methods - handle trait implementations differently
                let methods_to_include = match strategy {
                    BankStrategy::NoTests => self
                        .methods
                        .iter()
                        .filter(|m| !m.attributes.iter().any(|attr| attr == "#[test]"))
                        .collect::<Vec<_>>(),
                    BankStrategy::Summary => {
                        // For trait implementations, include all methods regardless of visibility
                        // For regular impls, only include public methods
                        if is_trait_impl {
                            self.methods
                                .iter()
                                .filter(|m| !m.attributes.iter().any(|attr| attr == "#[test]"))
                                .collect::<Vec<_>>()
                        } else {
                            self.methods
                                .iter()
                                .filter(|m| matches!(m.visibility, Visibility::Public))
                                .collect::<Vec<_>>()
                        }
                    }
                    _ => unreachable!(),
                };

                for method in methods_to_include {
                    // Standard method formatting
                    let mut method_formatted = method.format(strategy.clone())?;

                    // For trait implementations in Summary mode, replace empty results
                    // with a formatted version that treats the method as public
                    if is_trait_impl
                        && strategy == BankStrategy::Summary
                        && method_formatted.is_empty()
                    {
                        // Format with documentation and signature but indicate it's a trait method
                        let mut public_method_str = String::new();

                        // Add documentation
                        if let Some(doc) = &method.documentation {
                            for line in doc.lines() {
                                public_method_str.push_str(&format!("/// {}\n", line));
                            }
                        }

                        // Add attributes
                        for attr in &method.attributes {
                            public_method_str.push_str(&format!("{}\n", attr));
                        }

                        // Add function signature with trait implementation marker
                        if let Some(sig) = &method.signature {
                            public_method_str.push_str(sig);
                            public_method_str.push_str(" { ... }");
                        } else if let Some(source) = &method.source {
                            if let Some(idx) = source.find('{') {
                                public_method_str.push_str(source[0..idx].trim());
                                public_method_str.push_str(" { ... }");
                            } else {
                                public_method_str.push_str(source);
                            }
                        }

                        method_formatted = public_method_str;
                    }

                    if !method_formatted.is_empty() {
                        output.push_str("\n    ");
                        output.push_str(&method_formatted.replace("\n", "\n    "));
                    }
                }

                output.push_str("\n}");
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeclareKind;
    use crate::DeclareStatements;

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
            documentation: Some(format!("Documentation for {}", name)),
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

        StructUnit {
            name: name.to_string(),
            attributes: Vec::new(),
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            documentation: Some(format!("Documentation for {}", name)),
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
            document: Some(format!("Documentation for module {}", name)),
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

        let source = if is_trait_impl {
            Some("impl SomeTrait for SomeStruct { /* impl body */ }".to_string())
        } else {
            Some("impl SomeStruct { /* impl body */ }".to_string())
        };

        ImplUnit {
            attributes: Vec::new(),
            documentation: Some("Documentation for implementation".to_string()),
            methods,
            source,
        }
    }

    #[test]
    fn test_function_formatter_default() {
        let function = create_test_function("test_function", true, false);
        let formatted = function.format(BankStrategy::Default).unwrap();
        assert!(formatted.contains("fn test_function()"));
        assert!(formatted.contains("/* function body */"));
    }

    #[test]
    fn test_function_formatter_no_tests() {
        // Regular function
        let function = create_test_function("regular_function", true, false);
        let formatted = function.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("fn regular_function()"));
        assert!(formatted.contains("/* function body */"));

        // Test function
        let test_function = create_test_function("test_function", true, true);
        let formatted = test_function.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_function_formatter_summary() {
        // Public function
        let public_function = create_test_function("public_function", true, false);
        let formatted = public_function.format(BankStrategy::Summary).unwrap();
        assert!(formatted.contains("fn public_function()"));
        assert!(!formatted.contains("/* function body */"));
        assert!(formatted.contains("{ ... }"));

        // Private function
        let private_function = create_test_function("private_function", false, false);
        let formatted = private_function.format(BankStrategy::Summary).unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_struct_formatter_default() {
        let struct_unit = create_test_struct("TestStruct", true);
        let formatted = struct_unit.format(BankStrategy::Default).unwrap();
        assert!(formatted.contains("struct TestStruct"));
        assert!(formatted.contains("field: i32"));
    }

    #[test]
    fn test_struct_formatter_summary() {
        // Public struct
        let public_struct = create_test_struct("PublicStruct", true);
        let formatted = public_struct.format(BankStrategy::Summary).unwrap();
        assert!(formatted.contains("struct PublicStruct"));

        // Methods should only show signatures
        assert!(formatted.contains("impl PublicStruct"));
        assert!(formatted.contains("fn publicstruct_method()"));
        assert!(!formatted.contains("/* function body */"));

        // Private struct
        let private_struct = create_test_struct("PrivateStruct", false);
        let formatted = private_struct.format(BankStrategy::Summary).unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_module_formatter_default() {
        let module = create_test_module("test_module", true, false);
        let formatted = module.format(BankStrategy::Default).unwrap();
        assert!(formatted.contains("mod test_module"));
        assert!(formatted.contains("/* module contents */"));
    }

    #[test]
    fn test_module_formatter_no_tests() {
        // Regular module
        let module = create_test_module("regular_module", true, false);
        let formatted = module.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("pub mod regular_module"));
        assert!(formatted.contains("fn module_function"));
        assert!(formatted.contains("fn module_private_function"));
        assert!(formatted.contains("struct ModuleStruct"));
        assert!(formatted.contains("use std::io;"));

        // Test module
        let test_module = create_test_module("test_module", true, true);
        let formatted = test_module.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("#[cfg(test)]"));
        assert!(formatted.contains("pub mod test_module"));
    }

    #[test]
    fn test_module_formatter_summary() {
        // Public module
        let public_module = create_test_module("public_module", true, false);
        let formatted = public_module.format(BankStrategy::Summary).unwrap();
        assert!(formatted.contains("pub mod public_module"));
        assert!(formatted.contains("fn module_function()"));
        // Functions should only show signatures in summary
        assert!(!formatted.contains("/* function body */"));

        // Private module
        let private_module = create_test_module("private_module", false, false);
        let formatted = private_module.format(BankStrategy::Summary).unwrap();
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_struct_formatter_no_tests() {
        // Test struct with private methods
        let struct_unit = create_test_struct("TestStruct", true);
        let formatted = struct_unit.format(BankStrategy::NoTests).unwrap();

        // Should include both public and private methods
        assert!(formatted.contains("fn teststruct_method()")); // public method
        assert!(formatted.contains("fn teststruct_private_method()")); // private method
    }

    #[test]
    fn test_regular_impl_formatter_summary() {
        // Regular (non-trait) implementation
        let impl_unit = create_test_impl(false);
        let formatted = impl_unit.format(BankStrategy::Summary).unwrap();

        // Only public methods should be included in regular impls
        assert!(formatted.contains("impl SomeStruct"));
        assert!(formatted.contains("fn public_method"));
        assert!(!formatted.contains("fn private_method"));
    }

    #[test]
    fn test_trait_impl_formatter_summary() {
        // Trait implementation
        let impl_unit = create_test_impl(true);
        let formatted = impl_unit.format(BankStrategy::Summary).unwrap();

        // Both public and private methods should be included in trait impls
        assert!(formatted.contains("impl SomeTrait for SomeStruct"));
        assert!(formatted.contains("fn public_method"));
        assert!(formatted.contains("fn private_method"));
    }

    #[test]
    fn test_impl_formatter_no_tests() {
        // Both regular and trait implementation should include all non-test methods in NoTests mode
        let regular_impl = create_test_impl(false);
        let formatted = regular_impl.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("fn public_method"));
        assert!(formatted.contains("fn private_method"));

        let trait_impl = create_test_impl(true);
        let formatted = trait_impl.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("fn public_method"));
        assert!(formatted.contains("fn private_method"));
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
        let formatted = file_unit.format(BankStrategy::Default).unwrap();
        assert_eq!(formatted, "// This is the entire file content");

        // Test NoTests strategy - test modules and functions should be excluded
        let formatted = file_unit.format(BankStrategy::NoTests).unwrap();
        assert!(formatted.contains("pub mod public_module"));
        assert!(!formatted.contains("fn test_function"));
        assert!(formatted.contains("fn public_function"));
        assert!(formatted.contains("fn private_function"));
        assert!(formatted.contains("struct PublicStruct"));
        assert!(formatted.contains("struct PrivateStruct"));

        // Test Summary strategy - only public items should be included
        let formatted = file_unit.format(BankStrategy::Summary).unwrap();
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
        let formatted = file_unit.format(BankStrategy::NoTests).unwrap();

        // Should include all non-test items regardless of visibility
        assert!(formatted.contains("pub mod public_module"));
        assert!(formatted.contains("mod private_module"));
        assert!(!formatted.contains("fn test_function"));
        assert!(formatted.contains("fn public_function"));
        assert!(formatted.contains("fn private_function"));
        assert!(formatted.contains("struct PublicStruct"));
        assert!(formatted.contains("struct PrivateStruct"));
        assert!(formatted.contains("use std::collections::HashMap;"));
        assert!(formatted.contains("fn publicstruct_private_method()"));
    }
}
