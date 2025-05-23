mod python;
mod rules;
mod rust; // Uncommented
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
                if let Some(doc) = &self.doc {
                    output.push_str(&format!("{} {}\n", rules.doc_marker, doc.replace("\n", &format!("\n{} ", rules.doc_marker))));
                }
                for decl in &self.declares { output.push_str(&decl.source); output.push('\n'); }
                for module in &self.modules { if !rules.is_test_module(&module.name, &module.attributes) { let fmt = module.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for function in &self.functions { if !rules.is_test_function(&function.attributes) { let fmt = function.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for struct_unit in &self.structs { let fmt = struct_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}
                for trait_unit in &self.traits { let fmt = trait_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}
                for impl_unit in &self.impls { let fmt = impl_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}
            }
            BankStrategy::Summary => {
                if let Some(doc) = &self.doc {
                     output.push_str(&format!("{} {}\n", rules.doc_marker, doc.replace("\n", &format!("\n{} ", rules.doc_marker))));
                }
                for decl in &self.declares { output.push_str(&decl.source); output.push('\n'); } // Keep all declarations
                for module in &self.modules { if module.visibility == Visibility::Public { let fmt = module.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for function in &self.functions { if function.visibility == Visibility::Public { let fmt = function.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for struct_unit in &self.structs { if struct_unit.visibility == Visibility::Public { let fmt = struct_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for trait_unit in &self.traits { if trait_unit.visibility == Visibility::Public { let fmt = trait_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}}
                for impl_unit in &self.impls { let fmt = impl_unit.format(strategy, language)?; if !fmt.is_empty() { output.push_str(&fmt); output.push('\n'); }}
            }
        }
        // Ensure a single trailing newline, but not if the output is empty.
        let trimmed_output = output.trim_end_matches('\n');
        if trimmed_output.is_empty() { Ok(String::new()) } else { Ok(format!("{}\n", trimmed_output)) }
    }
}

// Implement Formatter for ModuleUnit
impl Formatter for ModuleUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        if *strategy == BankStrategy::Summary && rules.is_test_module(&self.name, &self.attributes) { return Ok(String::new()); }

        match strategy {
            BankStrategy::Default => { if let Some(source) = &self.source { output.push_str(source); } }
            BankStrategy::NoTests => {
                if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
                for attr in &self.attributes { output.push_str(&format!("{}\n", attr)); }
                let vis_str = self.visibility.as_str(language);
                output.push_str(&format!("{}mod {} {{\n", if vis_str.is_empty() {""} else {vis_str.trim_end()}, self.name)); // Ensure no double space if vis_str is empty
                for decl in &self.declares { output.push_str(&format!("    {}\n", decl.source)); }
                for function in &self.functions { if !rules.is_test_function(&function.attributes) { let f_fmt = function.format(strategy, language)?; if !f_fmt.is_empty() {output.push_str(&format!("    {}\n\n", f_fmt.replace("\n", "\n    ")));}}}
                for struct_unit in &self.structs { let s_fmt = struct_unit.format(strategy, language)?; if !s_fmt.is_empty() {output.push_str(&format!("    {}\n\n", s_fmt.replace("\n", "\n    ")));}}
                for trait_unit in &self.traits { let t_fmt = trait_unit.format(strategy, language)?; if !t_fmt.is_empty() {output.push_str(&format!("    {}\n\n", t_fmt.replace("\n", "\n    ")));}}
                for impl_unit in &self.impls { let i_fmt = impl_unit.format(strategy, language)?; if !i_fmt.is_empty() {output.push_str(&format!("    {}\n\n", i_fmt.replace("\n", "\n    ")));}}
                for submodule in &self.submodules { let sub_fmt = submodule.format(strategy, language)?; if !sub_fmt.is_empty() {output.push_str(&format!("    {}\n\n", sub_fmt.replace("\n", "\n    ")));}}
                output.push_str("}\n");
            }
            BankStrategy::Summary => {
                if self.visibility == Visibility::Public {
                    let fns: Vec<&FunctionUnit> = self.functions.iter().filter(|f| f.visibility == Visibility::Public && !rules.is_test_function(&f.attributes)).collect();
                    let structs: Vec<&StructUnit> = self.structs.iter().filter(|s| s.visibility == Visibility::Public).collect();
                    let traits: Vec<&TraitUnit> = self.traits.iter().filter(|t| t.visibility == Visibility::Public).collect();
                    let impls_to_format: Vec<String> = self.impls.iter().map(|i| i.format(strategy, language).unwrap_or_default()).filter(|s| !s.is_empty()).collect();
                    let mods: Vec<&ModuleUnit> = self.submodules.iter().filter(|m| m.visibility == Visibility::Public && !m.format(strategy, language).unwrap_or_default().is_empty() ).collect();

                    if fns.is_empty() && structs.is_empty() && traits.is_empty() && impls_to_format.is_empty() && mods.is_empty() && self.declares.is_empty() {
                        return Ok(String::new()); 
                    }

                    if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
                    for attr in &self.attributes { if !rules.test_module_markers.contains(&attr.as_str()) { output.push_str(&format!("{}\n", attr)); }}
                    output.push_str(&format!("pub mod {} {{\n", self.name));
                    for decl in &self.declares { output.push_str(&format!("    {}\n", decl.source)); }
                    if !self.declares.is_empty() && (!fns.is_empty() || !structs.is_empty() || !traits.is_empty() || !impls_to_format.is_empty() || !mods.is_empty()) { output.push('\n'); }

                    for function in fns { let f_fmt = function.format(strategy, language)?; if !f_fmt.is_empty() {output.push_str(&format!("    {}\n\n", f_fmt.replace("\n", "\n    ")));}}
                    for struct_unit in structs { let s_fmt = struct_unit.format(strategy, language)?; if !s_fmt.is_empty() {output.push_str(&format!("    {}\n\n", s_fmt.replace("\n", "\n    ")));}}
                    for trait_unit in traits { let t_fmt = trait_unit.format(strategy, language)?; if !t_fmt.is_empty() {output.push_str(&format!("    {}\n\n", t_fmt.replace("\n", "\n    ")));}}
                    for impl_formatted_str in impls_to_format { if !impl_formatted_str.is_empty() {output.push_str(&format!("    {}\n\n", impl_formatted_str.replace("\n", "\n    ")));}}
                    for submodule in mods { let sub_fmt = submodule.format(strategy, language)?; if !sub_fmt.is_empty() {output.push_str(&format!("    {}\n\n", sub_fmt.replace("\n", "\n    ")));}}
                    output.push_str("}\n");
                }
            }
        }
        Ok(output.trim_end_matches('\n').to_string() + "\n")
    }
}

// Implement Formatter for FunctionUnit
impl Formatter for FunctionUnit {
    fn format(&self, strategy: &BankStrategy, language: LanguageType) -> Result<String> {
        let mut output = String::new();
        let rules = FormatterRules::for_language(language);

        if *strategy == BankStrategy::Default { return Ok(self.source.clone().unwrap_or_default()); }
        if rules.is_test_function(&self.attributes) { return Ok(String::new()); }
        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public { return Ok(String::new()); }

        if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
        for attr in &self.attributes { if !rules.test_markers.contains(&attr.as_str()) { output.push_str(&format!("{}\n", attr)); }}

        match strategy {
            BankStrategy::Default => {} 
            BankStrategy::NoTests => {
                if let Some(sig) = &self.signature { output.push_str(sig); }
                if let Some(body) = &self.body {
                    if self.signature.is_some() && !output.ends_with(' ') && !body.starts_with('{') && !body.starts_with(':') && language == LanguageType::Rust { output.push(' '); }
                    output.push_str(body);
                } else if self.signature.is_none() { if let Some(src) = &self.source { output.push_str(src); }}
            }
            BankStrategy::Summary => {
                if let Some(signature) = &self.signature {
                    let formatted_sig = rules.format_signature(signature, Some(signature)); 
                    output.push_str(&formatted_sig);
                } else if let Some(source) = &self.source { 
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

        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public { return Ok(String::new()); }
        
        if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
        for attr in &self.attributes { output.push_str(&format!("{}\n", attr)); }

        match strategy {
            BankStrategy::Default | BankStrategy::NoTests => {
                if let Some(source) = &self.source { output.push_str(source); }
            }
            BankStrategy::Summary => {
                output.push_str(&self.head); 
                if !self.head.ends_with(';') { 
                    output.push_str(rules.function_body_start_marker); 
                    output.push('\n');
                    for (i, field) in self.fields.iter().enumerate() {
                        if let Some(field_src) = field.source.as_ref() {
                            let field_src_trimmed = field_src.trim_end_matches(','); // Remove trailing comma from source itself
                            output.push_str("    ");
                            output.push_str(field_src_trimmed);
                            // Add comma only if it's not the last field and field_sep is a comma
                            if i < self.fields.len() - 1 && rules.field_sep == "," {
                                output.push_str(",");
                            }
                            output.push('\n');
                        }
                    }
                    output.push_str(rules.function_body_end_marker); 
                }
                // Methods are handled by ImplUnit formatting
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

        if *strategy == BankStrategy::Summary && self.visibility != Visibility::Public { return Ok(String::new()); }
        
        if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
        for attr in &self.attributes { output.push_str(&format!("{}\n", attr)); }

        match strategy {
            BankStrategy::Default => { if let Some(source) = &self.source { output.push_str(source); } }
            BankStrategy::NoTests => { // For NoTests, full source is generally preferred if available and accurate
                 if let Some(source) = &self.source { 
                    output.push_str(source);
                } else { // Fallback if no source, construct from parts
                    // Assuming TraitUnit has a `head: String` field populated by the parser.
                    // If not, this needs to be `format!("{} trait {} ...", vis, name)`
                    // output.push_str(&self.head); // Ideal if TraitUnit.head exists and is accurate
                    // Fallback for current TraitUnit structure:
                     output.push_str(&format!("{} trait {}", self.visibility.as_str(language), self.name)); // Simplified
                    output.push_str(" {\n"); 
                    for method in &self.methods { if !rules.is_test_function(&method.attributes) { let m_fmt = method.format(strategy, language)?; if !m_fmt.is_empty() {output.push_str(&format!("    {}\n", m_fmt.replace("\n", "\n    ")));}}}
                    output.push_str(rules.function_body_end_marker);
                }
            }
            BankStrategy::Summary => {
                // Use self.head (assuming TraitUnit will have this field populated by the parser with generics)
                // For now, using a placeholder construction based on current TraitUnit structure
                // This should be: output.push_str(&self.head);
                // The parser now provides `head` in `TraitUnit`, so we assume it's available.
                // If `TraitUnit` struct definition in `parser/mod.rs` isn't updated with `head: String`,
                // this will fail. For now, constructing it simply.
                // UPDATE: The `TraitUnit` struct in `src/parser/mod.rs` does NOT have a `head` field.
                // The `parse_trait` *does* create `head_str`. This indicates a mismatch to be fixed in `TraitUnit` struct.
                // For now, I will try to use the source to get the head for summary.
                let trait_head_for_summary = self.source.as_ref()
                    .and_then(|s| s.lines().next()) // Get first line
                    .map_or_else(
                        || format!("{} trait {}", self.visibility.as_str(language), self.name), // Fallback
                        |s| s.split('{').next().unwrap_or("").trim().to_string() // Get part before '{'
                    );
                output.push_str(&trait_head_for_summary);
                output.push_str(rules.summary_ellipsis);
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

        let methods_to_include: Vec<&FunctionUnit> = match strategy {
            BankStrategy::Default => self.methods.iter().collect(),
            BankStrategy::NoTests => self.methods.iter().filter(|m| !rules.is_test_function(&m.attributes)).collect(),
            BankStrategy::Summary => {
                if is_trait_impl { self.methods.iter().filter(|m| !rules.is_test_function(&m.attributes)).collect() } 
                else { self.methods.iter().filter(|m| m.visibility == Visibility::Public && !rules.is_test_function(&m.attributes)).collect() }
            }
        };

        if methods_to_include.is_empty() && *strategy == BankStrategy::Summary && !is_trait_impl { return Ok(String::new()); }
        if let Some(doc) = &self.doc { for line in doc.lines() { output.push_str(&format!("{} {}\n", rules.doc_marker, line)); }}
        for attr in &self.attributes { output.push_str(&format!("{}\n", attr)); }

        match strategy {
            BankStrategy::Default => { if let Some(source) = &self.source { output.push_str(source); } }
            BankStrategy::NoTests | BankStrategy::Summary => {
                output.push_str(&self.head); 
                output.push_str(" {\n");
                for method in methods_to_include {
                    let method_formatted = method.format(strategy, language)?; 
                    if !method_formatted.is_empty() { output.push_str(&format!("    {}\n", method_formatted.replace("\n", "\n    "))); }
                }
                output.push_str(rules.function_body_end_marker);
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    // These tests are generic and test the FormatterRules and basic strategy dispatch.
    // Language-specific formatting tests will be in `src/parser/formatter/rust.rs`.
    use super::*;
    use crate::parser::Visibility;

    #[test]
    fn test_function_unit_format_dispatch() { 
        let function = FunctionUnit {
            name: "test_function".to_string(), visibility: Visibility::Public,
            doc: Some("Doc".to_string()), signature: Some("fn test()".to_string()), body: Some("{ }".to_string()),
            source: Some("fn test() { }".to_string()), attributes: vec!["#[test]".to_string()],
        };
        assert_eq!(function.format(&BankStrategy::Default, LanguageType::Rust).unwrap(), "fn test() { }");
        assert_eq!(function.format(&BankStrategy::NoTests, LanguageType::Rust).unwrap(), ""); 
        assert_eq!(function.format(&BankStrategy::Summary, LanguageType::Rust).unwrap(), ""); 

        let regular_function = FunctionUnit {
            name: "regular".to_string(), visibility: Visibility::Public, doc: None, 
            signature: Some("pub fn regular()".to_string()), body: Some("{ }".to_string()), source: None, attributes: vec![],
        };
        assert!(regular_function.format(&BankStrategy::NoTests, LanguageType::Rust).unwrap().contains("pub fn regular() { }"));
        assert!(regular_function.format(&BankStrategy::Summary, LanguageType::Rust).unwrap().contains("pub fn regular() { ... }"));
    }
}
