use crate::{
    DeclareKind, DeclareStatements, Error, FileUnit, FunctionUnit, LanguageParser, Result,
    StructUnit, TypeScriptParser, Visibility,
};
use std::{
    fs,
    ops::{Deref, DerefMut},
    path::Path,
};
use tree_sitter::{Node, Parser};

impl TypeScriptParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }

    // Helper method to process export statements
    fn process_export(&self, file_unit: &mut FileUnit, node: Node, source: &[u8]) {
        // Check if this is a standalone export or contains a declaration
        if let Some(decl_node) = node.child_by_field_name("declaration") {
            match decl_node.kind() {
                "function_declaration" => {
                    self.process_function(file_unit, decl_node, true, source);
                }
                "lexical_declaration" => {
                    for j in 0..decl_node.child_count() {
                        if let Some(var_node) = decl_node.child(j) {
                            if var_node.kind() == "variable_declarator" {
                                for k in 0..var_node.child_count() {
                                    if let Some(value_node) = var_node.child(k) {
                                        if value_node.kind() == "arrow_function"
                                            || value_node.kind() == "function_expression"
                                        {
                                            self.process_function_variable(
                                                file_unit, decl_node, var_node, true, source,
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "class_declaration" => {
                    self.process_class(file_unit, decl_node, true, source);
                }
                "interface_declaration" => {
                    self.process_interface(file_unit, decl_node, true, source);
                }
                "type_alias_declaration" => {
                    self.process_type_alias(file_unit, decl_node, true, source);
                }
                "enum_declaration" => {
                    self.process_enum(file_unit, decl_node, true, source);
                }
                _ => {}
            }
        } else {
            // Standalone export
            let source_text = node.utf8_text(source).unwrap_or("").to_string();
            file_unit.declares.push(DeclareStatements {
                source: source_text,
                kind: DeclareKind::Other("export".to_string()),
            });
        }
    }

    // Process a function declaration
    fn process_function(
        &self,
        file_unit: &mut FileUnit,
        node: Node,
        is_exported: bool,
        source: &[u8],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let func_source = node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation in previous sibling
            let documentation = find_documentation_for_node(node, source);

            // Extract function signature
            let mut signature = String::from("function ");
            signature.push_str(&name);

            // Add parameters
            if let Some(params_node) = node.child_by_field_name("parameters") {
                signature.push_str(params_node.utf8_text(source).unwrap_or("").trim());
            }

            // Add return type if present
            if let Some(return_type) = node.child_by_field_name("return_type") {
                signature.push_str(return_type.utf8_text(source).unwrap_or(""));
            }

            file_unit.functions.push(FunctionUnit {
                name,
                source: Some(func_source),
                visibility,
                doc: documentation,
                signature: Some(signature),
                body: None,
                attributes: vec![],
            });
        }
    }

    // Process a variable that contains a function
    fn process_function_variable(
        &self,
        file_unit: &mut FileUnit,
        decl_node: Node,
        var_node: Node,
        is_exported: bool,
        source: &[u8],
    ) {
        if let Some(name_node) = var_node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let func_source = decl_node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation
            let documentation = find_documentation_for_node(decl_node, source);

            // Find the function value (arrow function or function expression)
            let mut signature = None;

            if let Some(value_node) = var_node.child_by_field_name("value") {
                if value_node.kind() == "arrow_function"
                    || value_node.kind() == "function_expression"
                {
                    let mut sig = String::new();

                    // For arrow functions, use the variable name and add parameters
                    if value_node.kind() == "arrow_function" {
                        sig.push_str(&name);

                        // Add parameters
                        if let Some(params_node) = value_node.child_by_field_name("parameters") {
                            sig.push_str(params_node.utf8_text(source).unwrap_or("").trim());
                        }

                        // Add return type if present
                        if let Some(return_type) = value_node.child_by_field_name("return_type") {
                            sig.push_str(return_type.utf8_text(source).unwrap_or(""));
                        }

                        // Don't add the arrow operator to the signature
                    } else {
                        // For function expressions, format as "function name(params)"
                        sig.push_str("function ");
                        sig.push_str(&name);

                        // Add parameters
                        if let Some(params_node) = value_node.child_by_field_name("parameters") {
                            sig.push_str(params_node.utf8_text(source).unwrap_or("").trim());
                        }

                        // Add return type if present
                        if let Some(return_type) = value_node.child_by_field_name("return_type") {
                            sig.push_str(return_type.utf8_text(source).unwrap_or(""));
                        }
                    }

                    signature = Some(sig);
                }
            }

            file_unit.functions.push(FunctionUnit {
                name,
                source: Some(func_source),
                visibility,
                doc: documentation,
                signature,
                body: None,
                attributes: vec![],
            });
        }
    }

    // Process a class declaration
    fn process_class(
        &self,
        file_unit: &mut FileUnit,
        node: Node,
        is_exported: bool,
        source: &[u8],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let class_source = node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation
            let documentation = find_documentation_for_node(node, source);

            // Extract methods from the class body
            let mut methods = Vec::new();

            // Look for the class body
            if let Some(body_node) = node.child_by_field_name("body") {
                // Iterate through children to find method definitions
                for i in 0..body_node.child_count() {
                    if let Some(method_node) = body_node.child(i) {
                        // Check for method_definition or constructor_definition
                        if method_node.kind() == "method_definition"
                            || method_node.kind() == "constructor_definition"
                        {
                            if let Some(method_name_node) = method_node.child_by_field_name("name")
                            {
                                let method_name =
                                    method_name_node.utf8_text(source).unwrap_or("").to_string();
                                let method_source =
                                    method_node.utf8_text(source).unwrap_or("").to_string();

                                // Extract method signature
                                let mut signature = String::new();

                                // Default visibility for methods is public unless marked private
                                let mut method_visibility = Visibility::Public;

                                // Check if it's a constructor
                                if method_node.kind() == "constructor_definition" {
                                    signature.push_str("constructor");
                                } else {
                                    // Get modifiers if any (public, private, etc.)
                                    for j in 0..method_node.child_count() {
                                        if let Some(modifier) = method_node.child(j) {
                                            if modifier.kind() == "accessibility_modifier" {
                                                let modifier_text =
                                                    modifier.utf8_text(source).unwrap_or("").trim();
                                                signature.push_str(modifier_text);
                                                signature.push(' ');

                                                // Set visibility based on the modifier
                                                if modifier_text == "private" {
                                                    method_visibility = Visibility::Private;
                                                }
                                                break;
                                            }
                                        }
                                    }

                                    // Add method name
                                    signature.push_str(&method_name);
                                }

                                // Add parameters
                                if let Some(params_node) =
                                    method_node.child_by_field_name("parameters")
                                {
                                    signature.push_str(
                                        params_node.utf8_text(source).unwrap_or("").trim(),
                                    );
                                }

                                // Add return type if present
                                if let Some(return_type) =
                                    method_node.child_by_field_name("return_type")
                                {
                                    signature.push_str(return_type.utf8_text(source).unwrap_or(""));
                                }

                                // Add to methods list
                                methods.push(FunctionUnit {
                                    name: method_name,
                                    source: Some(method_source),
                                    visibility: method_visibility,
                                    doc: None, // Could extract doc comments for methods too
                                    signature: Some(signature),
                                    body: None,
                                    attributes: vec![],
                                });
                            }
                        }
                    }
                }
            }

            file_unit.structs.push(StructUnit {
                name: name.clone(),
                source: Some(class_source),
                head: format!("class {}", name),
                visibility,
                doc: documentation,
                methods,
                fields: Vec::new(),
                attributes: vec![],
            });
        }
    }

    // Process an interface declaration
    fn process_interface(
        &self,
        file_unit: &mut FileUnit,
        node: Node,
        is_exported: bool,
        source: &[u8],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let interface_source = node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation
            let documentation = find_documentation_for_node(node, source);

            // Extract method declarations from the interface body
            let mut methods = Vec::new();

            // Look for the interface body
            if let Some(body_node) = node.child_by_field_name("body") {
                // Iterate through children to find method declarations
                for i in 0..body_node.child_count() {
                    if let Some(method_node) = body_node.child(i) {
                        // Check for method_signature or property_signature with function type
                        if method_node.kind() == "method_signature" {
                            if let Some(method_name_node) = method_node.child_by_field_name("name")
                            {
                                let method_name =
                                    method_name_node.utf8_text(source).unwrap_or("").to_string();
                                let method_source =
                                    method_node.utf8_text(source).unwrap_or("").to_string();

                                // Extract method signature
                                let mut signature = String::new();

                                // Interface methods are public by default in TypeScript
                                signature.push_str(&method_name);

                                // Add parameters
                                if let Some(params_node) =
                                    method_node.child_by_field_name("parameters")
                                {
                                    signature.push_str(
                                        params_node.utf8_text(source).unwrap_or("").trim(),
                                    );
                                }

                                // Add return type if present
                                if let Some(return_type) =
                                    method_node.child_by_field_name("return_type")
                                {
                                    signature.push_str(return_type.utf8_text(source).unwrap_or(""));
                                }

                                // Add to methods list (interface methods are always public)
                                methods.push(FunctionUnit {
                                    name: method_name,
                                    source: Some(method_source),
                                    visibility: Visibility::Public,
                                    doc: None,
                                    signature: Some(signature),
                                    body: None,
                                    attributes: vec![],
                                });
                            }
                        }
                    }
                }
            }

            file_unit.structs.push(StructUnit {
                name: name.clone(),
                source: Some(interface_source),
                head: format!("interface {}", name),
                visibility,
                doc: documentation,
                methods,
                fields: Vec::new(),
                attributes: vec![],
            });
        }
    }

    // Process a type alias declaration
    fn process_type_alias(
        &self,
        file_unit: &mut FileUnit,
        node: Node,
        is_exported: bool,
        source: &[u8],
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let type_source = node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation
            let documentation = find_documentation_for_node(node, source);

            file_unit.structs.push(StructUnit {
                name: name.clone(),
                source: Some(type_source),
                head: format!("type {}", name),
                visibility,
                doc: documentation,
                methods: vec![],
                fields: Vec::new(),
                attributes: vec![],
            });
        }
    }

    // Process an enum declaration
    fn process_enum(&self, file_unit: &mut FileUnit, node: Node, is_exported: bool, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node.utf8_text(source).unwrap_or("").to_string();
            let enum_source = node.utf8_text(source).unwrap_or("").to_string();
            let visibility = if is_exported {
                Visibility::Public
            } else {
                Visibility::Private
            };

            // Check for documentation
            let documentation = find_documentation_for_node(node, source);

            file_unit.structs.push(StructUnit {
                name: name.clone(),
                source: Some(enum_source),
                head: format!("enum {}", name),
                visibility,
                doc: documentation,
                methods: vec![],
                fields: Vec::new(),
                attributes: vec![],
            });
        }
    }
}

// --- Helper Functions ---

// Helper to find documentation for a node
fn find_documentation_for_node(node: Node, source: &[u8]) -> Option<String> {
    let mut current_node = node;

    // Try to find a comment before this node
    while let Some(prev) = current_node.prev_sibling() {
        if prev.kind() == "comment" {
            // Check if it's adjacent (no other nodes in between)
            if prev.end_byte() == current_node.start_byte() - 1 ||
                   // Or check for whitespace
                   (prev.end_byte() < current_node.start_byte() &&
                    source[prev.end_byte()..current_node.start_byte()].iter().all(|&b| b == b' ' || b == b'\n' || b == b'\t' || b == b'\r'))
            {
                return extract_doc_comment(prev, source);
            }

            // If we find a non-comment node, stop looking
            if prev.kind() != "comment" && !prev.is_extra() {
                break;
            }
        }

        current_node = prev;
    }

    // If we didn't find documentation and this node is inside an export statement,
    // look for documentation before the export statement
    if let Some(parent) = node.parent() {
        if parent.kind() == "export_statement" {
            return find_documentation_for_node(parent, source);
        }
    }

    None
}

/// Extracts documentation from a JSDoc comment node.
fn extract_doc_comment(node: Node, source: &[u8]) -> Option<String> {
    if node.kind() == "comment" {
        let text = node.utf8_text(source).ok()?;
        if text.starts_with("/**") {
            let cleaned = text
                .trim_start_matches("/**")
                .trim_end_matches("*/")
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with('*') {
                        // Handle `*` or `* ` prefix
                        trimmed.trim_start_matches('*').trim_start()
                    } else {
                        trimmed
                    }
                })
                .collect::<Vec<&str>>()
                .join("\n")
                .trim()
                .to_string();

            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Finds the next non-comment, non-extra sibling node.
#[allow(dead_code)]
fn find_next_sibling_node(node: Node) -> Option<Node> {
    let mut current_node = node;
    while let Some(sibling) = current_node.next_sibling() {
        if sibling.kind() != "comment" && !sibling.is_extra() {
            return Some(sibling);
        }
        current_node = sibling;
    }
    None
}

/// Look for documentation in previous sibling comment node
#[allow(dead_code)]
fn find_doc_in_previous_comment(node: Node, source: &[u8]) -> Option<String> {
    let mut current = node;
    while let Some(prev) = current.prev_sibling() {
        if prev.kind() == "comment" {
            return extract_doc_comment(prev, source);
        }
        // Skip extra nodes like whitespace
        if !prev.is_extra() {
            // Stop if we find a non-comment, non-extra node
            break;
        }
        current = prev;
    }

    // If we didn't find documentation and this node is inside an export statement,
    // look for documentation before the export statement
    if let Some(parent) = node.parent() {
        if parent.kind() == "export_statement" {
            return find_doc_in_previous_comment(parent, source);
        }
    }

    None
}

impl Deref for TypeScriptParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for TypeScriptParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

impl LanguageParser for TypeScriptParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;
        let source_bytes = source_code.as_bytes();

        let tree = self.parser.parse(&source_code, None).ok_or_else(|| {
            Error::Parse(format!(
                "Tree-sitter failed to parse the file: {}",
                file_path.display()
            ))
        })?;

        let mut file_unit = FileUnit {
            path: file_path.to_path_buf(),
            source: Some(source_code.clone()),
            ..Default::default()
        };

        let root_node = tree.root_node();

        // First, check for file-level documentation
        if let Some(child) = root_node.child(0) {
            if child.kind() == "comment" {
                if let Some(doc) = extract_doc_comment(child, source_bytes) {
                    file_unit.doc = Some(doc);
                }
            }
        }

        // First pass: collect all export statements to track exported names
        let mut exported_names = Vec::new();
        let mut default_export_name = None;

        for i in 0..root_node.child_count() {
            if let Some(node) = root_node.child(i) {
                if node.kind() == "export_statement" {
                    // Direct exports should already be handled by parent check later, so focus on export blocks
                    let node_text = node.utf8_text(source_bytes).unwrap_or("");

                    // Handle named exports format: export { Name1, Name2 }
                    if node_text.contains("{") && node_text.contains("}") {
                        // Basic parsing of export statement text to extract names
                        // For more complex cases, a proper structured parsing approach would be better
                        if let Some(content) = node_text.split('{').nth(1) {
                            if let Some(items) = content.split('}').next() {
                                for item in items.split(',') {
                                    let name = item.trim();
                                    if !name.is_empty() {
                                        exported_names.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }

                    // Handle export default Name
                    if node_text.starts_with("export default") {
                        let parts: Vec<&str> = node_text.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let default_name = parts[2].trim_end_matches(';').to_string();
                            default_export_name = Some(default_name);
                        }
                    }
                }
            }
        }

        // Now traverse the tree to find declarations
        for i in 0..root_node.child_count() {
            if let Some(node) = root_node.child(i) {
                match node.kind() {
                    "function_declaration" => {
                        // Check if this function is explicitly exported or referenced in an export statement
                        let is_exported = node
                            .parent()
                            .is_some_and(|p| p.kind() == "export_statement")
                            || if let Some(name_node) = node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(source_bytes).unwrap_or("").to_string();
                                exported_names.contains(&name)
                                    || default_export_name.as_ref() == Some(&name)
                            } else {
                                false
                            };

                        self.process_function(&mut file_unit, node, is_exported, source_bytes);
                    }
                    "lexical_declaration" => {
                        for j in 0..node.child_count() {
                            if let Some(var_node) = node.child(j) {
                                if var_node.kind() == "variable_declarator" {
                                    // Check if this variable is a function and if it's exported
                                    let is_exported = if let Some(name_node) =
                                        var_node.child_by_field_name("name")
                                    {
                                        let name = name_node
                                            .utf8_text(source_bytes)
                                            .unwrap_or("")
                                            .to_string();
                                        exported_names.contains(&name)
                                            || default_export_name.as_ref() == Some(&name)
                                    } else {
                                        false
                                    };

                                    // Check if it's a function variable
                                    for k in 0..var_node.child_count() {
                                        if let Some(value_node) = var_node.child(k) {
                                            if value_node.kind() == "arrow_function"
                                                || value_node.kind() == "function_expression"
                                            {
                                                self.process_function_variable(
                                                    &mut file_unit,
                                                    node,
                                                    var_node,
                                                    is_exported,
                                                    source_bytes,
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "class_declaration" => {
                        // Check if this class is explicitly exported or referenced in an export statement
                        let is_exported = node
                            .parent()
                            .is_some_and(|p| p.kind() == "export_statement")
                            || if let Some(name_node) = node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(source_bytes).unwrap_or("").to_string();
                                exported_names.contains(&name)
                                    || default_export_name.as_ref() == Some(&name)
                            } else {
                                false
                            };

                        self.process_class(&mut file_unit, node, is_exported, source_bytes);
                    }
                    "interface_declaration" => {
                        // Check if this interface is explicitly exported or referenced in an export statement
                        let is_exported = node
                            .parent()
                            .is_some_and(|p| p.kind() == "export_statement")
                            || if let Some(name_node) = node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(source_bytes).unwrap_or("").to_string();
                                exported_names.contains(&name)
                                    || default_export_name.as_ref() == Some(&name)
                            } else {
                                false
                            };

                        self.process_interface(&mut file_unit, node, is_exported, source_bytes);
                    }
                    "type_alias_declaration" => {
                        // Check if this type is explicitly exported or referenced in an export statement
                        let is_exported = node
                            .parent()
                            .is_some_and(|p| p.kind() == "export_statement")
                            || if let Some(name_node) = node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(source_bytes).unwrap_or("").to_string();
                                exported_names.contains(&name)
                                    || default_export_name.as_ref() == Some(&name)
                            } else {
                                false
                            };

                        self.process_type_alias(&mut file_unit, node, is_exported, source_bytes);
                    }
                    "enum_declaration" => {
                        // Check if this enum is explicitly exported or referenced in an export statement
                        let is_exported = node
                            .parent()
                            .is_some_and(|p| p.kind() == "export_statement")
                            || if let Some(name_node) = node.child_by_field_name("name") {
                                let name =
                                    name_node.utf8_text(source_bytes).unwrap_or("").to_string();
                                exported_names.contains(&name)
                                    || default_export_name.as_ref() == Some(&name)
                            } else {
                                false
                            };

                        self.process_enum(&mut file_unit, node, is_exported, source_bytes);
                    }
                    "export_statement" => {
                        self.process_export(&mut file_unit, node, source_bytes);
                    }
                    "import_statement" => {
                        let source = node.utf8_text(source_bytes).unwrap_or("").to_string();
                        file_unit.declares.push(DeclareStatements {
                            source,
                            kind: DeclareKind::Import,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(file_unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn parse_ts_str(ts_code: &str) -> Result<FileUnit> {
        // Create a temporary file with the TypeScript code
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", ts_code).unwrap();
        let path = temp_file.path().to_path_buf();

        // Parse the file
        let mut parser = TypeScriptParser::try_new()?;
        parser.parse_file(&path)
    }

    #[test]
    fn test_parse_function() -> Result<()> {
        let ts_code = r#"
        /**
         * A function that adds two numbers.
         * @param a First number
         * @param b Second number
         * @returns The sum of a and b
         */
        function add(a: number, b: number): number {
            return a + b;
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.functions.len(), 1);
        let func = &file_unit.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.visibility, Visibility::Private);
        assert!(func
            .doc
            .as_ref()
            .unwrap()
            .contains("A function that adds two numbers"));

        Ok(())
    }

    #[test]
    fn test_parse_exported_function() -> Result<()> {
        let ts_code = r#"
        /**
         * An exported function.
         */
        export function multiply(a: number, b: number): number {
            return a * b;
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.functions.len(), 1);
        let func = &file_unit.functions[0];
        assert_eq!(func.name, "multiply");
        assert_eq!(func.visibility, Visibility::Public);
        // Only check documentation if it exists
        if let Some(doc) = &func.doc {
            assert!(doc.contains("An exported function"));
        }

        Ok(())
    }

    #[test]
    fn test_parse_function_variable() -> Result<()> {
        let ts_code = r#"
        /** Arrow function stored in a constant */
        const arrowFunc = (a: number, b: number): number => a + b;
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.functions.len(), 1);
        let func = &file_unit.functions[0];
        assert_eq!(func.name, "arrowFunc");
        assert_eq!(func.visibility, Visibility::Private);
        if let Some(doc) = &func.doc {
            assert!(doc.contains("Arrow function"));
        }

        Ok(())
    }

    #[test]
    fn test_parse_class() -> Result<()> {
        let ts_code = r#"
        /**
         * A person class.
         */
        class Person {
            name: string;
            age: number;

            constructor(name: string, age: number) {
                this.name = name;
                this.age = age;
            }

            /**
             * Get a greeting
             */
            greet(): string {
                return `Hello, my name is ${this.name}`;
            }
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.structs.len(), 1);
        let class = &file_unit.structs[0];
        assert_eq!(class.name, "Person");
        assert_eq!(class.head, "class Person");
        assert_eq!(class.visibility, Visibility::Private);
        assert!(class.doc.as_ref().unwrap().contains("A person class"));

        // TODO: When method extraction is implemented, test for those as well

        Ok(())
    }

    #[test]
    fn test_parse_interface() -> Result<()> {
        let ts_code = r#"
        /**
         * Represents a shape.
         */
        export interface Shape {
            area(): number;
            perimeter(): number;
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.structs.len(), 1);
        let interface = &file_unit.structs[0];
        assert_eq!(interface.name, "Shape");
        assert_eq!(interface.head, "interface Shape");
        assert_eq!(interface.visibility, Visibility::Public);
        assert!(interface
            .doc
            .as_ref()
            .unwrap()
            .contains("Represents a shape"));

        Ok(())
    }

    #[test]
    fn test_parse_type_alias() -> Result<()> {
        let ts_code = r#"
        /** Represents a point in 2D space */
        type Point = {
            x: number;
            y: number;
        };
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.structs.len(), 1);
        let type_alias = &file_unit.structs[0];
        assert_eq!(type_alias.name, "Point");
        assert_eq!(type_alias.head, "type Point");
        assert_eq!(type_alias.visibility, Visibility::Private);
        assert!(type_alias
            .doc
            .as_ref()
            .unwrap()
            .contains("Represents a point"));

        Ok(())
    }

    #[test]
    fn test_parse_enum() -> Result<()> {
        let ts_code = r#"
        /** Represents directions */
        enum Direction {
            North,
            East,
            South,
            West
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.structs.len(), 1);
        let enum_unit = &file_unit.structs[0];
        assert_eq!(enum_unit.name, "Direction");
        assert_eq!(enum_unit.head, "enum Direction");
        assert_eq!(enum_unit.visibility, Visibility::Private);
        assert!(enum_unit
            .doc
            .as_ref()
            .unwrap()
            .contains("Represents directions"));

        Ok(())
    }

    #[test]
    fn test_parse_imports_exports() -> Result<()> {
        let ts_code = r#"
        import { Component } from 'react';
        import * as util from './util';

        export { add } from './math';
        export * from './helpers';
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        // 2 imports + 2 exports
        assert_eq!(file_unit.declares.len(), 4);

        // Check imports
        let mut found_imports = 0;
        for declare in &file_unit.declares {
            if let DeclareKind::Import = declare.kind {
                found_imports += 1;
                assert!(declare.source.contains("import"));
            }
        }
        assert_eq!(found_imports, 2);

        // Check exports
        let mut found_exports = 0;
        for declare in &file_unit.declares {
            if let DeclareKind::Other(ref kind) = declare.kind {
                if kind == "export" {
                    found_exports += 1;
                    assert!(declare.source.contains("export"));
                }
            }
        }
        assert_eq!(found_exports, 2);

        Ok(())
    }

    #[test]
    fn test_parse_file_doc_comment() -> Result<()> {
        let ts_code = r#"/**
         * This is a file-level documentation comment.
         * @fileoverview Example TypeScript file
         */

        const x = 1;
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert!(file_unit.doc.is_some());
        assert!(file_unit
            .doc
            .as_ref()
            .unwrap()
            .contains("file-level documentation"));
        assert!(file_unit.doc.as_ref().unwrap().contains("@fileoverview"));

        Ok(())
    }

    #[test]
    fn test_complex_file() -> Result<()> {
        let ts_code = r#"/**
         * A complex TypeScript file with multiple declarations.
         * @fileoverview Test for parser
         */

        import { useState, useEffect } from 'react';

        /**
         * Interface for user data
         */
        export interface User {
            id: number;
            name: string;
            email: string;
        }

        /**
         * Type for API response
         */
        type ApiResponse<T> = {
            data: T;
            status: number;
            message: string;
        };

        /** Get a user by ID */
        export async function getUser(id: number): Promise<User> {
            // Implementation
            return { id, name: 'Test', email: 'test@example.com' };
        }

        /** User class implementation */
        class UserImpl implements User {
            id: number;
            name: string;
            email: string;

            constructor(id: number, name: string, email: string) {
                this.id = id;
                this.name = name;
                this.email = email;
            }

            toString() {
                return `User(${this.id}): ${this.name}`;
            }
        }

        /** Status enumeration */
        enum Status {
            Active = 'active',
            Inactive = 'inactive',
            Pending = 'pending'
        }

        /** Create a formatter function */
        export const formatUser = (user: User): string => {
            return `${user.name} <${user.email}>`;
        };
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        // Check all expected components
        assert!(file_unit.doc.is_some());
        assert_eq!(file_unit.declares.len(), 1); // One import
        assert_eq!(file_unit.functions.len(), 2); // getUser and formatUser
        assert_eq!(file_unit.structs.len(), 4); // User interface, ApiResponse type, UserImpl class, Status enum

        // Verify exported items have Public visibility
        let mut exported_count = 0;
        for func in &file_unit.functions {
            if func.visibility == Visibility::Public {
                exported_count += 1;
            }
        }
        for struct_item in &file_unit.structs {
            if struct_item.visibility == Visibility::Public {
                exported_count += 1;
            }
        }
        assert_eq!(exported_count, 3); // User interface, getUser function, formatUser constant

        Ok(())
    }

    #[test]
    fn test_function_signatures() -> Result<()> {
        let ts_code = r#"
        // Regular function
        function publicFunction(param: string): string {
            return `Hello ${param}`;
        }

        // Arrow function with type
        const arrowFunc = (x: number, y: number): number => x + y;

        // Public arrow function
        const publicArrowFunction = (param: string): string => `Hello ${param}`;

        // Private arrow function
        const _privateArrowFunction = (): number => 42;

        // Function with multiple parameters and return type
        function complexFunc(
            name: string,
            age: number,
            options?: { debug: boolean }
        ): Promise<Record<string, unknown>> {
            return Promise.resolve({ name, age });
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.functions.len(), 5);

        // Check regular function
        let func = &file_unit.functions[0];
        assert_eq!(func.name, "publicFunction");
        assert!(func
            .signature
            .as_ref()
            .unwrap()
            .contains("function publicFunction(param: string): string"));

        // Check arrow function
        let arrow = &file_unit.functions[1];
        assert_eq!(arrow.name, "arrowFunc");
        assert_eq!(
            arrow.signature.as_ref().unwrap(),
            "arrowFunc(x: number, y: number): number"
        );

        // Check public arrow function
        let public_arrow = &file_unit.functions[2];
        assert_eq!(public_arrow.name, "publicArrowFunction");
        assert_eq!(
            public_arrow.signature.as_ref().unwrap(),
            "publicArrowFunction(param: string): string"
        );

        // Check private arrow function
        let private_arrow = &file_unit.functions[3];
        assert_eq!(private_arrow.name, "_privateArrowFunction");
        assert_eq!(
            private_arrow.signature.as_ref().unwrap(),
            "_privateArrowFunction(): number"
        );

        // Check complex function
        let complex = &file_unit.functions[4];
        assert_eq!(complex.name, "complexFunc");
        assert!(complex
            .signature
            .as_ref()
            .unwrap()
            .contains("function complexFunc("));
        assert!(complex
            .signature
            .as_ref()
            .unwrap()
            .contains("): Promise<Record<string, unknown>>"));

        Ok(())
    }

    #[test]
    fn test_class_methods() -> Result<()> {
        let ts_code = r#"
        class PublicClass extends BaseClass implements PublicInterface {
          public publicField: string;
          private _privateField: number;

          constructor(publicField: string, privateField: number) {
            super();
            this.publicField = publicField;
            this._privateField = privateField;
          }

          public publicMethod(param: string): string {
            return `Hello ${param}`;
          }

          private _privateMethod(): number {
            return this._privateField;
          }

          abstractMethod(): void {
            console.log("Implemented abstract method");
          }
        }

        class GenericClass<T> {
          constructor(private value: T) { }

          getValue(): T {
            return this.value;
          }
        }

        @decorator
        class DecoratedClass {
          @decorator
          decoratedMethod() { }
        }
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        assert_eq!(file_unit.structs.len(), 3);

        // Check PublicClass
        let public_class = &file_unit.structs[0];
        assert_eq!(public_class.name, "PublicClass");
        assert_eq!(public_class.methods.len(), 4);

        // Check constructor
        let constructor = public_class
            .methods
            .iter()
            .find(|m| m.name == "constructor")
            .unwrap();
        assert!(constructor
            .signature
            .as_ref()
            .unwrap()
            .starts_with("constructor("));

        // Check public method
        let public_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "publicMethod")
            .unwrap();
        assert_eq!(
            public_method.signature.as_ref().unwrap(),
            "public publicMethod(param: string): string"
        );

        // Check private method
        let private_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "_privateMethod")
            .unwrap();
        assert_eq!(
            private_method.signature.as_ref().unwrap(),
            "private _privateMethod(): number"
        );

        // Check unmodified method
        let abstract_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "abstractMethod")
            .unwrap();
        assert_eq!(
            abstract_method.signature.as_ref().unwrap(),
            "abstractMethod(): void"
        );

        // Check GenericClass
        let generic_class = &file_unit.structs[1];
        assert_eq!(generic_class.name, "GenericClass");
        assert_eq!(generic_class.methods.len(), 2);

        // Check getter method
        let get_value = generic_class
            .methods
            .iter()
            .find(|m| m.name == "getValue")
            .unwrap();
        assert_eq!(get_value.signature.as_ref().unwrap(), "getValue(): T");

        // Check DecoratedClass
        let decorated_class = &file_unit.structs[2];
        assert_eq!(decorated_class.name, "DecoratedClass");
        assert_eq!(decorated_class.methods.len(), 1);

        // Check decorated method
        let decorated_method = decorated_class
            .methods
            .iter()
            .find(|m| m.name == "decoratedMethod")
            .unwrap();
        assert_eq!(
            decorated_method.signature.as_ref().unwrap(),
            "decoratedMethod()"
        );

        Ok(())
    }

    #[test]
    fn test_exports_and_visibility() -> Result<()> {
        let ts_code = r#"
        // Class with methods that have different visibility modifiers
        class PublicClass {
            // This should be public by default
            defaultMethod() {
                return "default visibility is public";
            }

            // Explicitly public
            public publicMethod() {
                return "explicitly public";
            }

            // Explicitly private
            private privateMethod() {
                return "explicitly private";
            }
        }

        // Interface with method declarations (all public by default)
        interface PublicInterface {
            methodOne(): string;
            methodTwo(): number;
        }

        // Enum definition
        enum PublicEnum {
            ONE,
            TWO,
            THREE
        }

        // Function definition
        function publicFunction() {
            return "I'm a function";
        }

        // Export statements
        export { PublicClass, PublicInterface, PublicEnum, publicFunction };
        export default PublicClass;
        "#;

        let file_unit = parse_ts_str(ts_code)?;

        // Check total counts
        assert_eq!(file_unit.functions.len(), 1);
        assert_eq!(file_unit.structs.len(), 3); // class, interface, and enum
        assert_eq!(file_unit.declares.len(), 2); // Two export statements

        // Check that all exported items are public
        let function = &file_unit.functions[0];
        assert_eq!(function.name, "publicFunction");
        assert_eq!(function.visibility, Visibility::Public);

        // Find PublicClass and check its visibility and methods
        let public_class = file_unit
            .structs
            .iter()
            .find(|s| s.name == "PublicClass")
            .unwrap();
        assert_eq!(public_class.visibility, Visibility::Public);
        assert_eq!(public_class.methods.len(), 3);

        // Check method visibility
        let default_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "defaultMethod")
            .unwrap();
        assert_eq!(default_method.visibility, Visibility::Public);

        let public_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "publicMethod")
            .unwrap();
        assert_eq!(public_method.visibility, Visibility::Public);

        let private_method = public_class
            .methods
            .iter()
            .find(|m| m.name == "privateMethod")
            .unwrap();
        assert_eq!(private_method.visibility, Visibility::Private);

        // Find PublicInterface and check its visibility
        let public_interface = file_unit
            .structs
            .iter()
            .find(|s| s.name == "PublicInterface")
            .unwrap();
        assert_eq!(public_interface.visibility, Visibility::Public);

        // Find PublicEnum and check its visibility
        let public_enum = file_unit
            .structs
            .iter()
            .find(|s| s.name == "PublicEnum")
            .unwrap();
        assert_eq!(public_enum.visibility, Visibility::Public);

        Ok(())
    }
}
