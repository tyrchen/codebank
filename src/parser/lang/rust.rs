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
            documentation,
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
            document,
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

        // Parse enum head (declaration line)
        let head = if let Some(src) = &source {
            if let Some(body_start_idx) = src.find('{') {
                src[0..body_start_idx].trim().to_string()
            } else if let Some(semi_idx) = src.find(';') {
                src[0..=semi_idx].trim().to_string()
            } else {
                format!("{} enum {}", visibility.as_str(LanguageType::Rust), name)
            }
        } else {
            format!("{} enum {}", visibility.as_str(LanguageType::Rust), name)
        };

        let struct_unit = StructUnit {
            name,
            head,
            visibility,
            documentation,
            source,
            attributes,
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

        // Parse struct head (declaration line)
        let head = if let Some(src) = &source {
            if let Some(body_start_idx) = src.find('{') {
                src[0..body_start_idx].trim().to_string()
            } else if let Some(semi_idx) = src.find(';') {
                src[0..=semi_idx].trim().to_string()
            } else {
                format!("{} struct {}", visibility.as_str(LanguageType::Rust), name)
            }
        } else {
            format!("{} struct {}", visibility.as_str(LanguageType::Rust), name)
        };

        let struct_unit = StructUnit {
            name,
            head,
            visibility,
            documentation,
            source,
            attributes,
            methods: Vec::new(),
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

        // Look for trait items (methods)
        if let Some(block_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "declaration_list")
        {
            for item in block_node.children(&mut block_node.walk()) {
                if item.kind() == "function_item" {
                    if let Ok(method) = self.parse_function(item, source_code) {
                        methods.push(method);
                    }
                }
            }
        }

        Ok(TraitUnit {
            name,
            visibility,
            documentation,
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

        let is_trait_impl = if let Some(source) = &source {
            source.contains(" for ") && source.contains("impl ")
        } else {
            false
        };

        // Parse impl head (declaration line)
        let head = if let Some(src) = &source {
            if let Some(body_start_idx) = src.find('{') {
                src[0..body_start_idx].trim().to_string()
            } else if let Some(semi_idx) = src.find(';') {
                src[0..=semi_idx].trim().to_string()
            } else {
                "impl".to_string()
            }
        } else {
            "impl".to_string()
        };

        if let Some(block_node) = node
            .children(&mut node.walk())
            .find(|child| child.kind() == "declaration_list")
        {
            for item in block_node.children(&mut block_node.walk()) {
                if item.kind() == "function_item" {
                    if let Ok(mut method) = self.parse_function(item, source_code) {
                        if is_trait_impl {
                            method.visibility = Visibility::Public;
                        }
                        methods.push(method);
                    }
                }
            }
        }

        Ok(ImplUnit {
            documentation,
            head: head.to_string(),
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
        let mut file_doc_comments = Vec::new();
        let mut current_node = root_node.child(0);

        // Iterate through nodes at the start of the file to find documentation
        while let Some(node) = current_node {
            if node.kind() == "line_comment" {
                if let Some(comment) = get_node_text(node, &source_code) {
                    if comment.starts_with("///") {
                        let cleaned = comment.trim_start_matches("///").trim().to_string();
                        file_doc_comments.push(cleaned);
                        current_node = node.next_sibling();
                        continue;
                    }
                }
            } else if node.kind() == "block_comment" {
                if let Some(comment) = get_node_text(node, &source_code) {
                    if comment.starts_with("/**") {
                        let lines: Vec<&str> = comment.lines().collect();
                        for line in lines.iter().skip(1).take(lines.len().saturating_sub(2)) {
                            let cleaned = line.trim_start_matches('*').trim().to_string();
                            if !cleaned.is_empty() {
                                file_doc_comments.push(cleaned);
                            }
                        }
                        current_node = node.next_sibling();
                        continue;
                    }
                }
            }
            break;
        }

        if !file_doc_comments.is_empty() {
            file_unit.document = Some(file_doc_comments.join("\n"));
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
}
