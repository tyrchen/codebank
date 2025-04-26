use crate::{
    Error, FileUnit, FunctionUnit, LanguageParser, ModuleUnit, ParameterUnit, Result, RustParser,
    Visibility,
};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::{Node, Parser};

impl RustParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }

    // Helper function to extract documentation from doc comments
    fn extract_documentation(&self, node: Node, source_code: &str) -> Option<String> {
        let mut doc_comments = Vec::new();
        let mut cursor = node.walk();

        // Walk through the children to find documentation comments
        for child in node.children(&mut cursor) {
            if child.kind() == "line_comment"
                && child
                    .utf8_text(source_code.as_bytes())
                    .map(|s| s.starts_with("///"))
                    .unwrap_or(false)
            {
                if let Ok(comment) = child.utf8_text(source_code.as_bytes()) {
                    let cleaned = comment.trim_start_matches("///").trim().to_string();
                    doc_comments.push(cleaned);
                }
            } else if child.kind() == "block_comment"
                && child
                    .utf8_text(source_code.as_bytes())
                    .map(|s| s.starts_with("/**"))
                    .unwrap_or(false)
            {
                if let Ok(comment) = child.utf8_text(source_code.as_bytes()) {
                    let lines: Vec<&str> = comment.lines().collect();
                    if lines.len() > 1 {
                        for line in &lines[1..lines.len() - 1] {
                            let cleaned = line.trim_start_matches('*').trim().to_string();
                            if !cleaned.is_empty() {
                                doc_comments.push(cleaned);
                            }
                        }
                    }
                }
            }
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }

    // Helper function to determine visibility
    fn determine_visibility(&self, node: Node, source_code: &str) -> Visibility {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                if let Ok(vis_text) = child.utf8_text(source_code.as_bytes()) {
                    match vis_text {
                        "pub" => return Visibility::Public,
                        "pub(crate)" => return Visibility::Crate,
                        s if s.starts_with("pub(") => return Visibility::Restricted(s.to_string()),
                        _ => break,
                    }
                }
            }
        }

        Visibility::Private
    }

    // Parse function and extract its details
    fn parse_function(&self, node: Node, source_code: &str) -> Result<FunctionUnit> {
        let mut name = "unknown".to_string();
        let mut cursor = node.walk();
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let mut parameters = Vec::new();
        let mut return_type = None;

        // Extract function name
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(ident) = child.utf8_text(source_code.as_bytes()) {
                    name = ident.to_string();
                    break;
                }
            }
        }

        // Reset cursor for parsing parameters
        cursor = node.walk();

        // Extract parameters
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" {
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "parameter" {
                        let mut param_name = "unknown".to_string();
                        let mut param_type = "unknown".to_string();
                        let mut is_self = false;

                        let mut inner_cursor = param.walk();
                        for part in param.children(&mut inner_cursor) {
                            if part.kind() == "identifier" {
                                if let Ok(ident) = part.utf8_text(source_code.as_bytes()) {
                                    param_name = ident.to_string();
                                    if ident == "self" {
                                        is_self = true;
                                        param_type = "Self".to_string();
                                    }
                                }
                            } else if part.kind() == "type_identifier"
                                || part.kind() == "primitive_type"
                            {
                                if let Ok(type_ident) = part.utf8_text(source_code.as_bytes()) {
                                    param_type = type_ident.to_string();
                                }
                            }
                        }

                        parameters.push(ParameterUnit {
                            name: param_name,
                            parameter_type: param_type,
                            is_self,
                        });
                    }
                }
            }
        }

        // Reset cursor for return type
        cursor = node.walk();

        // Extract return type
        for child in node.children(&mut cursor) {
            if child.kind() == "return_type" {
                let mut inner_cursor = child.walk();
                for type_node in child.children(&mut inner_cursor) {
                    if let Ok(type_str) = type_node.utf8_text(source_code.as_bytes()) {
                        return_type = Some(type_str.to_string());
                        break;
                    }
                }
            }
        }

        // Extract source code
        let source = if let Ok(func_source) = node.utf8_text(source_code.as_bytes()) {
            Some(func_source.to_string())
        } else {
            None
        };

        Ok(FunctionUnit {
            name,
            visibility,
            documentation,
            parameters,
            return_type,
            source,
        })
    }

    // Parse modules
    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let mut name = "unknown".to_string();
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);

        let mut cursor = node.walk();

        // Extract module name
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                if let Ok(ident) = child.utf8_text(source_code.as_bytes()) {
                    name = ident.to_string();
                    break;
                }
            }
        }

        // For module blocks, we would need to parse the contents
        // This is simplified and would need to recursively parse the module contents
        let functions = Vec::new();
        let structs = Vec::new();
        let traits = Vec::new();
        let impls = Vec::new();
        let submodules = Vec::new();

        // Extract source code
        let source = if let Ok(mod_source) = node.utf8_text(source_code.as_bytes()) {
            Some(mod_source.to_string())
        } else {
            None
        };

        Ok(ModuleUnit {
            name,
            visibility,
            documentation,
            functions,
            structs,
            traits,
            impls,
            submodules,
            source,
        })
    }
}

impl LanguageParser for RustParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        // Read the file content
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;

        // Parse the file with tree-sitter
        let tree = self
            .parse(source_code.as_bytes(), None)
            .ok_or_else(|| Error::Parse("Failed to parse file".to_string()))?;

        let root_node = tree.root_node();

        // Initialize file unit
        let mut file_unit = FileUnit {
            path: file_path.to_path_buf(),
            modules: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            source: Some(source_code.clone()),
        };

        // Traverse the AST to extract code units
        let mut cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            match node.kind() {
                "function_item" => {
                    let function = self.parse_function(node, &source_code)?;
                    file_unit.functions.push(function);
                }
                "mod_item" => {
                    let module = self.parse_module(node, &source_code)?;
                    file_unit.modules.push(module);
                }
                // Add support for other code units like structs, traits, impls
                // For now, this is a simplified implementation
                _ => continue,
            }
        }

        Ok(file_unit)
    }
}

impl Deref for RustParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for RustParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rust_parser_sample() {
        // Create a new Rust parser
        let mut parser = RustParser::try_new().expect("Failed to create Rust parser");

        // Parse the sample.rs fixture
        let file_path = Path::new("fixtures/sample.rs");
        let file_unit = parser
            .parse_file(file_path)
            .expect("Failed to parse sample.rs");

        // Verify the basic structure of the parsed file
        assert_eq!(file_unit.path, file_path);

        // Print function information for debugging
        println!("Functions found: {}", file_unit.functions.len());
        for (i, function) in file_unit.functions.iter().enumerate() {
            println!(
                "Function {}: name={}, visibility={:?}, has_docs={}",
                i,
                function.name,
                function.visibility,
                function.documentation.is_some()
            );
        }

        // Print module information for debugging
        println!("Modules found: {}", file_unit.modules.len());
        for (i, module) in file_unit.modules.iter().enumerate() {
            println!(
                "Module {}: name={}, visibility={:?}, has_docs={}",
                i,
                module.name,
                module.visibility,
                module.documentation.is_some()
            );
        }

        // Check that we have some content extracted
        assert!(!file_unit.functions.is_empty(), "No functions were parsed");
        assert!(!file_unit.modules.is_empty(), "No modules were parsed");

        // Basic test to ensure parsing is working
        assert!(
            file_unit.functions.iter().any(|f| !f.name.is_empty()),
            "No function names were parsed"
        );
        assert!(
            file_unit.modules.iter().any(|m| !m.name.is_empty()),
            "No module names were parsed"
        );
    }
}
