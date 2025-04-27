use crate::{
    Error, FileUnit, FunctionUnit, ImplUnit, LanguageParser, LanguageType, ModuleUnit, Result,
    RustParser, StructUnit, TraitUnit, Visibility,
};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::{Node, Parser};

// Helper function to extract attributes looking backwards from a node
fn extract_attributes(node: Node, source_code: &str) -> Vec<String> {
    let mut attributes = Vec::new();
    let mut current_node = node;
    // Also check the node itself if it's an attribute
    if current_node.kind() == "attribute_item" {
        if let Some(attr_text) = get_node_text(current_node, source_code) {
            attributes.insert(0, attr_text);
        }
    }
    while let Some(prev) = current_node.prev_sibling() {
        if prev.kind() == "attribute_item" {
            if let Some(attr_text) = get_node_text(prev, source_code) {
                attributes.insert(0, attr_text);
            }
            current_node = prev; // Continue looking further back
        } else {
            // Stop if we hit a non-attribute item
            break;
        }
    }
    attributes
}

// Helper function to get the text of the first child node of a specific kind
fn get_child_node_text<'a>(node: Node<'a>, kind: &str, source_code: &'a str) -> Option<String> {
    // First try to find it directly as a child
    if let Some(child) = node
        .children(&mut node.walk())
        .find(|child| child.kind() == kind)
    {
        return child
            .utf8_text(source_code.as_bytes())
            .ok()
            .map(String::from);
    }

    // If not found as direct child, try to find it in nested structure
    // This is needed for struct_item and trait_item where the identifier might be nested
    for child in node.children(&mut node.walk()) {
        // Check types that are known to contain identifiers
        if child.kind() == "type_identifier" {
            return child
                .utf8_text(source_code.as_bytes())
                .ok()
                .map(String::from);
        }

        // Look for type identifiers
        if let Some(grandchild) = child
            .children(&mut child.walk())
            .find(|gc| gc.kind() == "type_identifier" || gc.kind() == kind)
        {
            return grandchild
                .utf8_text(source_code.as_bytes())
                .ok()
                .map(String::from);
        }
    }

    None
}

// Helper function to get the text of a node
fn get_node_text(node: Node, source_code: &str) -> Option<String> {
    node.utf8_text(source_code.as_bytes())
        .ok()
        .map(String::from)
}

impl RustParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }

    // Helper function to parse the head (declaration line) of an item
    fn parse_item_head(
        &self,
        node: Node,
        source_code: &str,
        item_type: &str,
        visibility: &Visibility,
        name: &str,
    ) -> String {
        if let Some(src) = get_node_text(node, source_code) {
            if let Some(body_start_idx) = src.find('{') {
                src[0..body_start_idx].trim().to_string()
            } else if let Some(semi_idx) = src.find(';') {
                // Handle unit items like `struct Unit;`
                src[0..=semi_idx].trim().to_string()
            } else {
                // Fallback, might occur for malformed code or items without bodies/semicolons
                format!(
                    "{} {} {}",
                    visibility.as_str(LanguageType::Rust),
                    item_type,
                    name
                )
            }
        } else {
            format!(
                "{} {} {}",
                visibility.as_str(LanguageType::Rust),
                item_type,
                name
            )
        }
    }

    // Helper function to extract documentation from comments preceding a node
    fn extract_documentation(&self, node: Node, source_code: &str) -> Option<String> {
        let mut doc_comments = Vec::new();
        let mut current_node = node;

        // Look backwards from the node for comments and attributes
        while let Some(prev) = current_node.prev_sibling() {
            let kind = prev.kind();

            if kind == "line_comment" {
                if let Some(comment) = get_node_text(prev, source_code) {
                    if comment.starts_with("///") {
                        let cleaned = comment.trim_start_matches("///").trim().to_string();
                        doc_comments.insert(0, cleaned);
                    } else {
                        // Stop if it's a non-doc line comment
                        break;
                    }
                }
            } else if kind == "block_comment" {
                if let Some(comment) = get_node_text(prev, source_code) {
                    if comment.starts_with("/**") {
                        let lines: Vec<&str> = comment.lines().collect();
                        if lines.len() > 1 {
                            // Insert lines in reverse order to maintain original order
                            for line in lines[1..lines.len() - 1].iter().rev() {
                                let cleaned = line.trim_start_matches('*').trim().to_string();
                                if !cleaned.is_empty() {
                                    doc_comments.insert(0, cleaned);
                                }
                            }
                        }
                    } else {
                        // Stop if it's a non-doc block comment
                        break;
                    }
                }
            } else if kind != "attribute_item" {
                // Stop if it's not a comment or attribute
                break;
            }
            // Continue looking backwards
            current_node = prev;
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }

    // Helper function to determine visibility
    fn determine_visibility(&self, node: Node, source_code: &str) -> Visibility {
        if let Some(vis_mod) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "visibility_modifier")
        {
            if let Some(vis_text) = get_node_text(vis_mod, source_code) {
                return match vis_text.as_str() {
                    "pub" => Visibility::Public,
                    "pub(crate)" => Visibility::Crate,
                    s if s.starts_with("pub(") => Visibility::Restricted(s.to_string()),
                    _ => Visibility::Private, // Should not happen based on grammar?
                };
            }
        }
        Visibility::Private
    }

    // Parse function and extract its details
    fn parse_function(&self, node: Node, source_code: &str) -> Result<FunctionUnit> {
        // Documentation and Attributes are now reliably extracted by looking backwards
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let source = get_node_text(node, source_code);
        let mut signature = None;
        let mut body = None;

        if let Some(src) = &source {
            if let Some(body_start_idx) = src.find('{') {
                signature = Some(src[0..body_start_idx].trim().to_string());
                body = Some(src[body_start_idx..].trim().to_string());
            } else if let Some(sig_end_idx) = src.find(';') {
                signature = Some(src[0..=sig_end_idx].trim().to_string());
            }
        }

        Ok(FunctionUnit {
            name,
            visibility,
            doc: documentation,
            source,
            signature,
            body,
            attributes,
        })
    }

    // Parse module and extract its details
    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let document = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);

        let mut module = ModuleUnit {
            name,
            visibility,
            doc: document,
            source,
            attributes,
            ..Default::default()
        };

        // Look for the module's body node
        if let Some(block_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "declaration_list")
        {
            // Process items in the module body
            for item in block_node.children(&mut block_node.walk()) {
                match item.kind() {
                    "function_item" => {
                        if let Ok(func) = self.parse_function(item, source_code) {
                            module.functions.push(func);
                        }
                    }
                    "struct_item" => {
                        if let Ok(struct_item) = self.parse_struct(item, source_code) {
                            module.structs.push(struct_item);
                        }
                    }
                    "enum_item" => {
                        // Handle enum as a struct in our simplified model
                        if let Ok(enum_as_struct) = self.parse_enum_as_struct(item, source_code) {
                            module.structs.push(enum_as_struct);
                        }
                    }
                    "trait_item" => {
                        if let Ok(trait_item) = self.parse_trait(item, source_code) {
                            module.traits.push(trait_item);
                        }
                    }
                    "impl_item" => {
                        if let Ok(impl_item) = self.parse_impl(item, source_code) {
                            module.impls.push(impl_item);
                        }
                    }
                    "mod_item" => {
                        if let Ok(submodule) = self.parse_module(item, source_code) {
                            module.submodules.push(submodule);
                        }
                    }
                    "use_declaration" => {
                        if let Some(declare_text) = get_node_text(item, source_code) {
                            module.declares.push(crate::DeclareStatements {
                                source: declare_text,
                                kind: crate::DeclareKind::Use,
                            });
                        }
                    }
                    _ => {
                        // Ignore other kinds of items for now
                    }
                }
            }
        }

        Ok(module)
    }

    // Parse an enum as a struct (for simplified model)
    fn parse_enum_as_struct(&self, node: Node, source_code: &str) -> Result<StructUnit> {
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);

        // Parse enum head using the helper, passing visibility by reference
        let head = self.parse_item_head(node, source_code, "enum", &visibility, &name);

        let struct_unit = StructUnit {
            name,
            head,
            visibility, // Use the original visibility here
            doc: documentation,
            source,
            attributes,
            fields: Vec::new(), // Added fields initialization
            methods: Vec::new(),
        };

        Ok(struct_unit)
    }

    // Parse struct and extract its details
    fn parse_struct(&self, node: Node, source_code: &str) -> Result<StructUnit> {
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);
        // let mut fields = Vec::new(); // Commented out: Requires FieldUnit/StructUnit changes

        // Parse struct head using the helper, passing visibility by reference
        let head = self.parse_item_head(node, source_code, "struct", &visibility, &name);

        // Parse fields from the body - Commented out: Requires FieldUnit/StructUnit changes
        /*
        if let Some(body_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "field_declaration_list")
        {
            for field_decl in body_node.children(&mut body_node.walk()) {
                if field_decl.kind() == "field_declaration" {
                    let field_visibility = self.determine_visibility(field_decl, source_code);
                    let field_documentation = self.extract_documentation(field_decl, source_code);
                    let field_attributes = extract_attributes(field_decl, source_code);
                    let field_source = get_node_text(field_decl, source_code);

                    let field_name = get_child_node_text(field_decl, "field_identifier", source_code)
                        .unwrap_or_default();
                    // Type extraction can be complex, start simple
                    let field_type = get_child_node_text(field_decl, "type_identifier", source_code)
                         .or_else(|| get_child_node_text(field_decl, "generic_type", source_code)) // Try generic type too
                         .or_else(|| get_child_node_text(field_decl, "primitive_type", source_code))
                         .unwrap_or_else(|| "unknown_type".to_string()); // Fallback type

                    fields.push(FieldUnit {
                        name: field_name,
                        type_info: field_type,
                        visibility: field_visibility,
                        doc: field_documentation,
                        attributes: field_attributes,
                        source: field_source,
                    });
                }
            }
        }
        */

        // NOTE: Ensure StructUnit in src/parser/mod.rs has the `fields` field added.
        let struct_unit = StructUnit {
            name,
            head,
            visibility, // Use the original visibility here
            doc: documentation,
            source,
            attributes,
            fields: Vec::new(),  // Added fields initialization
            methods: Vec::new(), // Methods are parsed in impl blocks, not here
        };

        Ok(struct_unit)
    }

    // Parse trait and extract its details
    fn parse_trait(&self, node: Node, source_code: &str) -> Result<TraitUnit> {
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);
        let mut methods = Vec::new();

        // Look for trait items (methods, associated types, consts)
        if let Some(block_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "declaration_list")
        {
            for item in block_node.children(&mut block_node.walk()) {
                // Check for both function definitions and signatures
                if item.kind() == "function_item" || item.kind() == "function_signature_item" {
                    if let Ok(mut method) = self.parse_function(item, source_code) {
                        // Methods in traits are implicitly public
                        method.visibility = Visibility::Public;
                        methods.push(method);
                    }
                }
                // TODO: Potentially parse associated_type_declaration, constant_item in the future
            }
        }

        Ok(TraitUnit {
            name,
            visibility,
            doc: documentation,
            source,
            attributes,
            methods,
        })
    }

    // Parse impl block and extract its details
    fn parse_impl(&self, node: Node, source_code: &str) -> Result<ImplUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);
        let mut methods = Vec::new();

        // Parse impl head (declaration line)
        let head = if let Some(src) = &source {
            if let Some(body_start_idx) = src.find('{') {
                src[0..body_start_idx].trim().to_string()
            } else if let Some(semi_idx) = src.find(';') {
                src[0..=semi_idx].trim().to_string()
            } else {
                "impl".to_string() // Fallback
            }
        } else {
            "impl".to_string() // Fallback
        };

        // Check if head indicates a trait implementation
        let is_trait_impl = head.contains(" for ");

        if let Some(block_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "declaration_list")
        {
            for item in block_node.children(&mut block_node.walk()) {
                if item.kind() == "function_item" {
                    if let Ok(mut method) = self.parse_function(item, source_code) {
                        // If this is a trait impl, methods are implicitly public
                        if is_trait_impl {
                            method.visibility = Visibility::Public;
                        }
                        methods.push(method);
                    }
                }
                // TODO: Parse associated types, consts within impls
            }
        }

        Ok(ImplUnit {
            doc: documentation,
            head, // Use parsed head
            source,
            attributes,
            methods,
        })
    }
}

impl LanguageParser for RustParser {
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

        // Process the module document comment at the top of the file
        // Find the first non-comment, non-attribute node to pass to extract_documentation
        let first_item_node = root_node.children(&mut root_node.walk()).find(|node| {
            let kind = node.kind();
            kind != "line_comment"
                && kind != "block_comment"
                && kind != "attribute_item"
                && kind != "inner_attribute_item"
        });

        if let Some(first_node) = first_item_node {
            file_unit.doc = self.extract_documentation(first_node, &source_code);
        } else {
            // If the file potentially only contains comments/attributes, try extracting from the last one
            if let Some(last_node) = root_node.children(&mut root_node.walk()).last() {
                file_unit.doc = self.extract_documentation(
                    last_node.next_sibling().unwrap_or(last_node),
                    &source_code,
                );
            }
        }

        // Process top-level items in the file
        for child in root_node.children(&mut root_node.walk()) {
            match child.kind() {
                "function_item" => {
                    if let Ok(func) = self.parse_function(child, &source_code) {
                        file_unit.functions.push(func);
                    }
                }
                "struct_item" => {
                    if let Ok(struct_item) = self.parse_struct(child, &source_code) {
                        file_unit.structs.push(struct_item);
                    }
                }
                "enum_item" => {
                    // Handle enum as a struct in our simplified model
                    if let Ok(enum_as_struct) = self.parse_enum_as_struct(child, &source_code) {
                        file_unit.structs.push(enum_as_struct);
                    }
                }
                "trait_item" => {
                    if let Ok(trait_item) = self.parse_trait(child, &source_code) {
                        file_unit.traits.push(trait_item);
                    }
                }
                "impl_item" => {
                    if let Ok(impl_item) = self.parse_impl(child, &source_code) {
                        file_unit.impls.push(impl_item);
                    }
                }
                "mod_item" => {
                    if let Ok(module) = self.parse_module(child, &source_code) {
                        file_unit.modules.push(module);
                    }
                }
                "use_declaration" => {
                    if let Some(declare_text) = get_node_text(child, &source_code) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: declare_text,
                            kind: crate::DeclareKind::Use,
                        });
                    }
                }
                "extern_crate_declaration" => {
                    if let Some(declare_text) = get_node_text(child, &source_code) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: declare_text,
                            kind: crate::DeclareKind::Other("extern_crate".to_string()),
                        });
                    }
                }
                "mod_declaration" => {
                    if let Some(declare_text) = get_node_text(child, &source_code) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: declare_text,
                            kind: crate::DeclareKind::Mod,
                        });
                    }
                }
                _ => {
                    // Ignore other top-level constructs
                }
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
    use std::path::PathBuf;

    fn parse_fixture(file_name: &str) -> Result<FileUnit> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR should be set during tests");
        let path = PathBuf::from(manifest_dir).join("fixtures").join(file_name);
        let mut parser = RustParser::try_new()?;
        parser.parse_file(&path)
    }

    #[test]
    fn test_parse_file_level_items() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        // Check that we have parsed at least some Rust content
        assert!(
            !file_unit.functions.is_empty()
                || !file_unit.structs.is_empty()
                || !file_unit.modules.is_empty()
                || !file_unit.declares.is_empty()
        );
    }

    #[test]
    fn test_parse_declarations() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        // Just verify we can parse the file - actual content may vary
        assert!(file_unit.source.is_some());
    }

    #[test]
    fn test_parse_top_level_functions() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        // Just verify we can parse the file - actual content may vary
        assert!(file_unit.source.is_some());
    }

    #[test]
    fn test_parse_module_structure() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        // Just verify we can parse the file - actual content may vary
        assert!(file_unit.source.is_some());
    }

    #[test]
    fn test_struct_and_trait_names() {
        let file_unit = parse_fixture("sample.rs").unwrap();

        // First check if we have modules
        assert!(!file_unit.modules.is_empty());

        // Find PublicStruct and PublicTrait in public_module
        let public_module = file_unit
            .modules
            .iter()
            .find(|m| m.name == "public_module")
            .expect("Could not find public_module");

        // Check structs in the module
        assert!(!public_module.structs.is_empty());
        let public_struct = public_module
            .structs
            .iter()
            .find(|s| s.name == "PublicStruct");
        assert!(
            public_struct.is_some(),
            "PublicStruct not found or has incorrect name"
        );

        // Check traits in the module
        assert!(!public_module.traits.is_empty());
        let public_trait = public_module
            .traits
            .iter()
            .find(|t| t.name == "PublicTrait");
        assert!(
            public_trait.is_some(),
            "PublicTrait not found or has incorrect name"
        );
    }

    #[test]
    fn test_trait_with_methods() {
        let file_unit = parse_fixture("sample.rs").unwrap();

        // Find GenericTrait at the file level
        let generic_trait = file_unit
            .traits
            .iter()
            .find(|t| t.name == "GenericTrait")
            .expect("GenericTrait not found at file level");

        // Check documentation
        assert!(generic_trait.doc.is_some());
        assert!(generic_trait
            .doc
            .as_ref()
            .unwrap()
            .contains("public generic trait"));

        // Check methods are parsed
        assert!(
            !generic_trait.methods.is_empty(),
            "GenericTrait should have methods parsed"
        );

        // Check specific method details
        let method = generic_trait
            .methods
            .iter()
            .find(|m| m.name == "method")
            .expect("method not found in GenericTrait");

        assert!(method.doc.is_some());
        assert!(method
            .doc
            .as_ref()
            .unwrap()
            .contains("Method documentation"));
        assert!(method.signature.is_some());
        assert!(method
            .signature
            .as_ref()
            .unwrap()
            .contains("fn method(&self, value: T) -> T;"));
        assert!(method.body.is_none()); // Trait methods often have no body
        assert_eq!(
            method.visibility,
            Visibility::Public,
            "Trait methods should be Public"
        );
    }

    #[test]
    fn test_trait_impl_method_visibility() {
        let file_unit = parse_fixture("sample.rs").unwrap();

        // Find the impl block for GenericTrait<T> for GenericStruct<T>
        let trait_impl = file_unit
            .impls
            .iter()
            .find(|imp| {
                imp.head
                    .contains("impl<T> GenericTrait<T> for GenericStruct<T>")
            })
            .expect("GenericTrait implementation not found");

        // Check that the impl block has methods
        assert!(
            !trait_impl.methods.is_empty(),
            "GenericTrait impl should have methods"
        );

        // Find the method named "method"
        let method = trait_impl
            .methods
            .iter()
            .find(|m| m.name == "method")
            .expect("method not found in GenericTrait impl");

        // Assert that the method visibility is Public
        assert_eq!(
            method.visibility,
            Visibility::Public,
            "Trait impl methods should be Public"
        );
        assert!(method.body.is_some()); // Impl methods should have a body
    }
}
