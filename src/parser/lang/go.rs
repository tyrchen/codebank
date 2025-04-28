use super::GoParser;
use crate::{
    DeclareKind, DeclareStatements, Error, FieldUnit, FileUnit, FunctionUnit, ImplUnit,
    LanguageParser, ModuleUnit, Result, StructUnit, TraitUnit, Visibility,
};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::{Node, Parser};

impl LanguageParser for GoParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        // Read the file
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;

        // Parse the file
        let tree = self
            .parse(source_code.as_bytes(), None)
            .ok_or_else(|| Error::TreeSitter("Failed to parse source code".to_string()))?;
        let root_node = tree.root_node();

        // Create a new file unit
        let mut file_unit = FileUnit::new(file_path.to_path_buf());
        file_unit.source = Some(source_code.clone());

        // Maps to collect methods by receiver type
        let mut methods_by_type: std::collections::HashMap<String, Vec<FunctionUnit>> =
            std::collections::HashMap::new();

        // Process top-level declarations
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            match child.kind() {
                "package_clause" => {
                    let package_doc = extract_documentation(child, &source_code);
                    if let Some(package_name) =
                        get_child_node_text(child, "package_identifier", &source_code)
                    {
                        let module = ModuleUnit {
                            name: package_name,
                            visibility: Visibility::Public, // Packages are public
                            doc: package_doc,
                            source: get_node_text(child, &source_code),
                            attributes: Vec::new(),
                            ..Default::default()
                        };
                        file_unit.modules.push(module);
                    }
                }
                "import_declaration" => {
                    // Handle single and block imports
                    let mut import_cursor = child.walk();
                    for import_spec in child.children(&mut import_cursor) {
                        if import_spec.kind() == "import_spec"
                            || import_spec.kind() == "interpreted_string_literal"
                            || import_spec.kind() == "raw_string_literal"
                        {
                            if let Some(import_text) = get_node_text(import_spec, &source_code) {
                                file_unit.declares.push(DeclareStatements {
                                    source: import_text,
                                    kind: DeclareKind::Use,
                                });
                            }
                        } else if import_spec.kind() == "import_spec_list" {
                            let mut list_cursor = import_spec.walk();
                            for inner_spec in import_spec.children(&mut list_cursor) {
                                if inner_spec.kind() == "import_spec" {
                                    if let Some(import_text) =
                                        get_node_text(inner_spec, &source_code)
                                    {
                                        file_unit.declares.push(DeclareStatements {
                                            source: import_text,
                                            kind: DeclareKind::Use,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                "function_declaration" => {
                    if let Ok(func) = self.parse_function(child, &source_code) {
                        file_unit.functions.push(func);
                    }
                }
                "method_declaration" => {
                    if let Ok((receiver_type, method)) = self.parse_method(child, &source_code) {
                        methods_by_type
                            .entry(receiver_type)
                            .or_default()
                            .push(method);
                    }
                }
                "type_declaration" => {
                    let mut type_decl_cursor = child.walk();
                    for type_spec_node in child.children(&mut type_decl_cursor) {
                        if type_spec_node.kind() == "type_spec" {
                            let mut type_spec_cursor = type_spec_node.walk();
                            if let Some(type_def_node) = type_spec_node
                                .children(&mut type_spec_cursor)
                                .find(|n| n.kind() == "struct_type" || n.kind() == "interface_type")
                            {
                                if type_def_node.kind() == "struct_type" {
                                    if let Ok(struct_item) =
                                        self.parse_struct(type_spec_node, &source_code)
                                    {
                                        file_unit.structs.push(struct_item);
                                    }
                                } else if type_def_node.kind() == "interface_type" {
                                    if let Ok(interface_item) =
                                        self.parse_interface(type_spec_node, &source_code)
                                    {
                                        file_unit.traits.push(interface_item);
                                    }
                                }
                            }
                        }
                    }
                }
                "const_declaration" | "var_declaration" => {
                    let mut decl_cursor = child.walk();
                    for spec_node in child.children(&mut decl_cursor) {
                        if spec_node.kind() == "const_spec" || spec_node.kind() == "var_spec" {
                            if let Some(declare_text) = get_node_text(spec_node, &source_code) {
                                let kind_str = if child.kind() == "const_declaration" {
                                    "const"
                                } else {
                                    "var"
                                };
                                file_unit.declares.push(DeclareStatements {
                                    source: declare_text,
                                    kind: DeclareKind::Other(kind_str.to_string()),
                                });
                            }
                        } else if spec_node.kind() == "var_spec_list"
                            || spec_node.kind() == "const_spec_list"
                        {
                            let mut list_cursor = spec_node.walk();
                            for inner_spec_node in spec_node.children(&mut list_cursor) {
                                if inner_spec_node.kind() == "const_spec"
                                    || inner_spec_node.kind() == "var_spec"
                                {
                                    if let Some(declare_text) =
                                        get_node_text(inner_spec_node, &source_code)
                                    {
                                        let kind_str = if child.kind() == "const_declaration" {
                                            "const"
                                        } else {
                                            "var"
                                        };
                                        file_unit.declares.push(DeclareStatements {
                                            source: declare_text,
                                            kind: DeclareKind::Other(kind_str.to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                "comment" => {
                    // Ignore comments at top level for now
                }
                _ => {
                    // Ignore other top-level node: {}
                }
            }
        }

        // Add methods to their respective structs
        for struct_item in &mut file_unit.structs {
            if let Some(methods) = methods_by_type.remove(&struct_item.name) {
                struct_item.methods.extend(methods.clone()); // Add methods to struct

                // Also create an ImplUnit for each struct with methods
                let impl_unit = ImplUnit {
                    doc: None, // Could try to find doc for the impl block if needed
                    head: format!("methods for {}", struct_item.name),
                    source: None, // Source for the whole impl block is tricky
                    attributes: Vec::new(),
                    methods, // Moves methods into the impl unit
                };
                file_unit.impls.push(impl_unit);
            }
        }

        // For any methods whose receiver types weren't found as structs,
        // still create impl units (e.g., methods on built-in types or type aliases)
        for (receiver_type, methods) in methods_by_type {
            let impl_unit = ImplUnit {
                doc: None,
                head: format!("methods for {}", receiver_type),
                source: None,
                attributes: Vec::new(),
                methods,
            };
            file_unit.impls.push(impl_unit);
        }

        Ok(file_unit)
    }
}

impl GoParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_go::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }

    // Helper function to determine visibility (in Go, uppercase first letter means exported/public)
    fn determine_visibility(&self, name: &str) -> Visibility {
        if !name.is_empty() && name.chars().next().unwrap().is_uppercase() {
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    // Parse function and extract its details
    fn parse_function(&self, node: Node, source_code: &str) -> Result<FunctionUnit> {
        let documentation = extract_documentation(node, source_code);
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());

        let visibility = self.determine_visibility(&name);
        let source = get_node_text(node, source_code);
        let mut signature = None;
        let mut body = None;

        // Extract signature (everything before the body block)
        if let Some(body_node) = node.child_by_field_name("body") {
            let sig_end = body_node.start_byte();
            let sig_start = node.start_byte();
            if sig_end > sig_start {
                signature = Some(source_code[sig_start..sig_end].trim().to_string());
            }
            body = get_node_text(body_node, source_code);
        } else {
            // Fallback for function declarations without body (e.g. in interfaces - though handled separately)
            signature = source.clone();
        }

        Ok(FunctionUnit {
            name,
            visibility,
            doc: documentation,
            source,
            signature,
            body,
            attributes: Vec::new(), // Go doesn't have attributes like Rust
        })
    }

    // Parse struct and extract its details
    // Node passed here should be the `type_spec` node
    fn parse_struct(&self, type_spec_node: Node, source_code: &str) -> Result<StructUnit> {
        // Documentation should be associated with the type_spec node or its parent type_declaration
        let documentation =
            extract_documentation(type_spec_node, source_code).or_else(|| -> Option<String> {
                type_spec_node
                    .parent()
                    .and_then(|p| extract_documentation(p, source_code))
            });
        let name = get_child_node_text(type_spec_node, "type_identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(&name);
        let source = get_node_text(
            type_spec_node.parent().unwrap_or(type_spec_node),
            source_code,
        );
        let head = format!("type {} struct", name);

        let mut fields = Vec::new();

        if let Some(struct_type) = type_spec_node
            .children(&mut type_spec_node.walk())
            .find(|child| child.kind() == "struct_type")
        {
            if let Some(field_list) = struct_type
                .children(&mut struct_type.walk())
                .find(|child| child.kind() == "field_declaration_list")
            {
                let mut list_cursor = field_list.walk();
                for field_decl in field_list.children(&mut list_cursor) {
                    if field_decl.kind() == "field_declaration" {
                        let field_documentation = extract_documentation(field_decl, source_code);
                        let field_source = get_node_text(field_decl, source_code);
                        let mut field_names = Vec::new();
                        let mut decl_cursor = field_decl.walk();
                        for child in field_decl.children(&mut decl_cursor) {
                            if child.kind() == "identifier" || child.kind() == "field_identifier" {
                                if let Some(field_name) = get_node_text(child, source_code) {
                                    field_names.push(field_name);
                                }
                            } else if child.kind().ends_with("_type")
                                || child.kind() == "qualified_type"
                            {
                                // Stop collecting names when type is reached
                                break;
                            }
                        }
                        for field_name in field_names {
                            fields.push(FieldUnit {
                                name: field_name,
                                doc: field_documentation.clone(),
                                attributes: Vec::new(),
                                source: field_source.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(StructUnit {
            name,
            head,
            visibility,
            doc: documentation,
            source,
            attributes: Vec::new(),
            fields,
            methods: Vec::new(),
        })
    }

    // Parse interface (similar to trait in Rust)
    // Node passed here should be the `type_spec` node
    fn parse_interface(&self, type_spec_node: Node, source_code: &str) -> Result<TraitUnit> {
        // Documentation should be associated with the type_spec node or its parent type_declaration
        let documentation =
            extract_documentation(type_spec_node, source_code).or_else(|| -> Option<String> {
                type_spec_node
                    .parent()
                    .and_then(|p| extract_documentation(p, source_code))
            });
        let name = get_child_node_text(type_spec_node, "type_identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(&name);
        let source = get_node_text(
            type_spec_node.parent().unwrap_or(type_spec_node),
            source_code,
        );

        let mut methods = Vec::new();

        if let Some(interface_type) = type_spec_node
            .children(&mut type_spec_node.walk())
            .find(|child| child.kind() == "interface_type")
        {
            let mut interface_cursor = interface_type.walk();
            for child in interface_type.children(&mut interface_cursor) {
                if child.kind() == "method_elem" {
                    let method_spec = child; // Keep variable name for consistency
                    let method_doc = extract_documentation(method_spec, source_code);
                    let method_source = get_node_text(method_spec, source_code);
                    // Method name is typically the first identifier within method_spec
                    let method_name = get_child_node_text(method_spec, "identifier", source_code)
                        .or_else(|| {
                            get_child_node_text(method_spec, "field_identifier", source_code)
                        })
                        .unwrap_or_else(|| "unknown_interface_method".to_string());
                    let visibility = self.determine_visibility(&method_name); // Interface methods are implicitly public
                    // Interface methods only have signatures, no bodies
                    let signature = method_source.clone();

                    methods.push(FunctionUnit {
                        name: method_name,
                        visibility, // Could force Public, but determine_visibility works
                        doc: method_doc,
                        source: method_source,
                        signature,
                        body: None, // Interface methods don't have bodies
                        attributes: Vec::new(),
                    });
                }
            }
        }

        Ok(TraitUnit {
            name,
            visibility,
            doc: documentation,
            source,
            attributes: Vec::new(),
            methods,
        })
    }

    // Parse method (like impl in Rust)
    // Node is `method_declaration`
    fn parse_method(&self, node: Node, source_code: &str) -> Result<(String, FunctionUnit)> {
        let documentation = extract_documentation(node, source_code);
        let source = get_node_text(node, source_code);

        // Get method name (field identifier)
        let method_name = get_child_node_text(node, "field_identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());

        // Get receiver type (struct type)
        let receiver_type = if let Some(parameter_list) = node.child_by_field_name("receiver") {
            // The receiver is a parameter_list containing one parameter_declaration
            if let Some(parameter) = parameter_list
                .children(&mut parameter_list.walk())
                .find(|child| child.kind() == "parameter_declaration")
            {
                // Extract type from parameter declaration
                if let Some(type_node) = parameter.child_by_field_name("type") {
                    get_node_text(type_node, source_code)
                        .map(|s| s.trim_start_matches('*').to_string()) // Remove leading * for pointer receivers
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        let visibility = self.determine_visibility(&method_name);
        let mut signature = None;
        let mut body = None;

        // Extract signature (everything before the body block)
        if let Some(body_node) = node.child_by_field_name("body") {
            let sig_end = body_node.start_byte();
            let sig_start = node.start_byte();
            if sig_end > sig_start {
                signature = Some(source_code[sig_start..sig_end].trim().to_string());
            }
            body = get_node_text(body_node, source_code);
        } else {
            signature = source.clone();
        }

        let function = FunctionUnit {
            name: method_name,
            visibility,
            doc: documentation,
            source,
            signature,
            body,
            attributes: Vec::new(),
        };

        Ok((receiver_type, function))
    }
}

// Helper function to get the text of a node
fn get_node_text(node: Node, source_code: &str) -> Option<String> {
    node.utf8_text(source_code.as_bytes())
        .ok()
        .map(String::from)
}

// Helper function to get the text of the first child node of a specific kind
fn get_child_node_text<'a>(node: Node<'a>, kind: &str, source_code: &'a str) -> Option<String> {
    // First try to find it directly as a child using field name if common (e.g., 'name')
    if kind == "identifier" || kind == "package_identifier" || kind == "field_identifier" {
        if let Some(name_node) = node.child_by_field_name("name") {
            // Check if the node kind matches the expected identifier type
            if name_node.kind() == kind {
                return name_node
                    .utf8_text(source_code.as_bytes())
                    .ok()
                    .map(String::from);
            }
        }
    }

    // Then try finding by specific node kind
    if let Some(child) = node
        .children(&mut node.walk())
        .find(|child| child.kind() == kind)
    {
        return child
            .utf8_text(source_code.as_bytes())
            .ok()
            .map(String::from);
    }

    // Fallback: Look for any specific identifier kind child if specific kind not found
    if kind == "identifier" || kind == "package_identifier" || kind == "field_identifier" {
        if let Some(ident_child) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == kind)
        {
            return ident_child
                .utf8_text(source_code.as_bytes())
                .ok()
                .map(String::from);
        }
    }
    // Generic identifier fallback
    if let Some(ident_child) = node
        .children(&mut node.walk())
        .find(|child| child.kind() == "identifier")
    {
        return ident_child
            .utf8_text(source_code.as_bytes())
            .ok()
            .map(String::from);
    }

    None
}

// Extract documentation from comments preceding a node
fn extract_documentation(node: Node, source_code: &str) -> Option<String> {
    // Attempt to find a preceding comment block associated with the node.
    // Go documentation comments are typically immediately before the declaration.
    let mut prev_sibling = node.prev_sibling();
    while let Some(sibling) = prev_sibling {
        if sibling.kind() == "comment" {
            // Check if the comment is "close" enough (on the preceding line(s))
            if node.start_position().row == sibling.end_position().row + 1
                || node.start_position().row == sibling.start_position().row + 1
            {
                // Found a relevant comment block
                let doc_text = get_node_text(sibling, source_code)?; // Use ? to propagate None
                // Basic cleaning: remove comment markers and trim whitespace
                let cleaned_doc = doc_text
                    .trim_start_matches("//")
                    .trim_start_matches("/*")
                    .trim_end_matches("*/")
                    .trim()
                    .to_string();
                // If multiple comment lines form a block, they should be concatenated.
                // Tree-sitter often gives the whole block as one node.
                // If not, more complex logic might be needed to combine multi-line comments.
                return Some(cleaned_doc);
            } else {
                // Comment is not immediately preceding, stop searching backwards
                break;
            }
        } else if !sibling.is_extra() {
            // Reached a non-comment, non-whitespace node, stop searching
            break;
        }
        prev_sibling = sibling.prev_sibling();
    }

    None // No documentation comment found immediately preceding the node
}

impl Deref for GoParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for GoParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parse_fixture(file_name: &str) -> Result<FileUnit> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR should be set during tests");
        let path = PathBuf::from(manifest_dir).join("fixtures").join(file_name);
        let mut parser = GoParser::try_new()?;
        parser.parse_file(&path)
    }

    #[test]
    fn test_parse_go_package() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");
        assert_eq!(
            file_unit.modules.len(),
            1,
            "Should parse one package module"
        );
        assert_eq!(file_unit.modules[0].name, "example");
        assert!(
            file_unit.modules[0].doc.is_some(),
            "Package doc comment missing"
        );
        assert!(
            file_unit.modules[0]
                .doc
                .as_ref()
                .unwrap()
                .contains("sample Go file")
        );
    }

    #[test]
    fn test_parse_go_imports() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");
        // Count only imports
        let import_count = file_unit
            .declares
            .iter()
            .filter(|d| d.kind == DeclareKind::Use)
            .count();
        assert_eq!(
            import_count, 7,
            "Expected exactly 7 imports, found {}",
            import_count
        ); // Check exact count
        // Check specific imports
        assert!(
            file_unit
                .declares
                .iter()
                .any(|d| d.kind == DeclareKind::Use && d.source.contains("\"fmt\""))
        );
        assert!(
            file_unit
                .declares
                .iter()
                .any(|d| d.kind == DeclareKind::Use && d.source.contains("\"strings\""))
        );
        assert!(
            file_unit
                .declares
                .iter()
                .any(|d| d.kind == DeclareKind::Use && d.source.contains("\"os\""))
        );
        // Check const/var declarations
        let const_count = file_unit
            .declares
            .iter()
            .filter(|d| matches!(&d.kind, DeclareKind::Other(s) if s == "const"))
            .count();
        assert!(
            const_count >= 3,
            "Expected at least 3 const declarations, found {}",
            const_count
        );
        let var_count = file_unit
            .declares
            .iter()
            .filter(|d| matches!(&d.kind, DeclareKind::Other(s) if s == "var"))
            .count();
        assert!(
            var_count >= 1,
            "Expected at least 1 var declaration, found {}",
            var_count
        );
    }

    #[test]
    fn test_parse_go_functions() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");
        // Check top-level functions
        let new_person_func = file_unit.functions.iter().find(|f| f.name == "NewPerson");
        assert!(new_person_func.is_some(), "NewPerson function not found");
        let new_person_func = new_person_func.unwrap();
        assert_eq!(new_person_func.visibility, Visibility::Public);
        assert!(new_person_func.doc.is_some(), "NewPerson doc missing");
        assert!(
            new_person_func
                .doc
                .as_ref()
                .unwrap()
                .contains("creates a new Person instance")
        );
        assert!(new_person_func.signature.is_some());
        assert!(new_person_func.body.is_some());

        let upper_case_func = file_unit.functions.iter().find(|f| f.name == "UpperCase");
        assert!(upper_case_func.is_some(), "UpperCase function not found");
        let upper_case_func = upper_case_func.unwrap();
        assert_eq!(upper_case_func.visibility, Visibility::Public);
        assert!(upper_case_func.doc.is_some(), "UpperCase doc missing");
        assert!(
            upper_case_func
                .doc
                .as_ref()
                .unwrap()
                .contains("converts a string to uppercase")
        );
        assert!(upper_case_func.signature.is_some());
        assert!(upper_case_func.body.is_some());
    }

    #[test]
    fn test_parse_go_structs() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");

        let person_struct = file_unit.structs.iter().find(|s| s.name == "Person");
        assert!(person_struct.is_some(), "Person struct not found");
        let person_struct = person_struct.unwrap();
        assert_eq!(person_struct.visibility, Visibility::Public);
        assert!(person_struct.doc.is_some(), "Person doc missing");
        assert!(
            person_struct
                .doc
                .as_ref()
                .unwrap()
                .contains("represents a person")
        );
        assert_eq!(person_struct.fields.len(), 3, "Person should have 3 fields");
        // Check field names
        assert!(person_struct.fields.iter().any(|f| f.name == "Name"));
        assert!(person_struct.fields.iter().any(|f| f.name == "Age"));
        assert!(person_struct.fields.iter().any(|f| f.name == "address"));
        // Check field documentation
        let name_field = person_struct
            .fields
            .iter()
            .find(|f| f.name == "Name")
            .unwrap();
        assert!(name_field.doc.is_some(), "Name field doc missing");
        assert!(name_field.doc.as_ref().unwrap().contains("person's name"));

        let age_field = person_struct
            .fields
            .iter()
            .find(|f| f.name == "Age")
            .unwrap();
        assert!(age_field.doc.is_some(), "Age field doc missing");
        assert!(age_field.doc.as_ref().unwrap().contains("person's age"));

        let address_field = person_struct
            .fields
            .iter()
            .find(|f| f.name == "address")
            .unwrap();
        assert!(address_field.doc.is_some(), "address field doc missing");
        assert!(
            address_field
                .doc
                .as_ref()
                .unwrap()
                .contains("unexported field")
        );

        let greeter_impl_struct = file_unit.structs.iter().find(|s| s.name == "GreeterImpl");
        assert!(
            greeter_impl_struct.is_some(),
            "GreeterImpl struct not found"
        );
        let greeter_impl_struct = greeter_impl_struct.unwrap();
        assert_eq!(greeter_impl_struct.visibility, Visibility::Public);
        assert!(greeter_impl_struct.doc.is_some(), "GreeterImpl doc missing");
        assert!(
            greeter_impl_struct
                .doc
                .as_ref()
                .unwrap()
                .contains("implements the Greeter interface")
        );
        assert_eq!(
            greeter_impl_struct.fields.len(),
            1,
            "GreeterImpl should have 1 field"
        );
        assert_eq!(greeter_impl_struct.fields[0].name, "greeting");

        // Check associated methods (parsed into impls)
        let greeter_impl_methods = file_unit
            .impls
            .iter()
            .find(|imp| imp.head == "methods for GreeterImpl");
        assert!(
            greeter_impl_methods.is_some(),
            "Impl block for GreeterImpl not found"
        );
        assert_eq!(
            greeter_impl_methods.unwrap().methods.len(),
            1,
            "GreeterImpl should have 1 method"
        );
        assert_eq!(greeter_impl_methods.unwrap().methods[0].name, "Greet");
    }

    #[test]
    fn test_parse_go_interfaces() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");

        let greeter_interface = file_unit.traits.iter().find(|t| t.name == "Greeter");
        assert!(greeter_interface.is_some(), "Greeter interface not found");
        let greeter_interface = greeter_interface.unwrap();
        assert_eq!(greeter_interface.visibility, Visibility::Public);
        assert!(greeter_interface.doc.is_some(), "Greeter doc missing");
        assert!(
            greeter_interface
                .doc
                .as_ref()
                .unwrap()
                .contains("defines an interface")
        );
        assert_eq!(
            greeter_interface.methods.len(),
            1,
            "Greeter interface should have 1 method"
        );
        assert_eq!(greeter_interface.methods[0].name, "Greet");
        assert!(
            greeter_interface.methods[0].doc.is_some(),
            "Greet method doc missing"
        );
        assert!(
            greeter_interface.methods[0]
                .doc
                .as_ref()
                .unwrap()
                .contains("returns a greeting message")
        );
        assert!(greeter_interface.methods[0].signature.is_some());
        assert!(greeter_interface.methods[0].body.is_none());
    }

    #[test]
    fn test_parse_go_methods() {
        let file_unit = parse_fixture("sample.go").expect("Failed to parse Go file");

        // Find the ImplUnit for Person methods
        let person_impl = file_unit
            .impls
            .iter()
            .find(|imp| imp.head == "methods for Person");
        assert!(person_impl.is_some(), "Impl block for Person not found");
        let person_impl = person_impl.unwrap();

        // Check method count
        assert_eq!(person_impl.methods.len(), 3, "Person should have 3 methods");

        // Check SetAddress method
        let set_address = person_impl.methods.iter().find(|m| m.name == "SetAddress");
        assert!(set_address.is_some(), "SetAddress method not found");
        let set_address = set_address.unwrap();
        assert_eq!(set_address.visibility, Visibility::Public);
        assert!(set_address.doc.is_some(), "SetAddress doc missing");
        assert!(
            set_address
                .doc
                .as_ref()
                .unwrap()
                .contains("sets the person's address")
        );
        assert!(set_address.signature.is_some());
        assert!(set_address.body.is_some());

        // Check GetAddress method
        let get_address = person_impl.methods.iter().find(|m| m.name == "GetAddress");
        assert!(get_address.is_some(), "GetAddress method not found");
        let get_address = get_address.unwrap();
        assert_eq!(get_address.visibility, Visibility::Public);
        assert!(get_address.doc.is_some(), "GetAddress doc missing");
        assert!(
            get_address
                .doc
                .as_ref()
                .unwrap()
                .contains("returns the person's address")
        );
        assert!(get_address.signature.is_some());
        assert!(get_address.body.is_some());

        // Check String method
        let string_method = person_impl.methods.iter().find(|m| m.name == "String");
        assert!(string_method.is_some(), "String method not found");
        let string_method = string_method.unwrap();
        assert_eq!(string_method.visibility, Visibility::Public);
        assert!(string_method.doc.is_some(), "String method doc missing");
        assert!(
            string_method
                .doc
                .as_ref()
                .unwrap()
                .contains("implements the Stringer interface")
        );
        assert!(string_method.signature.is_some());
        assert!(string_method.body.is_some());
    }
}
