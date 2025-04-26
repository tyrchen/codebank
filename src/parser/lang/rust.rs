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
        let mut attributes = Vec::new();

        // Extract attributes
        let prev_node = node.prev_sibling();
        if let Some(prev) = prev_node {
            if prev.kind() == "attribute_item" {
                if let Ok(attr_text) = prev.utf8_text(source_code.as_bytes()) {
                    attributes.push(attr_text.to_string());
                }
            }
        }

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

        // Extract signature and body
        let mut signature = None;
        let mut body = None;

        if let Some(src) = &source {
            // Simple approach to get signature and body
            if let Some(body_start) = src.find('{') {
                signature = Some(src[0..body_start].trim().to_string());
                body = Some(src[body_start..].trim().to_string());
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

    // Parse modules
    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let mut name = "unknown".to_string();
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = Vec::new();
        let document = documentation.clone(); // Clone documentation to document field
        let mut declares = Vec::new(); // Initialize empty declares

        // Extract attributes
        let prev_node = node.prev_sibling();
        if let Some(prev) = prev_node {
            if prev.kind() == "attribute_item" {
                if let Ok(attr_text) = prev.utf8_text(source_code.as_bytes()) {
                    attributes.push(attr_text.to_string());
                }
            }
        }

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

        // Initialize collections
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut submodules = Vec::new();

        // Parse module body contents
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" {
                // This is the module body
                let mut body_cursor = child.walk();

                // Traverse all definitions in the module body
                for item in child.children(&mut body_cursor) {
                    match item.kind() {
                        "function_item" => {
                            if let Ok(function) = self.parse_function(item, source_code) {
                                functions.push(function);
                            }
                        }
                        "struct_item" => {
                            if let Ok(struct_def) = self.parse_struct(item, source_code) {
                                structs.push(struct_def);
                            }
                        }
                        "trait_item" => {
                            if let Ok(trait_def) = self.parse_trait(item, source_code) {
                                traits.push(trait_def);
                            }
                        }
                        "impl_item" => {
                            if let Ok(impl_def) = self.parse_impl(item, source_code) {
                                impls.push(impl_def);
                            }
                        }
                        "mod_item" => {
                            if let Ok(submodule) = self.parse_module(item, source_code) {
                                submodules.push(submodule);
                            }
                        }
                        "use_declaration" => {
                            if let Ok(use_text) = item.utf8_text(source_code.as_bytes()) {
                                declares.push(crate::DeclareStatements {
                                    source: use_text.to_string(),
                                    kind: crate::DeclareKind::Use,
                                });
                            }
                        }
                        "mod_declaration" => {
                            if let Ok(mod_text) = item.utf8_text(source_code.as_bytes()) {
                                declares.push(crate::DeclareStatements {
                                    source: mod_text.to_string(),
                                    kind: crate::DeclareKind::Mod,
                                });
                            }
                        }
                        "enum_item" => {
                            // Handle enum declarations - they're similar to structs in our model
                            if let Ok(enum_text) = item.utf8_text(source_code.as_bytes()) {
                                let enum_visibility = self.determine_visibility(item, source_code);
                                let enum_documentation =
                                    self.extract_documentation(item, source_code);

                                // Extract enum name
                                let mut enum_name = "unknown".to_string();
                                let mut enum_cursor = item.walk();
                                for part in item.children(&mut enum_cursor) {
                                    if part.kind() == "type_identifier" {
                                        if let Ok(name) = part.utf8_text(source_code.as_bytes()) {
                                            enum_name = name.to_string();
                                            break;
                                        }
                                    }
                                }

                                // Create a struct representation of the enum
                                let enum_struct = crate::StructUnit {
                                    name: enum_name,
                                    visibility: enum_visibility,
                                    documentation: enum_documentation,
                                    fields: Vec::new(), // Enums don't have fields in the same way
                                    methods: Vec::new(), // Methods would be in impl blocks
                                    source: Some(enum_text.to_string()),
                                    attributes: Vec::new(), // We'd need to extract these separately
                                };

                                structs.push(enum_struct);
                            }
                        }
                        // Can add more item types here
                        _ => continue,
                    }
                }
            }
        }

        // Extract source code
        let source = if let Ok(mod_source) = node.utf8_text(source_code.as_bytes()) {
            Some(mod_source.to_string())
        } else {
            None
        };

        Ok(ModuleUnit {
            name,
            document,
            declares,
            visibility,
            documentation,
            functions,
            structs,
            traits,
            impls,
            submodules,
            source,
            attributes,
        })
    }

    // Parse a struct definition
    fn parse_struct(&self, node: Node, source_code: &str) -> Result<crate::StructUnit> {
        let mut name = "unknown".to_string();
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = Vec::new();

        // Extract attributes
        let prev_node = node.prev_sibling();
        if let Some(prev) = prev_node {
            if prev.kind() == "attribute_item" {
                if let Ok(attr_text) = prev.utf8_text(source_code.as_bytes()) {
                    attributes.push(attr_text.to_string());
                }
            }
        }

        let mut cursor = node.walk();

        // Extract struct name
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                if let Ok(ident) = child.utf8_text(source_code.as_bytes()) {
                    name = ident.to_string();
                    break;
                }
            }
        }

        // Initialize fields and methods (methods would be in separate impl blocks)
        let mut fields = Vec::new();
        let methods = Vec::new();

        // Parse struct fields
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut field_cursor = child.walk();
                for field_node in child.children(&mut field_cursor) {
                    if field_node.kind() == "field_declaration" {
                        // Extract field information
                        let mut field_name = "unknown".to_string();
                        let mut field_type = "unknown".to_string();
                        let mut field_visibility = Visibility::Private;
                        let field_documentation =
                            self.extract_documentation(field_node, source_code);
                        let field_attributes = Vec::new();

                        let mut field_cursor2 = field_node.walk();
                        for field_part in field_node.children(&mut field_cursor2) {
                            if field_part.kind() == "visibility_modifier" {
                                if let Ok(vis_text) = field_part.utf8_text(source_code.as_bytes()) {
                                    if vis_text == "pub" {
                                        field_visibility = Visibility::Public;
                                    }
                                }
                            } else if field_part.kind() == "field_identifier" {
                                if let Ok(ident) = field_part.utf8_text(source_code.as_bytes()) {
                                    field_name = ident.to_string();
                                }
                            } else if field_part.kind() == "type_identifier"
                                || field_part.kind() == "primitive_type"
                            {
                                if let Ok(type_name) = field_part.utf8_text(source_code.as_bytes())
                                {
                                    field_type = type_name.to_string();
                                }
                            }
                        }

                        fields.push(crate::FieldUnit {
                            name: field_name,
                            visibility: field_visibility,
                            field_type,
                            documentation: field_documentation,
                            attributes: field_attributes,
                        });
                    }
                }
            }
        }

        // Extract source code
        let source = if let Ok(struct_source) = node.utf8_text(source_code.as_bytes()) {
            Some(struct_source.to_string())
        } else {
            None
        };

        Ok(crate::StructUnit {
            name,
            visibility,
            documentation,
            fields,
            methods,
            source,
            attributes,
        })
    }

    // Parse a trait definition
    fn parse_trait(&self, node: Node, source_code: &str) -> Result<crate::TraitUnit> {
        let mut name = "unknown".to_string();
        let visibility = self.determine_visibility(node, source_code);
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = Vec::new();

        // Extract attributes
        let prev_node = node.prev_sibling();
        if let Some(prev) = prev_node {
            if prev.kind() == "attribute_item" {
                if let Ok(attr_text) = prev.utf8_text(source_code.as_bytes()) {
                    attributes.push(attr_text.to_string());
                }
            }
        }

        let mut cursor = node.walk();

        // Extract trait name
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                if let Ok(ident) = child.utf8_text(source_code.as_bytes()) {
                    name = ident.to_string();
                    break;
                }
            }
        }

        // Parse trait methods
        let mut methods = Vec::new();
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "trait_item_list" {
                let mut method_cursor = child.walk();
                for method_node in child.children(&mut method_cursor) {
                    if method_node.kind() == "function_signature_item" {
                        // This is a trait method signature
                        let mut method_name = "unknown".to_string();
                        let method_visibility = Visibility::Public; // Trait methods are implicitly public
                        let method_documentation =
                            self.extract_documentation(method_node, source_code);
                        let mut parameters = Vec::new();
                        let mut return_type = None;
                        let method_attributes = Vec::new();

                        let mut method_cursor2 = method_node.walk();
                        for part in method_node.children(&mut method_cursor2) {
                            if part.kind() == "identifier" {
                                if let Ok(ident) = part.utf8_text(source_code.as_bytes()) {
                                    method_name = ident.to_string();
                                }
                            } else if part.kind() == "parameters" {
                                // Extract parameters similar to parse_function
                                let mut param_cursor = part.walk();
                                for param in part.children(&mut param_cursor) {
                                    if param.kind() == "parameter" {
                                        let mut param_name = "unknown".to_string();
                                        let mut param_type = "unknown".to_string();
                                        let mut is_self = false;

                                        let mut inner_cursor = param.walk();
                                        for param_part in param.children(&mut inner_cursor) {
                                            if param_part.kind() == "identifier" {
                                                if let Ok(ident) =
                                                    param_part.utf8_text(source_code.as_bytes())
                                                {
                                                    param_name = ident.to_string();
                                                    if ident == "self" {
                                                        is_self = true;
                                                        param_type = "Self".to_string();
                                                    }
                                                }
                                            } else if param_part.kind() == "type_identifier"
                                                || param_part.kind() == "primitive_type"
                                            {
                                                if let Ok(type_ident) =
                                                    param_part.utf8_text(source_code.as_bytes())
                                                {
                                                    param_type = type_ident.to_string();
                                                }
                                            }
                                        }

                                        parameters.push(crate::ParameterUnit {
                                            name: param_name,
                                            parameter_type: param_type,
                                            is_self,
                                        });
                                    }
                                }
                            } else if part.kind() == "return_type" {
                                let mut inner_cursor = part.walk();
                                for type_node in part.children(&mut inner_cursor) {
                                    if let Ok(type_str) =
                                        type_node.utf8_text(source_code.as_bytes())
                                    {
                                        return_type = Some(type_str.to_string());
                                        break;
                                    }
                                }
                            }
                        }

                        // Extract method source
                        let method_source = if let Ok(func_source) =
                            method_node.utf8_text(source_code.as_bytes())
                        {
                            Some(func_source.to_string())
                        } else {
                            None
                        };

                        let method_signature = method_source.clone();

                        methods.push(crate::FunctionUnit {
                            name: method_name,
                            visibility: method_visibility,
                            documentation: method_documentation,
                            parameters,
                            return_type,
                            source: method_source,
                            signature: method_signature,
                            body: None, // Trait methods don't have bodies
                            attributes: method_attributes,
                        });
                    }
                }
            }
        }

        // Extract source code
        let source = if let Ok(trait_source) = node.utf8_text(source_code.as_bytes()) {
            Some(trait_source.to_string())
        } else {
            None
        };

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
        let mut target_type = "unknown".to_string();
        let mut trait_name = None;
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = Vec::new();

        // Extract attributes
        let prev_node = node.prev_sibling();
        if let Some(prev) = prev_node {
            if prev.kind() == "attribute_item" {
                if let Ok(attr_text) = prev.utf8_text(source_code.as_bytes()) {
                    attributes.push(attr_text.to_string());
                }
            }
        }

        let mut cursor = node.walk();

        // Extract target type and trait name if present
        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                // This could be either the trait name or the target type
                if let Ok(ident) = child.utf8_text(source_code.as_bytes()) {
                    // If we've already seen the trait name, this is the target type
                    if trait_name.is_some() {
                        target_type = ident.to_string();
                    } else {
                        // Check if next sibling is "for" to determine if this is a trait name
                        let next = child.next_sibling();
                        if let Some(next_node) = next {
                            if next_node.kind() == "for_token" {
                                trait_name = Some(ident.to_string());
                                continue;
                            }
                        }
                        // If we didn't find "for", this is the target type
                        target_type = ident.to_string();
                    }
                }
            }
        }

        // Parse impl methods
        let mut methods = Vec::new();
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "block" {
                let mut method_cursor = child.walk();
                for method_node in child.children(&mut method_cursor) {
                    if method_node.kind() == "function_item" {
                        if let Ok(function) = self.parse_function(method_node, source_code) {
                            methods.push(function);
                        }
                    }
                }
            }
        }

        // Extract source code
        let source = if let Ok(impl_source) = node.utf8_text(source_code.as_bytes()) {
            Some(impl_source.to_string())
        } else {
            None
        };

        Ok(crate::ImplUnit {
            target_type,
            trait_name,
            documentation,
            methods,
            source,
            attributes,
        })
    }

    // Parse module content from source
    fn parse_module_content(&self, module: &mut ModuleUnit, source: &str) -> Result<()> {
        // Create patterns for extracting the module body - handle both pub mod and mod
        let pub_pattern = format!("pub mod {} {{", module.name);
        let regular_pattern = format!("mod {} {{", module.name);

        // Find the module body
        let (start_idx, is_pub) = if let Some(idx) = source.find(&pub_pattern) {
            (idx, true)
        } else if let Some(idx) = source.find(&regular_pattern) {
            (idx, false)
        } else {
            return Ok(());
        };

        // Calculate the position of the opening brace
        let pattern_len = if is_pub {
            pub_pattern.len() - 1
        } else {
            regular_pattern.len() - 1
        };
        let body_start = start_idx + pattern_len;

        // Find the matching closing brace
        let mut brace_count = 1;
        let mut body_end = body_start;

        for (i, c) in source[body_start..].chars().enumerate() {
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    body_end = body_start + i;
                    break;
                }
            }
        }

        // Extract the module body
        if body_end > body_start {
            let body = &source[body_start..body_end];
            println!("DEBUG: Module {} body length: {}", module.name, body.len());

            // Create a temporary source file for parsing - we need to wrap it in a source file context
            let temp_source = format!("{{\n{}\n}}", body);

            // We need to clone the parser because we can't borrow self as mutable
            let mut parser_clone = Parser::new();
            parser_clone
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .map_err(|e| Error::TreeSitter(e.to_string()))?;

            let tree = parser_clone.parse(temp_source.as_bytes(), None);

            if let Some(tree) = tree {
                let root_node = tree.root_node();
                println!("DEBUG: Root node kind: {}", root_node.kind());

                // Process all direct children
                let mut cursor = root_node.walk();
                for node in root_node.children(&mut cursor) {
                    println!("DEBUG: Child node kind: {}", node.kind());

                    if node.kind() == "block" {
                        // We got the block, now parse its children
                        let mut block_cursor = node.walk();
                        for item in node.children(&mut block_cursor) {
                            match item.kind() {
                                "struct_item" => {
                                    println!("DEBUG: Found struct in module body");
                                    if let Ok(struct_def) = self.parse_struct(item, &temp_source) {
                                        module.structs.push(struct_def);
                                    }
                                }
                                "trait_item" => {
                                    println!("DEBUG: Found trait in module body");
                                    if let Ok(trait_def) = self.parse_trait(item, &temp_source) {
                                        module.traits.push(trait_def);
                                    }
                                }
                                "impl_item" => {
                                    println!("DEBUG: Found impl in module body");
                                    if let Ok(impl_def) = self.parse_impl(item, &temp_source) {
                                        module.impls.push(impl_def);
                                    }
                                }
                                "function_item" => {
                                    println!("DEBUG: Found function in module body");
                                    if let Ok(function) = self.parse_function(item, &temp_source) {
                                        module.functions.push(function);
                                    }
                                }
                                "enum_item" => {
                                    println!("DEBUG: Found enum in module body");
                                    if let Ok(enum_text) = item.utf8_text(temp_source.as_bytes()) {
                                        let enum_visibility =
                                            self.determine_visibility(item, &temp_source);
                                        let enum_documentation =
                                            self.extract_documentation(item, &temp_source);

                                        // Extract enum name
                                        let mut enum_name = "unknown".to_string();
                                        let mut enum_cursor = item.walk();
                                        for part in item.children(&mut enum_cursor) {
                                            if part.kind() == "type_identifier" {
                                                if let Ok(name) =
                                                    part.utf8_text(temp_source.as_bytes())
                                                {
                                                    enum_name = name.to_string();
                                                    break;
                                                }
                                            }
                                        }

                                        // Create a struct representation of the enum
                                        let enum_struct = crate::StructUnit {
                                            name: enum_name,
                                            visibility: enum_visibility,
                                            documentation: enum_documentation,
                                            fields: Vec::new(),
                                            methods: Vec::new(),
                                            source: Some(enum_text.to_string()),
                                            attributes: Vec::new(),
                                        };

                                        module.structs.push(enum_struct);
                                    }
                                }
                                "attribute_item" => {
                                    // Skip attribute items, they're already handled with their associated items
                                    continue;
                                }
                                "comment" | "line_comment" => {
                                    // Skip standalone comments
                                    continue;
                                }
                                _ => {
                                    println!(
                                        "DEBUG: Unhandled node in module body: {}",
                                        item.kind()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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

        // Debug: Print all top-level node kinds
        println!("Debug: Top-level node kinds in file:");
        let mut debug_cursor = root_node.walk();
        for node in root_node.children(&mut debug_cursor) {
            println!("  - {}", node.kind());
        }

        // Initialize file unit
        let mut file_unit = FileUnit {
            path: file_path.to_path_buf(),
            document: None,
            declares: Vec::new(),
            ..Default::default()
        };

        // Extract module-level documentation from the first comments
        let mut first_line_comments = Vec::new();
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            if node.kind() == "line_comment" {
                if let Ok(comment) = node.utf8_text(source_code.as_bytes()) {
                    if comment.starts_with("//!") {
                        let cleaned = comment.trim_start_matches("//!").trim().to_string();
                        first_line_comments.push(cleaned);
                    }
                }
            } else {
                break; // Stop after first non-comment
            }
        }

        if !first_line_comments.is_empty() {
            file_unit.document = Some(first_line_comments.join("\n"));
        }

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
                "struct_item" => {
                    let struct_def = self.parse_struct(node, &source_code)?;
                    file_unit.structs.push(struct_def);
                }
                "trait_item" => {
                    let trait_def = self.parse_trait(node, &source_code)?;
                    file_unit.traits.push(trait_def);
                }
                "impl_item" => {
                    let impl_def = self.parse_impl(node, &source_code)?;
                    file_unit.impls.push(impl_def);
                }
                "use_declaration" => {
                    if let Ok(use_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: use_text.to_string(),
                            kind: crate::DeclareKind::Use,
                        });
                    }
                }
                "mod_declaration" => {
                    if let Ok(mod_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: mod_text.to_string(),
                            kind: crate::DeclareKind::Mod,
                        });
                    }
                }
                "type_item" => {
                    if let Ok(type_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: type_text.to_string(),
                            kind: crate::DeclareKind::Other("type".to_string()),
                        });
                    }
                }
                "const_item" => {
                    if let Ok(const_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: const_text.to_string(),
                            kind: crate::DeclareKind::Other("const".to_string()),
                        });
                    }
                }
                "static_item" => {
                    if let Ok(static_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: static_text.to_string(),
                            kind: crate::DeclareKind::Other("static".to_string()),
                        });
                    }
                }
                "macro_definition" => {
                    if let Ok(macro_text) = node.utf8_text(source_code.as_bytes()) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: macro_text.to_string(),
                            kind: crate::DeclareKind::Other("macro".to_string()),
                        });
                    }
                }
                "line_comment" => {
                    // Already handled for document extraction
                    continue;
                }
                "attribute_item" => {
                    // Attributes are parsed along with their associated items
                    continue;
                }
                _ => {
                    // Debug: Print unhandled node kinds
                    println!("  Unhandled node kind: {}", node.kind());
                    continue;
                }
            }
        }

        // Now parse module contents with a different approach
        for module in &mut file_unit.modules {
            if let Err(e) = self.parse_module_content(module, &source_code) {
                println!("DEBUG: Error parsing module content: {:?}", e);
            }
        }

        // Set the source code for the whole file
        file_unit.source = Some(source_code);

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

        // Verify source code is captured
        assert!(file_unit.source.is_some(), "File source was not captured");

        // Verify document is captured from the module comments
        assert!(
            file_unit.document.is_some(),
            "File document was not captured"
        );
        println!("Document: {}", file_unit.document.as_ref().unwrap());

        // Verify declarations
        assert!(
            !file_unit.declares.is_empty(),
            "No declarations were parsed"
        );
        println!("Declarations found: {}", file_unit.declares.len());
        for (i, decl) in file_unit.declares.iter().enumerate() {
            println!(
                "Declaration {}: kind={:?}, source={}",
                i, decl.kind, decl.source
            );
        }

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
                "Module {}: name={}, visibility={:?}, has_docs={}, has_document={}",
                i,
                module.name,
                module.visibility,
                module.documentation.is_some(),
                module.document.is_some()
            );

            // Check module contents
            println!("  Module {} structs: {}", module.name, module.structs.len());
            for (j, struct_unit) in module.structs.iter().enumerate() {
                println!(
                    "    Struct {}.{}: name={}, visibility={:?}, has_docs={}",
                    i,
                    j,
                    struct_unit.name,
                    struct_unit.visibility,
                    struct_unit.documentation.is_some()
                );
            }

            println!("  Module {} traits: {}", module.name, module.traits.len());
            for (j, trait_unit) in module.traits.iter().enumerate() {
                println!(
                    "    Trait {}.{}: name={}, visibility={:?}, has_docs={}",
                    i,
                    j,
                    trait_unit.name,
                    trait_unit.visibility,
                    trait_unit.documentation.is_some()
                );
            }

            println!("  Module {} impls: {}", module.name, module.impls.len());
            for (j, impl_unit) in module.impls.iter().enumerate() {
                println!(
                    "    Impl {}.{}: target={}, trait={:?}, methods={}",
                    i,
                    j,
                    impl_unit.target_type,
                    impl_unit.trait_name,
                    impl_unit.methods.len()
                );
            }

            // Check that document field matches documentation
            assert_eq!(
                module.document, module.documentation,
                "Module document and documentation should match"
            );

            // Check submodules
            if !module.submodules.is_empty() {
                println!(
                    "  Submodules found in {}: {}",
                    module.name,
                    module.submodules.len()
                );
                for (j, submodule) in module.submodules.iter().enumerate() {
                    println!(
                        "  Submodule {}.{}: name={}, visibility={:?}, has_docs={}",
                        i,
                        j,
                        submodule.name,
                        submodule.visibility,
                        submodule.documentation.is_some()
                    );

                    // Check that document field matches documentation
                    assert_eq!(
                        submodule.document, submodule.documentation,
                        "Submodule document and documentation should match"
                    );
                }
            }
        }

        // Check that we have some content extracted
        assert!(!file_unit.functions.is_empty(), "No functions were parsed");
        assert!(!file_unit.modules.is_empty(), "No modules were parsed");

        // Test for specific elements
        assert!(
            file_unit
                .functions
                .iter()
                .any(|f| f.name == "public_function" && f.visibility == Visibility::Public),
            "Public function not found"
        );

        assert!(
            file_unit
                .functions
                .iter()
                .any(|f| f.name == "private_function" && f.visibility == Visibility::Private),
            "Private function not found"
        );

        assert!(
            file_unit
                .modules
                .iter()
                .any(|m| m.name == "public_module" && m.visibility == Visibility::Public),
            "Public module not found"
        );

        assert!(
            file_unit
                .modules
                .iter()
                .any(|m| m.name == "private_module" && m.visibility == Visibility::Private),
            "Private module not found"
        );

        // Test public_module contents
        let public_module = file_unit
            .modules
            .iter()
            .find(|m| m.name == "public_module")
            .expect("Public module not found");

        // Verify structs in public_module
        assert!(
            public_module
                .structs
                .iter()
                .any(|s| s.name == "PublicStruct" && s.visibility == Visibility::Public),
            "PublicStruct not found in public_module"
        );

        // Verify traits in public_module
        assert!(
            public_module
                .traits
                .iter()
                .any(|t| t.name == "PublicTrait" && t.visibility == Visibility::Public),
            "PublicTrait not found in public_module"
        );

        // Verify impls in public_module (at least one impl for PublicStruct)
        assert!(
            public_module
                .impls
                .iter()
                .any(|i| i.target_type == "PublicStruct"),
            "Implementation for PublicStruct not found in public_module"
        );

        // Check for trait implementation
        let trait_impl = public_module.impls.iter().find(|i| {
            i.trait_name.as_ref().map_or(false, |t| t == "PublicTrait")
                && i.target_type == "PublicStruct"
        });
        assert!(
            trait_impl.is_some(),
            "Implementation of PublicTrait for PublicStruct not found"
        );

        // Check for declares in sample file
        let type_declares = file_unit
            .declares
            .iter()
            .filter(|d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "type"))
            .count();
        assert!(type_declares > 0, "No type declarations found");

        let const_declares = file_unit
            .declares
            .iter()
            .filter(|d| matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "const"))
            .count();
        assert!(const_declares > 0, "No const declarations found");

        // Check for 'use' declarations in modules with declares
        for module in &file_unit.modules {
            println!("Module {} declares: {}", module.name, module.declares.len());
        }
    }
}
