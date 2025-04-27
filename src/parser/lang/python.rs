use crate::{
    Error, FileUnit, FunctionUnit, LanguageParser, ModuleUnit, PythonParser, Result, StructUnit,
    Visibility,
};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tree_sitter::{Node, Parser};

// Helper function to get the text of a node
fn get_node_text(node: Node, source_code: &str) -> Option<String> {
    node.utf8_text(source_code.as_bytes())
        .ok()
        .map(String::from)
}

// Helper function to get the text of the first child node of a specific kind
fn get_child_node_text<'a>(node: Node<'a>, kind: &str, source_code: &'a str) -> Option<String> {
    node.children(&mut node.walk())
        .find(|child| child.kind() == kind)
        .and_then(|child| child.utf8_text(source_code.as_bytes()).ok())
        .map(String::from)
}

impl PythonParser {
    pub fn try_new() -> Result<Self> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| Error::TreeSitter(e.to_string()))?;
        Ok(Self { parser })
    }

    // Extract docstring from a node
    fn extract_documentation(&self, node: Node, source_code: &str) -> Option<String> {
        let mut cursor = node.walk();
        let mut children = node.children(&mut cursor);

        // For function/class nodes, we need to skip the definition line
        if node.kind() == "function_definition" || node.kind() == "class_definition" {
            children.next(); // Skip the function/class definition line
        }

        // Look for the docstring
        for child in children {
            match child.kind() {
                "block" => {
                    // For function/class bodies, look in the block
                    let mut body_cursor = child.walk();
                    let mut body_children = child.children(&mut body_cursor);
                    if let Some(first_expr) = body_children.next() {
                        if first_expr.kind() == "expression_statement" {
                            if let Some(string) = first_expr
                                .children(&mut first_expr.walk())
                                .find(|c| c.kind() == "string")
                            {
                                return self.clean_docstring(string, source_code);
                            }
                        }
                    }
                }
                "expression_statement" => {
                    // For module level docstrings
                    if let Some(string) = child
                        .children(&mut child.walk())
                        .find(|c| c.kind() == "string")
                    {
                        return self.clean_docstring(string, source_code);
                    }
                }
                "ERROR" => {
                    // For ERROR nodes, try to get the string content directly
                    let mut error_cursor = child.walk();
                    let error_children = child.children(&mut error_cursor);
                    for error_child in error_children {
                        if error_child.kind() == "string" {
                            if let Some(string_content) = error_child
                                .children(&mut error_child.walk())
                                .find(|c| c.kind() == "string_content")
                            {
                                if let Some(content) = get_node_text(string_content, source_code) {
                                    return Some(content.trim().to_string());
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        None
    }

    // Helper to clean up docstring content
    fn clean_docstring(&self, node: Node, source_code: &str) -> Option<String> {
        let doc = get_node_text(node, source_code)?;
        // Clean up the docstring - handle both single and triple quotes
        let doc = if doc.starts_with("\"\"\"") && doc.ends_with("\"\"\"") {
            // Handle triple quotes
            doc[3..doc.len() - 3].trim()
        } else if doc.starts_with("'''") && doc.ends_with("'''") {
            // Handle triple single quotes
            doc[3..doc.len() - 3].trim()
        } else {
            // Handle single quotes
            doc.trim_matches('"').trim_matches('\'').trim()
        };
        Some(doc.to_string())
    }

    // Extract decorators from a node
    fn extract_decorators(&self, node: Node, source_code: &str) -> Vec<String> {
        let mut decorators = Vec::new();
        let mut cursor = node.walk();

        // Look for decorators before the function/class definition
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                if let Some(text) = get_node_text(child, source_code) {
                    decorators.push(text);
                }
            }
        }
        decorators
    }

    // Parse function and extract its details
    fn parse_function(&self, node: Node, source_code: &str) -> Result<FunctionUnit> {
        // If this is a decorated function, get the actual function definition
        let function_node = if node.kind() == "decorated_definition" {
            node.children(&mut node.walk())
                .find(|child| child.kind() == "function_definition")
                .unwrap_or(node)
        } else {
            node
        };

        let name = get_child_node_text(function_node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let documentation = self.extract_documentation(function_node, source_code);
        let attributes = self.extract_decorators(node, source_code);
        let source = get_node_text(function_node, source_code);
        let visibility = if name.starts_with('_') {
            Visibility::Private
        } else {
            Visibility::Public
        };

        let mut signature = None;
        let mut body = None;

        if let Some(src) = &source {
            if let Some(body_start_idx) = src.find(':') {
                signature = Some(src[0..body_start_idx].trim().to_string());
                body = Some(src[body_start_idx + 1..].trim().to_string());
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

    // Parse class and extract its details
    fn parse_class(&self, node: Node, source_code: &str) -> Result<StructUnit> {
        // If this is a decorated class, get the actual class definition
        let class_node = if node.kind() == "decorated_definition" {
            node.children(&mut node.walk())
                .find(|child| child.kind() == "class_definition")
                .unwrap_or(node)
        } else {
            node
        };

        let name = get_child_node_text(class_node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let documentation = self.extract_documentation(class_node, source_code);
        let attributes = self.extract_decorators(node, source_code);
        let source = get_node_text(class_node, source_code);
        let visibility = if name.starts_with('_') {
            Visibility::Private
        } else {
            Visibility::Public
        };

        // TODO: parse class head
        let head = format!("class {}", name);

        // Extract methods from class body
        let mut methods = Vec::new();
        let mut cursor = class_node.walk();
        for child in class_node.children(&mut cursor) {
            if child.kind() == "block" {
                let mut block_cursor = child.walk();
                for method_node in child.children(&mut block_cursor) {
                    match method_node.kind() {
                        "function_definition" | "decorated_definition" => {
                            if let Ok(method) = self.parse_function(method_node, source_code) {
                                methods.push(method);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }

        Ok(StructUnit {
            name,
            head,
            visibility,
            documentation,
            source,
            attributes,
            methods,
        })
    }

    #[allow(dead_code)]
    // Parse module and extract its details
    fn parse_module(&self, node: Node, source_code: &str) -> Result<ModuleUnit> {
        let name = get_child_node_text(node, "identifier", source_code)
            .unwrap_or_else(|| "unknown".to_string());
        let document = self.extract_documentation(node, source_code);
        let source = get_node_text(node, source_code);
        let visibility = if name.starts_with('_') {
            Visibility::Private
        } else {
            Visibility::Public
        };

        Ok(ModuleUnit {
            name,
            visibility,
            document,
            source,
            attributes: Vec::new(),
            declares: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            submodules: Vec::new(),
        })
    }
}

impl LanguageParser for PythonParser {
    fn parse_file(&mut self, file_path: &Path) -> Result<FileUnit> {
        let source_code = fs::read_to_string(file_path).map_err(Error::Io)?;
        let tree = self
            .parse(source_code.as_bytes(), None)
            .ok_or_else(|| Error::TreeSitter("Failed to parse Python file".to_string()))?;

        let mut file_unit = FileUnit {
            path: file_path.to_path_buf(),
            source: Some(source_code.clone()),
            document: None,
            declares: Vec::new(),
            modules: Vec::new(),
            functions: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let root_node = tree.root_node();

        // First look for module docstring
        {
            let mut cursor = root_node.walk();
            let mut children = root_node.children(&mut cursor);

            if let Some(first_expr) = children.next() {
                if first_expr.kind() == "expression_statement" {
                    if let Some(string) = first_expr
                        .children(&mut first_expr.walk())
                        .find(|c| c.kind() == "string")
                    {
                        if let Some(doc) = get_node_text(string, &source_code) {
                            // Clean up the docstring - handle both single and triple quotes
                            let doc = doc
                                .trim_start_matches(r#"""""#)
                                .trim_end_matches(r#"""""#)
                                .trim_start_matches(r#"'''"#)
                                .trim_end_matches(r#"'''"#)
                                .trim_start_matches('"')
                                .trim_end_matches('"')
                                .trim_start_matches('\'')
                                .trim_end_matches('\'')
                                .trim();
                            file_unit.document = Some(doc.to_string());
                        }
                    }
                }
            }
        }

        // Process imports first
        {
            let mut cursor = root_node.walk();
            for node in root_node.children(&mut cursor) {
                if node.kind() == "import_statement" || node.kind() == "import_from_statement" {
                    if let Some(import_text) = get_node_text(node, &source_code) {
                        file_unit.declares.push(crate::DeclareStatements {
                            source: import_text,
                            kind: crate::DeclareKind::Import,
                        });
                    }
                }
            }
        }

        // Then process all top-level nodes
        let mut cursor = root_node.walk();
        for node in root_node.children(&mut cursor) {
            match node.kind() {
                "function_definition" => {
                    let func = self.parse_function(node, &source_code)?;
                    file_unit.functions.push(func);
                }
                "class_definition" => {
                    let class = self.parse_class(node, &source_code)?;
                    file_unit.structs.push(class);
                }
                "decorated_definition" => {
                    let mut node_cursor = node.walk();
                    let children: Vec<_> = node.children(&mut node_cursor).collect();
                    if let Some(def_node) = children.iter().find(|n| {
                        n.kind() == "function_definition" || n.kind() == "class_definition"
                    }) {
                        match def_node.kind() {
                            "function_definition" => {
                                let func = self.parse_function(node, &source_code)?;
                                file_unit.functions.push(func);
                            }
                            "class_definition" => {
                                let class = self.parse_class(node, &source_code)?;
                                file_unit.structs.push(class);
                            }
                            _ => {}
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(file_unit)
    }
}

impl Deref for PythonParser {
    type Target = Parser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl DerefMut for PythonParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file(content: &str) -> Result<(tempfile::TempDir, PathBuf)> {
        let dir = tempfile::tempdir().map_err(Error::Io)?;
        let file_path = dir.path().join("test.py");
        fs::write(&file_path, content).map_err(Error::Io)?;
        Ok((dir, file_path))
    }

    #[test]
    fn test_parse_function() -> Result<()> {
        let content = r#"
def hello_world():
    """This is a docstring."""
    print("Hello, World!")
"#;
        let (_dir, file_path) = create_test_file(content)?;
        let mut parser = PythonParser::try_new()?;
        let file_unit = parser.parse_file(&file_path)?;

        assert_eq!(file_unit.functions.len(), 1);
        let func = &file_unit.functions[0];
        assert_eq!(func.name, "hello_world");
        assert_eq!(func.visibility, Visibility::Public);
        assert_eq!(func.documentation, Some("This is a docstring.".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_class() -> Result<()> {
        let content = r#"
@dataclass
class Person:
    """A person class."""
    def __init__(self, name: str):
        self.name = name
"#;
        let (_dir, file_path) = create_test_file(content)?;
        let mut parser = PythonParser::try_new()?;
        let file_unit = parser.parse_file(&file_path)?;

        assert_eq!(file_unit.structs.len(), 1);
        let class = &file_unit.structs[0];
        assert_eq!(class.name, "Person");
        assert_eq!(class.visibility, Visibility::Public);
        assert_eq!(class.documentation, Some("A person class.".to_string()));
        assert_eq!(class.attributes.len(), 1);
        assert_eq!(class.attributes[0], "@dataclass");
        Ok(())
    }

    #[test]
    fn test_parse_private_members() -> Result<()> {
        let content = r#"
def _private_function():
    """A private function."""
    pass

class _PrivateClass:
    """A private class."""
    pass
"#;
        let (_dir, file_path) = create_test_file(content)?;
        let mut parser = PythonParser::try_new()?;
        let file_unit = parser.parse_file(&file_path)?;

        assert_eq!(file_unit.functions[0].visibility, Visibility::Private);
        assert_eq!(file_unit.structs[0].visibility, Visibility::Private);
        Ok(())
    }

    #[test]
    fn test_parse_module_docstring() -> Result<()> {
        let content = r#"'''This is a module docstring.'''

def hello_world():
    pass
"#;
        let (_dir, file_path) = create_test_file(content)?;
        let mut parser = PythonParser::try_new()?;
        let file_unit = parser.parse_file(&file_path)?;

        assert_eq!(
            file_unit.document,
            Some("This is a module docstring.".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_parse_module_docstring_with_triple_quotes() -> Result<()> {
        let content = r#"'''This is a module docstring with triple quotes.'''

def hello_world():
    pass
"#;
        let (_dir, file_path) = create_test_file(content)?;
        let mut parser = PythonParser::try_new()?;
        let file_unit = parser.parse_file(&file_path)?;

        assert_eq!(
            file_unit.document,
            Some("This is a module docstring with triple quotes.".to_string())
        );
        Ok(())
    }
}
