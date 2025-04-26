use crate::{
    Error, FileUnit, FunctionUnit, LanguageParser, ModuleUnit, ParameterUnit, Result, RustParser,
    Visibility,
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
    node.children(&mut node.walk())
        .find(|child| child.kind() == kind)
        .and_then(|child| child.utf8_text(source_code.as_bytes()).ok())
        .map(String::from)
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

    // Helper to parse function parameters
    fn parse_parameters(&self, params_node: Node, source_code: &str) -> Vec<ParameterUnit> {
        let mut parameters = Vec::new();
        for param in params_node.children(&mut params_node.walk()) {
            if param.kind() == "parameter" {
                let mut param_name = "unknown".to_string();
                let mut param_type = "unknown".to_string();
                let mut is_self = false;

                for part in param.children(&mut param.walk()) {
                    match part.kind() {
                        "identifier" => {
                            if let Some(ident) = get_node_text(part, source_code) {
                                param_name = ident.clone();
                                if ident == "self" {
                                    is_self = true;
                                    param_type = "Self".to_string();
                                }
                            }
                        }
                        "type_identifier" | "primitive_type" => {
                            if let Some(type_ident) = get_node_text(part, source_code) {
                                param_type = type_ident;
                            }
                        }
                        _ => {}
                    }
                }

                parameters.push(ParameterUnit {
                    name: param_name,
                    parameter_type: param_type,
                    is_self,
                });
            }
        }
        parameters
    }

    // Helper to parse function return type
    fn parse_return_type(&self, node: Node, source_code: &str) -> Option<String> {
        node.children(&mut node.walk())
            .find(|child| child.kind() == "return_type")
            .and_then(|return_node| {
                return_node
                    .children(&mut return_node.walk())
                    .next() // Get the first child representing the type
                    .and_then(|type_node| get_node_text(type_node, source_code))
            })
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
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut signature = None;
        let mut body = None;

        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "parameters" => parameters = self.parse_parameters(child, source_code),
                "return_type" => return_type = self.parse_return_type(node, source_code),
                _ => {}
            }
        }
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
            parameters,
            return_type,
            source,
            signature,
            body,
            attributes,
        })
    }

    // Helper function to parse struct/enum fields/variants
    fn parse_field_or_variant(&self, node: Node, source_code: &str) -> Result<crate::FieldUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let visibility = self.determine_visibility(node, source_code); // Fields can have visibility

        let mut name = "unknown".to_string();
        let mut field_type = "unknown".to_string();

        // Handle both field_declaration (structs) and enum_variant
        match node.kind() {
            "field_declaration" => {
                name = get_child_node_text(node, "field_identifier", source_code)
                    .unwrap_or_else(|| "unknown_field".to_string());
                // Try to find different kinds of type identifiers
                field_type = get_child_node_text(node, "type_identifier", source_code)
                    .or_else(|| get_child_node_text(node, "primitive_type", source_code))
                    .or_else(|| get_child_node_text(node, "generic_type", source_code))
                    .unwrap_or_else(|| "unknown_type".to_string());
            }
            "enum_variant" => {
                name = get_child_node_text(node, "identifier", source_code)
                    .unwrap_or_else(|| "unknown_variant".to_string());
                // Check if the variant has fields (tuple or struct)
                if node.child_by_field_name("fields").is_some()
                    || node.child_by_field_name("ordered_fields").is_some()
                {
                    field_type = format!("variant({})", name); // Indicate variant with data
                } else {
                    field_type = "variant".to_string(); // Simple variant
                }
            }
            _ => {
                return Err(Error::Parse(format!(
                    "Unexpected node kind for field/variant: {}",
                    node.kind()
                )))
            }
        }

        Ok(crate::FieldUnit {
            name,
            visibility,
            field_type,
            documentation,
            attributes,
        })
    }

    // Parse module declaration ONLY (name, visibility, docs, attributes, source)
    // Content parsing is deferred.
    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let module_source_text = get_node_text(node, source_code);
        let document = documentation.clone();

        println!("DEBUG: Parsing module '{}'", name);

        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut submodules = Vec::new();
        let mut declares = Vec::new();

        // Find the declaration_list node (contains the module body content)
        let body_node_opt = node.child_by_field_name("body").or_else(|| {
            node.children(&mut node.walk())
                .find(|n| n.kind() == "declaration_list")
        });

        if let Some(body_node) = body_node_opt {
            println!(
                "DEBUG: Processing body/declaration_list for module '{}'",
                name
            );
            for child_node in body_node.children(&mut body_node.walk()) {
                match child_node.kind() {
                    "function_item" => match self.parse_function(child_node, source_code) {
                        Ok(func) => functions.push(func),
                        Err(e) => eprintln!("Error parsing function in module {}: {}", name, e),
                    },
                    "mod_item" => {
                        // Check if it's an inline declaration `mod sub;`
                        let inline_mod = child_node.child_by_field_name("body").is_none()
                            && child_node.to_sexp().contains(';');

                        if inline_mod {
                            println!("DEBUG: Found inline module declaration");
                            if let Some(decl_source) = get_node_text(child_node, source_code) {
                                // Only add if not already there
                                if !declares.iter().any(|d: &crate::DeclareStatements| {
                                    d.source == decl_source && d.kind == crate::DeclareKind::Mod
                                }) {
                                    declares.push(crate::DeclareStatements {
                                        source: decl_source,
                                        kind: crate::DeclareKind::Mod,
                                    });

                                    if let Some(sub_mod_name) =
                                        get_child_node_text(child_node, "identifier", source_code)
                                    {
                                        println!(
                                            "DEBUG: Found inline submodule decl '{}' inside '{}'",
                                            sub_mod_name, name
                                        );
                                    }
                                }
                            }
                        } else {
                            // Parse full submodule recursively
                            match self.parse_module(child_node, source_code) {
                                Ok(submodule) => submodules.push(submodule),
                                Err(e) => {
                                    eprintln!("Error parsing submodule in module {}: {}", name, e)
                                }
                            }
                        }
                    }
                    "struct_item" => match self.parse_struct(child_node, source_code) {
                        Ok(struct_unit) => structs.push(struct_unit),
                        Err(e) => eprintln!("Error parsing struct in module {}: {}", name, e),
                    },
                    "enum_item" => match self.parse_enum_as_struct(child_node, source_code) {
                        Ok(enum_unit) => structs.push(enum_unit),
                        Err(e) => eprintln!("Error parsing enum: {}", e),
                    },
                    "trait_item" => match self.parse_trait(child_node, source_code) {
                        Ok(trait_unit) => traits.push(trait_unit),
                        Err(e) => eprintln!("Error parsing trait: {}", e),
                    },
                    "impl_item" => match self.parse_impl(child_node, source_code) {
                        Ok(impl_unit) => {
                            println!(
                                "DEBUG: Pushing impl for {} (trait: {:?}) into module {}",
                                impl_unit.target_type, impl_unit.trait_name, name
                            );
                            impls.push(impl_unit);
                        }
                        Err(e) => eprintln!("Error parsing impl in module {}: {}", name, e),
                    },
                    "type_alias_declaration"
                    | "const_item"
                    | "static_item"
                    | "macro_definition"
                    | "use_declaration"
                    | "extern_crate_declaration" => {
                        if let Some(decl_source) = get_node_text(child_node, source_code) {
                            let kind = match child_node.kind() {
                                k => crate::DeclareKind::Other(k.to_string()),
                            };
                            declares.push(crate::DeclareStatements {
                                source: decl_source,
                                kind,
                            });
                        }
                    }
                    _ => {} // Ignore other top-level items for now
                }
            }
        } else {
            eprintln!(
                "Warning: No declaration_list found for module '{}', and it doesn't look like an inline module.",
                name
            );
            if node
                .child_by_field_name("body")
                .map_or(false, |b| b.named_child_count() == 0)
            {
                println!(
                    "DEBUG: Module '{}' appears to have an empty body {{}}.",
                    name
                );
            }
        }

        Ok(ModuleUnit {
            name,
            visibility,
            document,
            declares,
            functions,
            structs,
            traits,
            impls,
            submodules,
            source: module_source_text, // Store the source of the module definition itself
            attributes,
        })
    }

    // Parse an enum as a StructUnit for simplicity
    fn parse_enum_as_struct(&self, node: Node, source_code: &str) -> Result<crate::StructUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let name = get_child_node_text(node, "identifier", source_code)
            .or_else(|| get_child_node_text(node, "type_identifier", source_code))
            .unwrap_or_else(|| "unknown_enum".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let source = get_node_text(node, source_code);

        let mut variants = Vec::new();
        // Find the body node robustly - look for named child 'body' or common brace patterns
        let body_node = node
            .child_by_field_name("body")
            .or_else(|| {
                node.children(&mut node.walk())
                    .find(|n| n.kind() == "declaration_list")
            })
            .or_else(|| {
                node.children(&mut node.walk())
                    .find(|n| n.kind() == "enum_variant_list")
            }); // tree-sitter-rust < 0.20.7 used this name

        if let Some(body) = body_node {
            // Find the list of variants within the body
            let variant_list_node = body
                .children(&mut body.walk())
                .find(|n| n.kind() == "enum_variant_list")
                .unwrap_or(body); // Fallback to body itself if no specific list node

            for variant_node in variant_list_node.named_children(&mut variant_list_node.walk()) {
                if variant_node.kind() == "enum_variant" {
                    match self.parse_field_or_variant(variant_node, source_code) {
                        Ok(variant_unit) => variants.push(variant_unit),
                        Err(e) => {
                            eprintln!("Warning: Failed to parse enum variant in '{}': {}", name, e)
                        } // Log error but continue
                    }
                }
            }
        } else {
            eprintln!(
                "Warning: Could not find 'body' child in enum item for '{}'",
                name
            );
        }

        Ok(crate::StructUnit {
            name,
            visibility,
            documentation,
            fields: variants,    // Store variants in the 'fields' vec
            methods: Vec::new(), // Methods are in impl blocks
            source,
            attributes,
        })
    }

    // Parse a struct definition
    fn parse_struct(&self, node: Node, source_code: &str) -> Result<crate::StructUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let name = get_child_node_text(node, "identifier", source_code)
            .or_else(|| get_child_node_text(node, "type_identifier", source_code)) // Structs might use type_identifier
            .unwrap_or_else(|| "unknown_struct".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let source = get_node_text(node, source_code);

        let mut fields = Vec::new();
        // Look for field declaration list (structs can have different body types)
        let field_list_node_opt = node
            .children(&mut node.walk())
            .find(|n| n.kind() == "field_declaration_list"); // Check for named fields

        if let Some(field_list_node) = field_list_node_opt {
            for field_node in field_list_node.children(&mut field_list_node.walk()) {
                if field_node.kind() == "field_declaration" {
                    match self.parse_field_or_variant(field_node, source_code) {
                        Ok(field_unit) => fields.push(field_unit),
                        Err(e) => eprintln!("Warning: Failed to parse struct field: {}", e), // Log error but continue
                    }
                }
            }
        } else {
            // Handle tuple structs (ordered_field_declaration_list) or unit structs (no list)
            let ordered_field_list_node_opt = node
                .children(&mut node.walk())
                .find(|n| n.kind() == "ordered_field_declaration_list"); // Check for tuple fields

            if let Some(_ordered_field_list_node) = ordered_field_list_node_opt {
                // Use _ prefix as it's not used
                // TODO: Implement parsing for tuple struct fields if needed
                // For now, we might just note that it's a tuple struct
                eprintln!(
                    "Note: Tuple struct '{}' field parsing not fully implemented.",
                    name
                );
            } else {
                // Likely a unit struct
                eprintln!("Note: Unit struct '{}' detected.", name);
            }
        }

        Ok(crate::StructUnit {
            name,
            visibility,
            documentation,
            fields,
            methods: Vec::new(), // Methods are in impl blocks
            source,
            attributes,
        })
    }

    // Parse a trait definition
    fn parse_trait(&self, node: Node, source_code: &str) -> Result<crate::TraitUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let name = get_child_node_text(node, "type_identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let visibility = self.determine_visibility(node, source_code);
        let source = get_node_text(node, source_code);

        let mut methods = Vec::new();
        if let Some(item_list_node) = node
            .children(&mut node.walk())
            .find(|n| n.kind() == "trait_item_list")
        // Assuming this node kind exists
        {
            for item_node in item_list_node.children(&mut item_list_node.walk()) {
                // Look for function signatures within the trait body
                if item_node.kind() == "function_signature_item" {
                    // Parse function signature similar to parse_function but without body
                    if let Ok(method_sig) = self.parse_function(item_node, source_code) {
                        methods.push(method_sig);
                    }
                }
                // Potentially handle associated types, consts, etc.
            }
        }

        Ok(crate::TraitUnit {
            name,
            visibility,
            documentation,
            methods,
            source,
            attributes,
        })
    }

    // Parse an impl block
    fn parse_impl(&self, node: Node, source_code: &str) -> Result<crate::ImplUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let attributes = extract_attributes(node, source_code);
        let source = get_node_text(node, source_code);

        let mut target_type = "unknown".to_string();
        let mut trait_name = None;

        // Find the path (type identifier) for the implementing type - should be after any "for" token
        let for_token_position = node
            .children(&mut node.walk())
            .position(|c| c.kind() == "for_token");

        // Find trait name (if this is a trait implementation)
        let possible_trait = node.child_by_field_name("trait");
        if let Some(trait_node) = possible_trait {
            // This is a trait impl - extract the trait name
            if let Some(trait_text) = get_node_text(trait_node, source_code) {
                trait_name = Some(trait_text);
            }
        }

        // The type being implemented is either:
        // 1. If there's a "for" token, it's the type after the "for"
        // 2. If there's no "for" token, it's the first type identifer
        if let Some(for_pos) = for_token_position {
            // Find the target type after the "for" token
            let children: Vec<_> = node.children(&mut node.walk()).collect();
            if for_pos + 1 < children.len() {
                if let Some(type_text) = get_node_text(children[for_pos + 1], source_code) {
                    target_type = type_text;
                }
            }
        } else {
            // No "for" token - this is an inherent impl
            // The type is the first type identifier
            if let Some(first_type) = node.children(&mut node.walk()).find(|c| {
                println!("c: {:#?} {}", c, c.to_string());
                c.kind() == "type_identifier"
                    || c.kind() == "generic_type"
                    || c.kind() == "primitive_type"
            }) {
                if let Some(type_text) = get_node_text(first_type, source_code) {
                    println!("type_text: {}", type_text);
                    target_type = type_text;
                }
            }
        }

        // For easier debugging
        println!(
            "DEBUG: Impl target_type: {}, trait_name: {:?}",
            target_type, trait_name
        );

        let mut methods = Vec::new();

        // Find the declaration_list node containing methods
        let mut decl_list_node_opt = None;
        for child in node.children(&mut node.walk()) {
            if child.kind() == "declaration_list" {
                decl_list_node_opt = Some(child);
                break;
            }
        }

        if let Some(decl_list_node) = decl_list_node_opt {
            println!("DEBUG: Found declaration_list in impl for {}", target_type);
            for item_node in decl_list_node.children(&mut decl_list_node.walk()) {
                println!(
                    "DEBUG: Checking item '{}' in impl declaration_list",
                    item_node.kind()
                );
                if item_node.kind() == "function_item" {
                    println!("DEBUG: Parsing function_item in impl declaration_list");
                    if let Ok(function) = self.parse_function(item_node, source_code) {
                        println!("DEBUG: Parsed method: {}", function.name);
                        methods.push(function);
                    } else {
                        println!("DEBUG: Failed to parse function_item in impl declaration_list");
                    }
                }
                // Potentially handle other associated items like consts, types
            }
        } else {
            println!(
                "DEBUG: No declaration_list found in impl for {}",
                target_type
            );
            // Debug: Print children of impl_item if list not found
            println!("DEBUG: Children of impl_item for {}:", target_type);
            for child in node.children(&mut node.walk()) {
                println!(
                    "  - Kind: {}, Text: {:?}",
                    child.kind(),
                    get_node_text(child, source_code)
                );
            }
        }

        Ok(crate::ImplUnit {
            target_type,
            trait_name,
            documentation,
            methods,
            source,
            attributes,
        })
    }
}

impl LanguageParser for RustParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        let content = fs::read_to_string(file_path)?;
        let tree = self
            .parser
            .parse(&content, None)
            .ok_or(Error::Parse(format!(
                "Failed to parse file: {}",
                file_path.display()
            )))?;
        let root_node = tree.root_node();

        let mut file_unit = FileUnit::new(file_path.to_path_buf());
        file_unit.source = Some(content.clone());

        // Extract file-level documentation first
        let mut file_doc_comments = Vec::new();
        let mut cursor = root_node.walk();
        let mut first_code_node_start_byte: Option<usize> = None;

        for node in root_node.children(&mut cursor) {
            let kind = node.kind();
            if kind == "line_comment" {
                if let Some(comment) = get_node_text(node, &content) {
                    // Check for module-level doc comments (//!)
                    if comment.starts_with("//!") {
                        let cleaned = comment.trim_start_matches("//!").trim().to_string();
                        file_doc_comments.push(cleaned);
                    } else if comment.starts_with("///") {
                        // Regular doc comments (///)
                        let cleaned = comment.trim_start_matches("///").trim().to_string();
                        file_doc_comments.push(cleaned);
                    } else {
                        // Stop if it's a non-doc line comment
                        // Record the start byte of the first non-doc comment/code node
                        if first_code_node_start_byte.is_none() {
                            first_code_node_start_byte = Some(node.start_byte());
                        }
                        break;
                    }
                }
            } else if kind == "block_comment" {
                if let Some(comment) = get_node_text(node, &content) {
                    // Check for module-level doc comments (/*!)
                    if comment.starts_with("/*!") {
                        let lines: Vec<&str> = comment.lines().collect();
                        if lines.len() > 1 {
                            for line in lines[1..lines.len() - 1].iter() {
                                let cleaned = line.trim_start_matches('*').trim().to_string();
                                if !cleaned.is_empty() {
                                    file_doc_comments.push(cleaned);
                                }
                            }
                        }
                    } else if comment.starts_with("/**") {
                        // Regular doc block comments
                        let lines: Vec<&str> = comment.lines().collect();
                        if lines.len() > 1 {
                            for line in lines[1..lines.len() - 1].iter() {
                                let cleaned = line.trim_start_matches('*').trim().to_string();
                                if !cleaned.is_empty() {
                                    file_doc_comments.push(cleaned);
                                }
                            }
                        }
                    } else {
                        // Stop if it's a non-doc block comment
                        // Record the start byte of the first non-doc comment/code node
                        if first_code_node_start_byte.is_none() {
                            first_code_node_start_byte = Some(node.start_byte());
                        }
                        break;
                    }
                }
            } else if kind != "attribute_item" {
                // It's a code node or non-doc comment
                // Record the start byte and stop collecting comments
                if first_code_node_start_byte.is_none() {
                    first_code_node_start_byte = Some(node.start_byte());
                }
                break;
            }
            // Skip attribute_items associated with the file itself (usually handled by subsequent items)
        }

        if !file_doc_comments.is_empty() {
            file_unit.document = Some(file_doc_comments.join("\n"));
        }

        // Reset cursor or create a new one to iterate again for actual items
        let mut item_cursor = root_node.walk();
        for node in root_node.children(&mut item_cursor) {
            // Skip nodes that occur *before* the first identified code/non-doc-comment node
            if let Some(start_byte) = first_code_node_start_byte {
                if node.start_byte() < start_byte {
                    // This includes the file-level comments/attributes we already processed
                    continue;
                }
            } else if first_code_node_start_byte.is_none() {
                // This case means the file ONLY contained doc comments/attributes handled above
                let kind = node.kind();
                if kind == "line_comment" || kind == "block_comment" || kind == "attribute_item" {
                    continue;
                }
            }

            let kind = node.kind(); // Define kind here for the match scope
                                    // Process the current node (which is either the first code item or a subsequent one)
            match kind {
                "function_item" => {
                    if let Ok(func_unit) = self.parse_function(node, &content) {
                        file_unit.functions.push(func_unit);
                    }
                }
                "mod_item" => {
                    if let Ok(mod_unit) = self.parse_module(node, &content) {
                        file_unit.modules.push(mod_unit);
                    } else {
                        // If parse_module failed, check if it was an inline mod decl
                        // that should be added as a file-level declaration
                        if node.child_by_field_name("body").is_none()
                            && node.to_sexp().contains(";")
                        {
                            if let Some(decl_source) = get_node_text(node, &content) {
                                file_unit.declares.push(crate::DeclareStatements {
                                    source: decl_source,
                                    kind: crate::DeclareKind::Mod,
                                });
                            }
                        }
                    }
                }
                "struct_item" => match self.parse_struct(node, &content) {
                    Ok(struct_unit) => file_unit.structs.push(struct_unit),
                    Err(e) => eprintln!("Error parsing struct: {}", e),
                },
                "enum_item" => match self.parse_enum_as_struct(node, &content) {
                    Ok(enum_unit) => file_unit.structs.push(enum_unit),
                    Err(e) => eprintln!("Error parsing enum: {}", e),
                },
                "trait_item" => match self.parse_trait(node, &content) {
                    Ok(trait_unit) => file_unit.traits.push(trait_unit),
                    Err(e) => eprintln!("Error parsing trait: {}", e),
                },
                "impl_item" => match self.parse_impl(node, &content) {
                    Ok(impl_unit) => file_unit.impls.push(impl_unit),
                    Err(e) => eprintln!("Error parsing impl: {}", e),
                },
                "use_declaration" | "extern_crate_declaration" => {
                    if let Some(decl_source) = get_node_text(node, &content) {
                        let kind = match kind {
                            "use_declaration" => crate::DeclareKind::Use,
                            "extern_crate_declaration" => crate::DeclareKind::Import,
                            _ => crate::DeclareKind::Other(kind.to_string()), // Should not happen
                        };
                        file_unit.declares.push(crate::DeclareStatements {
                            source: decl_source,
                            kind,
                        });
                    }
                }
                "type_alias_declaration"
                | "type_item"
                | "const_item"
                | "static_item"
                | "macro_definition" => {
                    if let Some(decl_source) = get_node_text(node, &content) {
                        let kind = match kind {
                            "type_alias_declaration" => {
                                crate::DeclareKind::Other("type_alias_declaration".to_string())
                            }
                            "type_item" => {
                                crate::DeclareKind::Other("type_alias_declaration".to_string())
                            } // Map type_item to type_alias_declaration for consistency
                            "const_item" => crate::DeclareKind::Other("const_item".to_string()),
                            "static_item" => crate::DeclareKind::Other("static_item".to_string()),
                            "macro_definition" => {
                                crate::DeclareKind::Other("macro_definition".to_string())
                            }
                            _ => crate::DeclareKind::Other(kind.to_string()),
                        };

                        // Debug logging for type aliases and other declarations
                        println!("DEBUG: Found declaration with kind: {:?}", kind);
                        println!("DEBUG: Declaration source: {}", decl_source);

                        file_unit.declares.push(crate::DeclareStatements {
                            source: decl_source,
                            kind,
                        });
                    }
                }
                _ => {
                    println!("INFO: Unhandled top-level item kind: {}", kind);
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
    use std::path::Path;

    // Helper function to parse a fixture file
    fn parse_fixture(file_name: &str) -> Result<FileUnit> {
        let mut parser = RustParser::try_new().expect("Failed to create Rust parser");
        let file_path = Path::new("fixtures").join(file_name);
        parser.parse_file(&file_path)
    }

    #[test]
    fn test_parse_file_level_items() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");

        // Arrange: Expected values
        let expected_doc =
            "This is a file-level documentation comment\nIt describes the purpose of this file\nThis is a public module";

        // Act: Get actual values
        let actual_doc = file_unit.document.as_deref();
        let num_declares = file_unit.declares.len();
        let num_top_level_functions = file_unit.functions.len();
        let num_top_level_structs = file_unit.structs.len(); // Includes enums parsed as structs
        let num_top_level_traits = file_unit.traits.len();
        let num_top_level_impls = file_unit.impls.len();
        let num_modules = file_unit.modules.len();

        // Assert: Check file-level properties
        assert_eq!(
            actual_doc,
            Some(expected_doc),
            "File documentation mismatch"
        );
        assert!(num_declares > 0, "Expected some declarations");
        assert_eq!(num_top_level_functions, 2, "Expected 2 top-level functions");
        // There are only 2 top-level structs: AttributedStruct, GenericStruct
        assert_eq!(num_top_level_structs, 2, "Expected 2 top-level structs");
        assert_eq!(num_top_level_traits, 1, "Expected 1 top-level trait"); // GenericTrait
        assert_eq!(num_top_level_impls, 2, "Expected 2 top-level impls"); // AttributedStruct, GenericStruct
        assert_eq!(num_modules, 3, "Expected 3 modules");
    }

    #[test]
    fn test_parse_declarations() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");

        // Assert: Check for specific declaration kinds
        assert!(
            file_unit
                .declares
                .iter()
                .any(|d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "type_alias_declaration")
                    && d.source.contains("PublicType")),
            "PublicType alias not found"
        );
        assert!(
            file_unit.declares.iter().any(
                |d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "const_item")
                    && d.source.contains("PUBLIC_CONSTANT")
            ),
            "PUBLIC_CONSTANT not found"
        );
        assert!(
            file_unit.declares.iter().any(
                |d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "static_item")
                    && d.source.contains("PUBLIC_STATIC")
            ),
            "PUBLIC_STATIC not found"
        );
        assert!(
            file_unit.declares.iter().any(
                |d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "macro_definition")
                    && d.source.contains("public_macro")
            ),
            "public_macro not found"
        );
    }

    #[test]
    fn test_parse_top_level_functions() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");

        // Assert: Check public function
        let public_fn = file_unit
            .functions
            .iter()
            .find(|f| f.name == "public_function");
        assert!(public_fn.is_some(), "Public function not found");
        assert_eq!(public_fn.unwrap().visibility, Visibility::Public);
        assert!(public_fn.unwrap().documentation.is_some());

        // Assert: Check private function
        let private_fn = file_unit
            .functions
            .iter()
            .find(|f| f.name == "private_function");
        assert!(private_fn.is_some(), "Private function not found");
        assert_eq!(private_fn.unwrap().visibility, Visibility::Private);
        assert!(private_fn.unwrap().documentation.is_some());
    }

    #[test]
    fn test_parse_module_structure() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");

        // Assert: Check public module structure
        let public_mod = file_unit.modules.iter().find(|m| m.name == "public_module");
        assert!(public_mod.is_some(), "Public module not found");
        let public_mod = public_mod.unwrap();
        assert_eq!(public_mod.visibility, Visibility::Public);
        assert!(
            public_mod.document.is_some(),
            "Public module documentation missing"
        );
        assert!(
            public_mod.structs.len() > 0,
            "Expected structs/enums in public_module"
        );
        assert_eq!(
            public_mod
                .structs
                .iter()
                .filter(|s| s.name == "PublicStruct")
                .count(),
            1,
            "PublicStruct missing"
        );
        assert_eq!(
            public_mod
                .structs
                .iter()
                .filter(|s| s.name == "PublicEnum")
                .count(),
            1,
            "PublicEnum missing"
        );
        assert_eq!(
            public_mod.traits.len(),
            1,
            "Expected 1 trait in public_module"
        );
        assert_eq!(
            public_mod.traits[0].name, "PublicTrait",
            "PublicTrait missing"
        );
        assert_eq!(
            public_mod.impls.len(),
            2, // Should find both inherent and trait impl now
            "Expected 2 impls in public_module"
        ); // Inherent + Trait
        assert!(
            public_mod.functions.is_empty(),
            "Expected no top-level functions directly in public_module"
        );
        // Check submodule declarations
        assert!(
            public_mod
                .declares
                .iter()
                .any(|d| d.kind == crate::DeclareKind::Mod && d.source.contains("nested_module")),
            "Expected 'mod nested_module;' declaration"
        );

        // Assert: Check private module structure
        let private_mod = file_unit
            .modules
            .iter()
            .find(|m| m.name == "private_module");
        assert!(private_mod.is_some(), "Private module not found");
        let private_mod = private_mod.unwrap();
        assert_eq!(private_mod.visibility, Visibility::Private);
        assert!(
            private_mod.document.is_some(),
            "Private module documentation missing"
        );
        assert!(
            private_mod.structs.len() > 0,
            "Expected structs/enums in private_module"
        );
        assert_eq!(
            private_mod.traits.len(),
            1,
            "Expected 1 trait in private_module"
        );
        assert_eq!(
            private_mod.impls.len(),
            2,
            "Expected 2 impls in private_module"
        );

        // Assert: Check test module structure
        let test_mod = file_unit.modules.iter().find(|m| m.name == "tests");
        assert!(test_mod.is_some(), "Test module not found");
        assert!(!test_mod.unwrap().attributes.is_empty());
        assert!(
            test_mod.unwrap().functions.len() > 0,
            "Expected functions in test module"
        );
        assert!(
            test_mod
                .unwrap()
                .functions
                .iter()
                .any(|f| f.attributes.contains(&"#[test]".to_string())),
            "Expected test functions"
        );
        assert!(
            test_mod
                .unwrap()
                .declares
                .iter()
                // Correct the check for use declarations
                .any(|d| d.kind == crate::DeclareKind::Use && d.source.starts_with("use super::*")),
            "Expected use declaration in test module"
        );
    }

    // Reinstate tests for items within modules
    #[test]
    fn test_parse_struct_fields_in_module() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");
        let public_module = file_unit
            .modules
            .iter()
            .find(|m| m.name == "public_module")
            .expect("Public module not found");
        let public_struct = public_module
            .structs
            .iter()
            .find(|s| s.name == "PublicStruct")
            .expect("PublicStruct not found");

        assert_eq!(
            public_struct.fields.len(),
            2,
            "Expected 2 fields in PublicStruct"
        );

        let public_field = public_struct.fields.iter().find(|f| f.name == "field");
        assert!(public_field.is_some(), "Public field 'field' not found");
        assert_eq!(public_field.unwrap().visibility, Visibility::Public);
        assert!(public_field.unwrap().documentation.is_some());
        assert_eq!(public_field.unwrap().field_type, "String");

        let private_field = public_struct
            .fields
            .iter()
            .find(|f| f.name == "private_field");
        assert!(
            private_field.is_some(),
            "Private field 'private_field' not found"
        );
        assert_eq!(private_field.unwrap().visibility, Visibility::Private);
        assert!(private_field.unwrap().documentation.is_some());
        assert_eq!(private_field.unwrap().field_type, "i32");
    }

    #[test]
    fn test_parse_impl_blocks_in_module() {
        let file_unit = parse_fixture("sample.rs").expect("Failed to parse sample.rs");
        let public_module = file_unit
            .modules
            .iter()
            .find(|m| m.name == "public_module")
            .expect("Public module not found");

        // Find inherent impl for PublicStruct
        let inherent_impl = public_module
            .impls
            .iter()
            .find(|i| i.target_type == "PublicStruct" && i.trait_name.is_none())
            .expect("Inherent impl for PublicStruct not found");
        assert_eq!(
            inherent_impl.methods.len(),
            2,
            "Expected 2 methods in inherent impl"
        );
        assert!(
            inherent_impl.methods.iter().any(|m| m.name == "new"),
            "'new' method not found"
        );
        assert!(
            inherent_impl
                .methods
                .iter()
                .any(|m| m.name == "get_private_field"),
            "'get_private_field' method not found"
        );

        // Find trait impl for PublicStruct
        let trait_impl = public_module
            .impls
            .iter()
            .find(|i| {
                println!("i: {:#?}", i);
                i.target_type == "PublicStruct" && i.trait_name.as_deref() == Some("PublicTrait")
            })
            .expect("Trait impl for PublicStruct not found");
        assert_eq!(
            trait_impl.methods.len(),
            1,
            "Expected 1 method in trait impl"
        );
        assert!(
            trait_impl.methods.iter().any(|m| m.name == "method"),
            "'method' method not found" // Corrected method name in assert message
        );
    }
}
