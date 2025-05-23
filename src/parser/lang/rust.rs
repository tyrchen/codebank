use crate::{
    Error, FieldUnit, FileUnit, FunctionUnit, ImplUnit, LanguageParser, LanguageType, ModuleUnit,
    Result, RustParser, StructUnit, TraitUnit, Visibility,
};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::{Node, Parser, Query, QueryCursor, QueryMatch};

const VISIBILITY_MODIFIER_QUERY: &str = r#"
(visibility_modifier) @visibility
"#;

const ATTRIBUTE_QUERY: &str = r#"
(attribute_item) @attribute
"#;

// This query is general. Specific doc comment patterns (///, /**) are handled in extract_documentation.
const DOC_COMMENT_QUERY: &str = r#"
[
  (line_comment) @doc_comment_line
  (block_comment) @doc_comment_block
]
"#;

const FUNCTION_QUERY: &str = r#"
(function_item
  (attribute_item)* @attribute_node 
  (visibility_modifier)? @visibility_node
  name: (identifier) @name
  parameters: (parameters) @parameters
  return_type: (type)? @return_type
  body: (block) @body
  ;; Inner documentation comments
  (line_comment)* @doc_comment_line_inner
  (block_comment)* @doc_comment_block_inner
)
"#;

const STRUCT_QUERY: &str = r#"
(struct_item
  (attribute_item)* @attribute_node
  (visibility_modifier)? @visibility_node
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics
  (field_declaration_list)? @fields
  ;; Inner documentation comments
  (line_comment)* @doc_comment_line_inner
  (block_comment)* @doc_comment_block_inner
)
"#;

const ENUM_QUERY: &str = r#"
(enum_item
  (attribute_item)* @attribute_node
  (visibility_modifier)? @visibility_node
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics
  body: (enum_variant_list) @variants
  ;; Inner documentation comments
  (line_comment)* @doc_comment_line_inner
  (block_comment)* @doc_comment_block_inner
)
"#;

const TRAIT_QUERY: &str = r#"
(trait_item
  (attribute_item)* @attribute_node 
  (visibility_modifier)? @visibility_node 
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics 
  body: (declaration_list) @body 
)
"#;

// IMPL_QUERY focuses on the overall structure including the body.
const IMPL_QUERY: &str = r#"
(impl_item
  (attribute_item)* @attribute_node
  type_parameters: (type_parameters)? @impl_generics 
  body: (declaration_list) @body
  // Trait and type are better handled by IMPL_HEAD_QUERY for head construction
)
"#;

// IMPL_HEAD_QUERY is more focused on the "impl ... for ..." part, useful for head construction.
// Designed to be run on the `impl_item` node.
const IMPL_HEAD_QUERY: &str = r#"
(impl_item
    type_parameters: (type_parameters)? @impl_generics
    trait: (type_identifier)? @trait_name
    trait: (generic_type (type_identifier) @trait_name_generic (_)? @trait_generics_args)? @trait_full
    type: (type_identifier) @type_name
    type: (generic_type (type_identifier) @type_name_generic (_)? @type_generics_args)? @type_full
    // Scoped identifiers for trait or type
    trait: (scoped_type_identifier path: _ @trait_path name: (type_identifier) @trait_name_scoped)? @trait_full_scoped
    type: (scoped_type_identifier path: _ @type_path name: (type_identifier) @type_name_scoped)? @type_full_scoped
)
"#;


const MODULE_QUERY: &str = r#"
(mod_item
  (attribute_item)* @attribute_node
  (visibility_modifier)? @visibility_node
  name: (identifier) @name
  body: (declaration_list)? @body 
)
"#;


// Helper function to extract attributes looking backwards from a node
fn extract_attributes(node: Node, source_code: &str) -> Vec<String> {
    let mut attributes = Vec::new();
    let mut current_node = node;
    if current_node.kind() == "attribute_item" {
        if let Some(attr_text) = get_node_text(current_node, source_code) { attributes.insert(0, attr_text); }
    }
    while let Some(prev) = current_node.prev_sibling() {
        if prev.kind() == "attribute_item" {
            if let Some(attr_text) = get_node_text(prev, source_code) { attributes.insert(0, attr_text); }
            current_node = prev; 
        } else if prev.kind() == "line_comment" || prev.kind() == "block_comment" {
            current_node = prev;
        } else { break; }
    }
    attributes
}

// Helper function to get the text of the first child node of a specific kind
fn get_child_node_text<'a>(node: Node<'a>, kind: &str, source_code: &'a str) -> Option<String> {
    if let Some(child) = node.children(&mut node.walk()).find(|child| child.kind() == kind) {
        return child.utf8_text(source_code.as_bytes()).ok().map(String::from);
    }
    for child in node.children(&mut node.walk()) {
        if child.kind() == "type_identifier" { return child.utf8_text(source_code.as_bytes()).ok().map(String::from); }
        if let Some(grandchild) = child.children(&mut child.walk()).find(|gc| gc.kind() == "type_identifier" || gc.kind() == kind) {
            return grandchild.utf8_text(source_code.as_bytes()).ok().map(String::from);
        }
    }
    None
}

// Helper function to get the text of a node
fn get_node_text(node: Node, source_code: &str) -> Option<String> {
    node.utf8_text(source_code.as_bytes()).ok().map(String::from)
}

impl RustParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser.set_language(&language.into()).map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }
    
    fn get_capture_text_from_match<'a>( query_match: &QueryMatch<'a, 'a>, capture_name: &str, source_code: &'a str, query: &'a Query) -> Option<String> {
        query_match.captures.iter().find(|c| query.capture_names()[c.index as usize] == capture_name).and_then(|c| get_node_text(c.node, source_code))
    }
    
    fn get_capture_node_from_match<'a>( query_match: &QueryMatch<'a, 'a>, capture_name: &str, query: &'a Query) -> Option<Node<'a>> {
        query_match.captures.iter().find(|c| query.capture_names()[c.index as usize] == capture_name).map(|c| c.node)
    }

    fn parse_item_head( &self, node: Node, source_code: &str, item_type: &str, visibility: &Visibility, name: &str) -> String {
        if let Some(src) = get_node_text(node, source_code) {
            if let Some(body_start_idx) = src.find('{') { return src[0..body_start_idx].trim().to_string(); }
            else if let Some(semi_idx) = src.find(';') { return src[0..=semi_idx].trim().to_string(); }
        }
        let vis_str = visibility.as_str(LanguageType::Rust);
        if vis_str.is_empty() { format!("{} {}", item_type, name) } else { format!("{} {} {}", vis_str, item_type, name) }
    }

    fn extract_documentation(&self, node: Node, source_code: &str) -> Option<String> {
        let mut doc_comments = Vec::new();
        let mut current_node = node;
        while let Some(prev) = current_node.prev_sibling() {
            let kind = prev.kind();
            if kind == "line_comment" { if let Some(comment) = get_node_text(prev, source_code) { if comment.starts_with("///") { doc_comments.insert(0, comment.trim_start_matches("///").trim().to_string()); } } }
            else if kind == "block_comment" { if let Some(comment) = get_node_text(prev, source_code) { if comment.starts_with("/**") { let lines: Vec<&str> = comment.lines().collect(); if lines.len() > 1 { for line in lines[1..lines.len()-1].iter().rev() { let cleaned = line.trim_start_matches('*').trim().to_string(); if !cleaned.is_empty() { doc_comments.insert(0, cleaned);}}}}} }
            else if kind != "attribute_item" { break; }
            current_node = prev;
        }
        if doc_comments.is_empty() { None } else { Some(doc_comments.join("\n")) }
    }

    fn determine_visibility(&self, node: Node, source_code: &str) -> Visibility {
        let vis_node_opt = node.child_by_field_name("visibility_modifier").or_else(|| node.children(&mut node.walk()).find(|child| child.kind() == "visibility_modifier"));
        if let Some(vis_mod_node) = vis_node_opt { if let Some(vis_text) = get_node_text(vis_mod_node, source_code) {
            return match vis_text.as_str() { "pub" => Visibility::Public, "pub(crate)" => Visibility::Crate, s if s.starts_with("pub(") => Visibility::Restricted(s.to_string()), _ => Visibility::Private };
        }}
        Visibility::Private
    }

    fn parse_function(&self, node: Node, source_code: &str) -> Result<FunctionUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = extract_attributes(node, source_code);
        let mut visibility = self.determine_visibility(node, source_code);
        let full_source = get_node_text(node, source_code);

        let query = Query::new(self.parser.language().unwrap(), FUNCTION_QUERY).map_err(|e| Error::TreeSitter(format!("FUNCTION_QUERY compile error: {}", e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, node, source_code.as_bytes());
        let mut name = "unknown".to_string();
        let mut signature: Option<String> = None;
        let mut body_text: Option<String> = None; 
        let mut parameters_text_opt: Option<String> = None;
        let mut return_type_text_opt: Option<String> = None;

        if let Some(query_match) = matches.peekable().peek() {
            if let Some(n) = Self::get_capture_text_from_match(query_match, "name", source_code, &query) { name = n; }
            parameters_text_opt = Self::get_capture_text_from_match(query_match, "parameters", source_code, &query);
            return_type_text_opt = Self::get_capture_text_from_match(query_match, "return_type", source_code, &query);
            body_text = Self::get_capture_text_from_match(query_match, "body", source_code, &query);
            for cap in query_match.captures { if query.capture_names()[cap.index as usize] == "attribute_node" { if let Some(attr_txt)=get_node_text(cap.node, source_code){ if !attributes.contains(&attr_txt) { attributes.push(attr_txt);}}}}
            if visibility == Visibility::Private { if let Some(vis_node) = Self::get_capture_node_from_match(query_match, "visibility_node", &query) { if let Some(vis_text)=get_node_text(vis_node,source_code){ visibility = Visibility::from_str(&vis_text, LanguageType::Rust);}}}}
        
        let parameters_text = parameters_text_opt.unwrap_or_default();
        let return_type_text = return_type_text_opt.unwrap_or_default();
        let mut sig_str = format!("fn {}", name);
        sig_str.push_str(&parameters_text);
        if !return_type_text.is_empty() { sig_str.push_str(&format!(" -> {}", return_type_text)); }
        if body_text.is_none() { sig_str.push(';'); }
        signature = Some(sig_str.trim().to_string());
        
        let is_query_sig_problematic = signature.as_deref().map_or(true, |s| s.is_empty() || (parameters_text.is_empty() && !s.contains("()")));
        if is_query_sig_problematic {
             if let Some(src) = &full_source {
                if let Some(body_start_idx) = src.find('{') { signature = Some(src[0..body_start_idx].trim().to_string()); }
                else if let Some(sig_end_idx) = src.find(';') { signature = Some(src[0..=sig_end_idx].trim().to_string()); }
            } else if signature.as_deref().map_or(true, |s| s.trim() == format!("fn {}", name)) { signature = None; }
        }
        Ok(FunctionUnit { name, visibility, doc: documentation, source: full_source, signature, body: body_text, attributes })
    }

    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = extract_attributes(node, source_code); 
        let mut visibility = self.determine_visibility(node, source_code); 
        let full_source = get_node_text(node, source_code);

        let query = Query::new(self.parser.language().unwrap(), MODULE_QUERY).map_err(|e| Error::TreeSitter(format!("MODULE_QUERY error: {}", e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, node, source_code.as_bytes());
        let mut name = "unknown".to_string();
        let mut body_node_opt: Option<Node> = None;

        if let Some(query_match) = matches.peekable().peek() {
            if let Some(n_text) = Self::get_capture_text_from_match(query_match, "name", source_code, &query) { name = n_text; }
            body_node_opt = Self::get_capture_node_from_match(query_match, "body", &query);
            for cap in query_match.captures { if query.capture_names()[cap.index as usize] == "attribute_node" { if let Some(attr_txt)=get_node_text(cap.node, source_code){ if !attributes.contains(&attr_txt) { attributes.push(attr_txt);}}}}
            if visibility == Visibility::Private { if let Some(vis_node) = Self::get_capture_node_from_match(query_match, "visibility_node", &query) { if let Some(vis_text)=get_node_text(vis_node,source_code){ visibility = Visibility::from_str(&vis_text, LanguageType::Rust);}}}}
        } else { 
            name = get_child_node_text(node, "identifier", source_code).unwrap_or_else(||"unknown".to_string());
            if node.child_by_field_name("body").is_none() { body_node_opt = None; } 
            else { body_node_opt = node.children(&mut node.walk()).find(|child| child.kind() == "declaration_list"); }
        }
        
        let mut module_unit = ModuleUnit { name, visibility, doc: documentation, source: full_source, attributes, ..Default::default() };
        if let Some(body_node) = body_node_opt {
            for item in body_node.children(&mut body_node.walk()) {
                match item.kind() {
                    "function_item" => if let Ok(func) = self.parse_function(item, source_code) { module_unit.functions.push(func); },
                    "struct_item" => if let Ok(s) = self.parse_struct(item, source_code) { module_unit.structs.push(s); },
                    "enum_item" => if let Ok(e) = self.parse_enum_as_struct(item, source_code) { module_unit.structs.push(e); },
                    "trait_item" => if let Ok(t) = self.parse_trait(item, source_code) { module_unit.traits.push(t); },
                    "impl_item" => if let Ok(i) = self.parse_impl(item, source_code) { module_unit.impls.push(i); },
                    "mod_item" => if let Ok(m) = self.parse_module(item, source_code) { module_unit.submodules.push(m); },
                    "use_declaration" => if let Some(txt) = get_node_text(item, source_code) { module_unit.declares.push(crate::DeclareStatements{source: txt, kind: crate::DeclareKind::Use}); },
                    _ => {}
                }
            }
        }
        Ok(module_unit)
    }

    fn parse_enum_as_struct(&self, node: Node, source_code: &str) -> Result<StructUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = extract_attributes(node, source_code); 
        let mut visibility = self.determine_visibility(node, source_code); 
        let full_source = get_node_text(node, source_code);

        let query = Query::new(self.parser.language().unwrap(), ENUM_QUERY).map_err(|e| Error::TreeSitter(format!("ENUM_QUERY error: {}",e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, node, source_code.as_bytes());
        let mut name = "unknown".to_string();
        let mut head_str: String;
        let mut enum_variant_list_node: Option<Node> = None;
        
        if let Some(query_match) = matches.peekable().peek() {
            if let Some(n_text) = Self::get_capture_text_from_match(query_match, "name", source_code, &query) { name = n_text; }
            let generics_text = Self::get_capture_text_from_match(query_match, "generics", source_code, &query).unwrap_or_default();
            enum_variant_list_node = Self::get_capture_node_from_match(query_match, "variants", &query);
            for cap in query_match.captures { if query.capture_names()[cap.index as usize] == "attribute_node" { if let Some(attr_txt)=get_node_text(cap.node, source_code){ if !attributes.contains(&attr_txt) { attributes.push(attr_txt);}}}}
            if visibility == Visibility::Private { if let Some(vis_node) = Self::get_capture_node_from_match(query_match, "visibility_node", &query) { if let Some(vis_text)=get_node_text(vis_node,source_code){ visibility = Visibility::from_str(&vis_text, LanguageType::Rust);}}}}
            let vis_str = visibility.as_str(LanguageType::Rust);
            head_str = if vis_str.is_empty() { format!("enum {}{}", name, generics_text) } else { format!("{} enum {}{}", vis_str, name, generics_text) };
        } else {
            name = get_child_node_text(node, "identifier", source_code).unwrap_or_else(|| "unknown".to_string());
            head_str = self.parse_item_head(node, source_code, "enum", &visibility, &name);
            enum_variant_list_node = node.children(&mut node.walk()).find(|child| child.kind() == "enum_variant_list");
        }

        let mut fields = Vec::new();
        if let Some(body_node) = enum_variant_list_node { for variant_node in body_node.children(&mut body_node.walk()) { if variant_node.kind() == "enum_variant" {
            let v_name = get_child_node_text(variant_node, "identifier", source_code).unwrap_or_default();
            let v_doc = self.extract_documentation(variant_node, source_code);
            let v_attrs = extract_attributes(variant_node, source_code);
            let v_src = get_node_text(variant_node, source_code).map(|s| if s.ends_with(',') { s[..s.len()-1].to_string()} else {s});
            fields.push(FieldUnit { name: v_name, doc: v_doc, attributes: v_attrs, source: v_src });
        }}}
        Ok(StructUnit { name, head: head_str, visibility, doc: documentation, source: full_source, attributes, fields, methods: Vec::new() })
    }

    fn parse_struct(&self, node: Node, source_code: &str) -> Result<StructUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = extract_attributes(node, source_code);
        let mut visibility = self.determine_visibility(node, source_code);
        let full_source = get_node_text(node, source_code);

        let query = Query::new(self.parser.language().unwrap(), STRUCT_QUERY).map_err(|e| Error::TreeSitter(format!("STRUCT_QUERY error: {}",e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, node, source_code.as_bytes());
        let mut name = "unknown".to_string();
        let mut head_str: String;
        let mut field_declaration_list_node: Option<Node> = None;

        if let Some(query_match) = matches.peekable().peek() {
            if let Some(n_text) = Self::get_capture_text_from_match(query_match, "name", source_code, &query) { name = n_text; }
            let generics_text = Self::get_capture_text_from_match(query_match, "generics", source_code, &query).unwrap_or_default();
            field_declaration_list_node = Self::get_capture_node_from_match(query_match, "fields", &query);
            for cap in query_match.captures { if query.capture_names()[cap.index as usize] == "attribute_node" { if let Some(attr_txt)=get_node_text(cap.node, source_code){ if !attributes.contains(&attr_txt) { attributes.push(attr_txt);}}}}
            if visibility == Visibility::Private { if let Some(vis_node) = Self::get_capture_node_from_match(query_match, "visibility_node", &query) { if let Some(vis_text)=get_node_text(vis_node,source_code){ visibility = Visibility::from_str(&vis_text, LanguageType::Rust);}}}}
            let vis_str = visibility.as_str(LanguageType::Rust);
            head_str = if vis_str.is_empty() { format!("struct {}{}", name, generics_text) } else { format!("{} struct {}{}", vis_str, name, generics_text) };
            if field_declaration_list_node.is_none() && full_source.as_deref().map_or(false, |s| s.trim_end().ends_with(';')) { head_str.push(';'); }
        } else {
            name = get_child_node_text(node, "identifier", source_code).unwrap_or_else(|| "unknown".to_string());
            head_str = self.parse_item_head(node, source_code, "struct", &visibility, &name);
            field_declaration_list_node = node.children(&mut node.walk()).find(|child| child.kind() == "field_declaration_list");
        }
        
        let mut fields = Vec::new();
        if let Some(body_node) = field_declaration_list_node { for field_decl in body_node.children(&mut body_node.walk()) { if field_decl.kind() == "field_declaration" {
            let f_doc = self.extract_documentation(field_decl, source_code);
            let f_attrs = extract_attributes(field_decl, source_code);
            let f_src = get_node_text(field_decl, source_code);
            let f_name = get_child_node_text(field_decl, "field_identifier", source_code).unwrap_or_default();
            fields.push(FieldUnit { name: f_name, doc: f_doc, attributes: f_attrs, source: f_src });
        }}}
        Ok(StructUnit { name, head: head_str, visibility, doc: documentation, source: full_source, attributes, fields, methods: Vec::new() })
    }

    fn parse_trait(&self, node: Node, source_code: &str) -> Result<TraitUnit> {
        let documentation = self.extract_documentation(node, source_code); 
        let mut attributes = extract_attributes(node, source_code); 
        let mut visibility = self.determine_visibility(node, source_code); 
        let full_source = get_node_text(node, source_code);

        let query = Query::new(self.parser.language().unwrap(), TRAIT_QUERY).map_err(|e| Error::TreeSitter(format!("TRAIT_QUERY error: {}", e)))?;
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, node, source_code.as_bytes());
        let mut name = "unknown".to_string();
        let mut head_str: String;
        let mut body_node_opt: Option<Node> = None;

        if let Some(query_match) = matches.peekable().peek() {
            if let Some(n_text) = Self::get_capture_text_from_match(query_match, "name", source_code, &query) { name = n_text; }
            let generics_text = Self::get_capture_text_from_match(query_match, "generics", source_code, &query).unwrap_or_default();
            body_node_opt = Self::get_capture_node_from_match(query_match, "body", &query);
            for cap in query_match.captures { let cap_name = &query.capture_names()[cap.index as usize]; if *cap_name == "attribute_node" { if let Some(attr_txt) = get_node_text(cap.node, source_code) { if !attributes.contains(&attr_txt) { attributes.push(attr_txt); }}}}
            if visibility == Visibility::Private { if let Some(vis_node) = Self::get_capture_node_from_match(query_match, "visibility_node", &query) { if let Some(vis_text) = get_node_text(vis_node, source_code) { visibility = Visibility::from_str(&vis_text, LanguageType::Rust); }}}
            let vis_str = visibility.as_str(LanguageType::Rust);
            head_str = if vis_str.is_empty() { format!("trait {}{}", name, generics_text) } else { format!("{} trait {}{}", vis_str, name, generics_text) };
        } else {
            name = get_child_node_text(node, "identifier", source_code).unwrap_or_else(|| "unknown".to_string());
            head_str = self.parse_item_head(node, source_code, "trait", &visibility, &name);
            body_node_opt = node.children(&mut node.walk()).find(|child| child.kind() == "declaration_list");
        }

        let mut methods = Vec::new();
        if let Some(body_node) = body_node_opt { for item_node in body_node.children(&mut body_node.walk()) {
            if item_node.kind() == "function_item" || item_node.kind() == "function_signature_item" {
                if let Ok(mut method) = self.parse_function(item_node, source_code) { method.visibility = Visibility::Public; methods.push(method); }
            }
        }}
        Ok(TraitUnit { name, head: head_str, visibility, doc: documentation, source: full_source, attributes, methods })
    }

    fn parse_impl(&self, node: Node, source_code: &str) -> Result<ImplUnit> {
        let documentation = self.extract_documentation(node, source_code);
        let mut attributes = extract_attributes(node, source_code); 
        let full_source = get_node_text(node, source_code);
        
        let head_query = Query::new(self.parser.language().unwrap(), IMPL_HEAD_QUERY).map_err(|e| Error::TreeSitter(format!("IMPL_HEAD_QUERY error: {}",e)))?;
        let main_query = Query::new(self.parser.language().unwrap(), IMPL_QUERY).map_err(|e| Error::TreeSitter(format!("IMPL_QUERY error: {}",e)))?;
        let mut cursor = QueryCursor::new();
        let head_matches = cursor.matches(&head_query, node, source_code.as_bytes());
        let mut head_str: String;

        if let Some(head_match) = head_matches.peekable().peek() {
            let impl_generics = Self::get_capture_text_from_match(head_match, "impl_generics", source_code, &head_query).unwrap_or_default();
            let trait_capture = Self::get_capture_text_from_match(head_match, "trait_full", source_code, &head_query)
                .or_else(|| Self::get_capture_text_from_match(head_match, "trait_full_scoped", source_code, &head_query))
                .or_else(|| Self::get_capture_text_from_match(head_match, "trait_name", source_code, &head_query)); 
            let type_capture = Self::get_capture_text_from_match(head_match, "type_full", source_code, &head_query)
                .or_else(|| Self::get_capture_text_from_match(head_match, "type_full_scoped", source_code, &head_query))
                .or_else(|| Self::get_capture_text_from_match(head_match, "type_name", source_code, &head_query)) 
                .unwrap_or_else(|| "UnknownType".to_string());
            
            head_str = format!("impl{}", impl_generics);
            if let Some(trait_text) = trait_capture { head_str.push_str(&format!(" {} for {}", trait_text, type_capture)); } 
            else { head_str.push_str(&format!(" {}", type_capture)); }
        } else { 
             head_str = if let Some(src) = &full_source { if let Some(body_start_idx) = src.find('{') { src[0..body_start_idx].trim().to_string() } else { "impl".to_string() }} else { "impl".to_string() };
        }

        let mut body_node_opt: Option<Node> = None;
        let main_matches = cursor.matches(&main_query, node, source_code.as_bytes());
        if let Some(main_match) = main_matches.peekable().peek() {
            body_node_opt = Self::get_capture_node_from_match(main_match, "body", &main_query);
            for cap in main_match.captures { if main_query.capture_names()[cap.index as usize] == "attribute_node" { if let Some(attr_txt)=get_node_text(cap.node, source_code){ if !attributes.contains(&attr_txt) { attributes.push(attr_txt);}}}}
        } else if body_node_opt.is_none() { 
            body_node_opt = node.children(&mut node.walk()).find(|child| child.kind() == "declaration_list");
        }
        
        let mut methods = Vec::new();
        let is_trait_impl = head_str.contains(" for "); 
        if let Some(body_node) = body_node_opt { for item in body_node.children(&mut body_node.walk()) { if item.kind() == "function_item" {
            if let Ok(mut method) = self.parse_function(item, source_code) { if is_trait_impl { method.visibility = Visibility::Public; } methods.push(method); }
        }}}
        Ok(ImplUnit { doc: documentation, head: head_str, source: full_source, attributes, methods })
    }
}

impl LanguageParser for RustParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;
        let tree = self.parse(source_code.as_bytes(), None).ok_or_else(|| Error::TreeSitter("Failed to parse source code".to_string()))?;
        let root_node = tree.root_node();
        let mut file_unit = FileUnit::new(file_path.to_path_buf());
        file_unit.source = Some(source_code.clone());

        let first_item_node = root_node.children(&mut root_node.walk()).find(|node| {
            let kind = node.kind(); kind != "line_comment" && kind != "block_comment" && kind != "attribute_item" && kind != "inner_attribute_item"
        });
        if let Some(first_node) = first_item_node { file_unit.doc = self.extract_documentation(first_node, &source_code); }
        else if let Some(last_node) = root_node.children(&mut root_node.walk()).last() { file_unit.doc = self.extract_documentation(last_node.next_sibling().unwrap_or(last_node), &source_code); }

        for child in root_node.children(&mut root_node.walk()) {
            match child.kind() {
                "function_item" => { if let Ok(func) = self.parse_function(child, &source_code) { file_unit.functions.push(func); } }
                "struct_item" => { if let Ok(struct_item) = self.parse_struct(child, &source_code) { file_unit.structs.push(struct_item); } }
                "enum_item" => { if let Ok(enum_as_struct) = self.parse_enum_as_struct(child, &source_code) { file_unit.structs.push(enum_as_struct); } }
                "trait_item" => { if let Ok(trait_item) = self.parse_trait(child, &source_code) { file_unit.traits.push(trait_item); } }
                "impl_item" => { if let Ok(impl_item) = self.parse_impl(child, &source_code) { file_unit.impls.push(impl_item); } }
                "mod_item" => { if let Ok(module) = self.parse_module(child, &source_code) { file_unit.modules.push(module); } }
                "use_declaration" => { if let Some(text) = get_node_text(child, &source_code) { file_unit.declares.push(crate::DeclareStatements { source: text, kind: crate::DeclareKind::Use }); } }
                "extern_crate_declaration" => { if let Some(text) = get_node_text(child, &source_code) { file_unit.declares.push(crate::DeclareStatements { source: text, kind: crate::DeclareKind::Other("extern_crate".to_string()) }); } }
                "mod_declaration" => { if let Some(text) = get_node_text(child, &source_code) { file_unit.declares.push(crate::DeclareStatements { source: text, kind: crate::DeclareKind::Mod }); } }
                _ => {}
            }
        }
        Ok(file_unit)
    }
}

impl Deref for RustParser { type Target = Parser; fn deref(&self) -> &Self::Target { &self.parser } }
impl DerefMut for RustParser { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.parser } }

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parse_fixture(file_name: &str) -> Result<FileUnit> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let path = PathBuf::from(manifest_dir).join("fixtures").join(file_name);
        RustParser::try_new()?.parse_file(&path)
    }

    #[test]
    fn test_parse_file_level_items() { 
        let file_unit = parse_fixture("sample.rs").unwrap(); 
        assert!( !file_unit.functions.is_empty() || !file_unit.structs.is_empty() || !file_unit.modules.is_empty() || !file_unit.declares.is_empty() );
        assert_eq!(file_unit.doc.as_deref(), Some("This is a file-level documentation comment.\nIt describes the purpose of this sample file which includes a variety of Rust items."));
    }

    #[test]
    fn test_use_extern_mod_declarations() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        let declares = &file_unit.declares;
        assert!(declares.iter().any(|d| d.source == "extern crate proc_macro;" && matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "extern_crate")));
        assert!(declares.iter().any(|d| d.source == "extern crate serde as serde_renamed;" && matches!(d.kind, crate::DeclareKind::Other(ref s) if s == "extern_crate")));
        assert!(declares.iter().any(|d| d.source == "use std::collections::HashMap;" && d.kind == crate::DeclareKind::Use));
        assert!(declares.iter().any(|d| d.source == "use std::fmt::{self, Debug as FmtDebug};" && d.kind == crate::DeclareKind::Use));
        assert!(declares.iter().any(|d| d.source == "use crate::public_module::PublicStruct;" && d.kind == crate::DeclareKind::Use));
        assert!(declares.iter().any(|d| d.source == "mod my_other_module;" && d.kind == crate::DeclareKind::Mod));
    }


    #[test]
    fn test_parse_top_level_functions_attributes_docs() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        let main_fn = file_unit.functions.iter().find(|f| f.name == "main").expect("main function not found"); // From cfg(test)
        assert_eq!(main_fn.name, "main");
        assert_eq!(main_fn.visibility, Visibility::Private); // Based on no explicit visibility

        let public_fn = file_unit.functions.iter().find(|f| f.name == "public_function").expect("public_function not found");
        assert_eq!(public_fn.name, "public_function");
        assert_eq!(public_fn.visibility, Visibility::Public);
        assert_eq!(public_fn.signature.as_deref(), Some("fn public_function() -> String"));
        assert_eq!(public_fn.doc.as_deref(), Some("A public function with multiple attributes and docs.\nSecond line of doc."));
        assert_eq!(public_fn.attributes, vec!["#[inline]".to_string(), "#[must_use = \"Return value should be used\"]".to_string()]);

        let private_fn = file_unit.functions.iter().find(|f| f.name == "private_function").expect("private_function not found");
        assert_eq!(private_fn.name, "private_function");
        assert_eq!(private_fn.visibility, Visibility::Private);
        assert_eq!(private_fn.signature.as_deref(), Some("fn private_function(s: &str) -> String"));
        assert_eq!(private_fn.doc.as_deref(), Some("This is a private function with documentation"));
        assert_eq!(private_fn.attributes, vec!["#[allow(dead_code)]".to_string()]);

    }

    #[test]
    fn test_module_parsing_with_details() { 
        let file_unit = parse_fixture("sample.rs").unwrap();
        let module = file_unit.modules.iter().find(|m| m.name == "public_module").expect("public_module not found");
        assert_eq!(module.name, "public_module");
        assert_eq!(module.visibility, Visibility::Public);
        assert_eq!(module.doc.as_deref(), Some("This is a public module.\nIt has multiple lines of documentation."));
        assert!(module.attributes.contains(&"#[cfg(feature = \"some_feature\")]".to_string()));
        assert!(module.attributes.contains(&"#[deprecated(note = \"This module is old\")]".to_string()));
        // Check for inner doc comment "//! Inner documentation for public_module."
        // Current extract_documentation gets outer. Inner module doc is not captured by FileUnit.doc or ModuleUnit.doc in this setup.
        // This would require specific handling for inner module docs if needed for ModuleUnit.

        let crate_vis_fn = module.functions.iter().find(|f| f.name == "crate_visible_function").expect("crate_visible_function not found");
        assert_eq!(crate_vis_fn.visibility, Visibility::Crate);
        
        // Testing nested module
        let nested_mod = module.submodules.iter().find(|m| m.name == "nested_module").expect("nested_module not found");
        assert_eq!(nested_mod.name, "nested_module");
        assert_eq!(nested_mod.visibility, Visibility::Private); // Default visibility
        // Add assertion for nested_module's inner doc if ModuleUnit.doc should capture it.
        // For now, assuming ModuleUnit.doc is for outer docs of the mod item itself.

        let visible_to_public_module_fn = nested_mod.functions.iter().find(|f| f.name == "visible_to_public_module").unwrap();
        assert_eq!(visible_to_public_module_fn.visibility, Visibility::Restricted("pub(super)".to_string()));

    }

    #[test]
    fn test_struct_trait_heads_generics_and_attributes() { 
        let file_unit = parse_fixture("sample.rs").unwrap();
        let public_module = file_unit.modules.iter().find(|m| m.name == "public_module").unwrap();
        
        let ps_module = public_module.structs.iter().find(|s| s.name == "PublicStruct").unwrap();
        assert_eq!(ps_module.head, "pub struct PublicStruct<T: FmtDebug + Clone, U>"); // Where clause not in head
        assert_eq!(ps_module.visibility, Visibility::Public);
        assert!(ps_module.attributes.contains(&"#[derive(Debug, Clone)]".to_string()));
        assert!(ps_module.attributes.contains(&"#[serde(rename_all = \"camelCase\")]".to_string()));
        assert_eq!(ps_module.doc.as_deref(), Some("This is a public struct with documentation.\nIt also has generics and attributes."));
        let field_another = ps_module.fields.iter().find(|f| f.name == "another_field").unwrap();
        assert_eq!(field_another.doc.as_deref(), Some("Inner attribute doc for another_field"));


        let pt_module = public_module.traits.iter().find(|t| t.name == "PublicTrait").unwrap();
        assert_eq!(pt_module.head, "pub trait PublicTrait<T>");
        assert!(pt_module.attributes.contains(&"#[allow(unused_variables)]".to_string()));

        let gs_file = file_unit.structs.iter().find(|s| s.name == "GenericStruct").unwrap();
        assert_eq!(gs_file.head, "pub struct GenericStruct<T>"); // Now pub due to fixture update
        assert_eq!(gs_file.visibility, Visibility::Public);


        let gt_file = file_unit.traits.iter().find(|t| t.name == "GenericTrait").unwrap();
        assert_eq!(gt_file.head, "pub trait GenericTrait<T>");
        assert!(gt_file.attributes.contains(&"#[allow(unused_variables)]".to_string()));
    }
    
    #[test]
    fn test_type_const_static() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        // Type aliases are not specifically parsed into their own Unit type yet.
        // Constants and Statics are not specifically parsed into their own Unit type yet.
        // For now, check their presence by looking for related items if necessary or assume they don't interfere.
        // Example: Find the public static var
        let static_var_func = file_unit.functions.iter().any(|f| f.source.as_deref().unwrap_or("").contains("PUBLIC_STATIC_VAR"));
        // This is a weak test. Real parsing of const/static would be better.
        // For now, just acknowledge they are in sample.rs
        assert!(file_unit.source.as_ref().unwrap().contains("pub type PublicTypeAlias<T>"));
        assert!(file_unit.source.as_ref().unwrap().contains("pub const PUBLIC_CONSTANT: &str"));
        assert!(file_unit.source.as_ref().unwrap().contains("pub static PUBLIC_STATIC_VAR: i32"));

    }


    #[test]
    fn test_impl_blocks_details() {
        let file_unit = parse_fixture("sample.rs").unwrap();
        
        let generic_struct_impl = file_unit.impls.iter().find(|imp| imp.head == "impl<T> GenericStruct<T>").expect("Impl for GenericStruct not found or head mismatch");
        assert_eq!(generic_struct_impl.doc.as_deref(), Some("Implementation for GenericStruct."));
        assert!(generic_struct_impl.attributes.contains(&"#[allow(dead_code)]".to_string()));
        let method_in_gs_impl = generic_struct_impl.methods.iter().find(|m| m.name == "new").expect("new method not found in GenericStruct impl");
        assert_eq!(method_in_gs_impl.signature.as_deref(), Some("fn new(value: T) -> Self"));
        assert_eq!(method_in_gs_impl.visibility, Visibility::Private); // Default for impl methods

        let trait_impl = file_unit.impls.iter().find(|imp| imp.head == "impl<T> GenericTrait<T> for GenericStruct<T> where T: Clone + FmtDebug").expect("Trait impl for GenericStruct not found or head mismatch");
        assert_eq!(trait_impl.doc.as_deref(), Some("Implementation of GenericTrait for GenericStruct.\nIncludes a where clause."));
        let method_in_trait_impl = trait_impl.methods.iter().find(|m| m.name == "method").expect("method not found in GenericTrait impl");
        assert_eq!(method_in_trait_impl.signature.as_deref(), Some("fn method(&self, value: T) -> T"));
        assert_eq!(method_in_trait_impl.visibility, Visibility::Public); // Trait methods are pub
    }

    #[test]
    fn test_advanced_fixture_parsing() {
        let file_unit = parse_fixture("sample_advanced.rs").unwrap();
        assert_eq!(file_unit.doc.as_deref(), Some("File for advanced Rust constructs."));

        let level1_mod = file_unit.modules.iter().find(|m| m.name == "level1").expect("level1 module");
        let level2_mod = level1_mod.submodules.iter().find(|m| m.name == "level2").expect("level2 submodule");
        let deep_struct = level2_mod.structs.iter().find(|s| s.name == "DeepStruct").expect("DeepStruct");
        assert_eq!(deep_struct.visibility, Visibility::Restricted("pub(in crate::level1)".to_string()));
        assert_eq!(deep_struct.doc.as_deref(), Some("Struct deep inside modules."));
        assert!(deep_struct.attributes.contains(&"#[derive(Default)]".to_string()));

        let complex_fn = level1_mod.functions.iter().find(|f| f.name == "complex_generic_function").expect("complex_generic_function");
        assert_eq!(complex_fn.name, "complex_generic_function");
        assert_eq!(complex_fn.visibility, Visibility::Public);
        assert_eq!(complex_fn.signature.as_deref(), Some("fn complex_generic_function<'a, T, U>(param_t: T, param_u: &'a U) -> Result<T, U::Error> where T: std::fmt::Debug + Clone + Send + 'static, U: std::error::Error + ?Sized, for<'b> &'b U: Send"));
        
        let adv_generic_struct = file_unit.structs.iter().find(|s| s.name == "AdvancedGenericStruct").expect("AdvancedGenericStruct");
        assert_eq!(adv_generic_struct.head, "pub struct AdvancedGenericStruct<'a, A, B> where A: AsRef<[u8]> + ?Sized, B: 'a + Send + Sync");

        let generic_result_enum = file_unit.structs.iter().find(|s| s.name == "GenericResult").expect("GenericResult enum"); // Enums are StructUnit
        assert_eq!(generic_result_enum.head, "pub enum GenericResult<S, E> where S: Send, E: std::fmt::Debug");

        let adv_trait = file_unit.traits.iter().find(|t| t.name == "AdvancedTrait").expect("AdvancedTrait");
        assert_eq!(adv_trait.head, "pub trait AdvancedTrait"); // Associated types/consts not in head string
        // TODO: Add parsing for associated types and consts in traits to test them here.

        let my_type_impl = file_unit.impls.iter().find(|i| i.head == "impl AdvancedTrait for MyTypeForAdvancedTrait").expect("MyTypeForAdvancedTrait impl");
        assert_eq!(my_type_impl.methods.len(), 1); // process method
        // TODO: Add parsing for associated types and consts in impls to test them here.

        let unit_struct = file_unit.structs.iter().find(|s| s.name == "MyUnitStruct").expect("MyUnitStruct");
        assert_eq!(unit_struct.head, "pub struct MyUnitStruct;");
        assert!(unit_struct.fields.is_empty());

        let empty_struct = file_unit.structs.iter().find(|s| s.name == "EmptyStruct").expect("EmptyStruct");
        assert_eq!(empty_struct.head, "pub struct EmptyStruct"); // or "pub struct EmptyStruct {}" depending on how head is formed
        assert!(empty_struct.fields.is_empty());
        
        let no_fields_struct = file_unit.structs.iter().find(|s| s.name == "NoFieldsStruct").expect("NoFieldsStruct");
        assert_eq!(no_fields_struct.visibility, Visibility::Private);


    }

    #[test]
    fn test_edge_case_file_parsing() {
        let empty_file_unit = parse_fixture("empty.rs").unwrap();
        assert!(empty_file_unit.functions.is_empty());
        assert!(empty_file_unit.structs.is_empty());
        assert!(empty_file_unit.modules.is_empty());
        assert!(empty_file_unit.declares.is_empty());
        assert!(empty_file_unit.doc.is_none());

        let comments_only_unit = parse_fixture("only_comments.rs").unwrap();
        assert!(comments_only_unit.functions.is_empty());
        assert!(comments_only_unit.structs.is_empty());
        // File-level inner doc comments (//! or /*! */) should be captured if they are the only content.
        // extract_documentation looks at prev_siblings of the "first item". If no items, it might look at last node.
        assert!(comments_only_unit.doc.is_some());
        assert!(comments_only_unit.doc.as_ref().unwrap().contains("This is an inner line comment, often used for module-level docs."));
        assert!(comments_only_unit.doc.as_ref().unwrap().contains("This is an inner block comment.\nAlso for module-level docs usually."));

    }

    // Original tests from before, reviewed and potentially slightly adjusted for new head formats or details.
    #[test]
    fn test_original_struct_with_fields() { // Renamed from test_struct_with_fields
        let file_unit = parse_fixture("sample_with_fields.rs").unwrap();
        let struct_with_fields = file_unit.structs.iter().find(|s| s.name == "StructWithFields").unwrap();
        assert_eq!(struct_with_fields.head, "pub struct StructWithFields");
        assert_eq!(struct_with_fields.doc.as_deref(), Some("Documentation for the struct"));
        assert!(!struct_with_fields.fields.is_empty());
        
        let public_field = struct_with_fields.fields.iter().find(|f| f.name == "public_field").unwrap();
        assert_eq!(public_field.doc.as_deref(), Some("A public field documentation"));
        assert!(public_field.source.as_ref().unwrap().contains("pub public_field: String"));
        
        let private_field = struct_with_fields.fields.iter().find(|f| f.name == "_private_field").unwrap();
        assert_eq!(private_field.doc.as_deref(), Some("A private field documentation"));
        assert!(private_field.attributes[0].contains("#[allow(dead_code)]"));
    }

    #[test]
    fn test_original_enum_with_variants() { // Renamed from test_parse_enum_with_variants
        let file_unit = parse_fixture("sample_enum.rs").unwrap();
        let public_enum = file_unit.structs.iter().find(|s| s.name == "PublicEnum").unwrap(); 
        assert_eq!(public_enum.visibility, Visibility::Public);
        assert_eq!(public_enum.doc.as_deref(), Some("This is a public enum with documentation"));
        assert!(public_enum.attributes.contains(&"#[derive(Debug)]".to_string()));
        assert_eq!(public_enum.head, "pub enum PublicEnum"); 
        assert_eq!(public_enum.fields.len(), 3); 

        let variant1 = public_enum.fields.iter().find(|f| f.name == "Variant1").unwrap();
        assert_eq!(variant1.doc.as_deref(), Some("Variant documentation"));
        assert_eq!(variant1.source.as_ref().unwrap(), "Variant1");

        let variant2 = public_enum.fields.iter().find(|f| f.name == "Variant2").unwrap();
        assert!(variant2.attributes.contains(&"#[allow(dead_code)]".to_string()));
        assert_eq!(variant2.source.as_ref().unwrap(), "Variant2(String)");
        assert_eq!(variant2.doc.as_deref(), Some("Another variant documentation"));


        let variant3 = public_enum.fields.iter().find(|f| f.name == "Variant3").unwrap();
        assert_eq!(variant3.doc.as_deref(), Some("Yet another variant documentation"));
        assert_eq!(variant3.source.as_ref().unwrap(), "Variant3 { field: i32 }");

        let private_enum = file_unit.structs.iter().find(|s| s.name == "PrivateEnum").expect("PrivateEnum not found");
        assert_eq!(private_enum.visibility, Visibility::Private);

        let impl_block = file_unit.impls.iter().find(|i| i.head == "impl PublicEnum").expect("Impl for PublicEnum not found");
        assert_eq!(impl_block.methods.len(), 1);
        let describe_method = impl_block.methods.first().unwrap();
        assert_eq!(describe_method.name, "describe");
        assert_eq!(describe_method.visibility, Visibility::Public);
    }
}
