use crate::parser::LanguageType;

pub struct FormatterRules {
    pub summary_ellipsis: &'static str,
    pub function_body_start_marker: &'static str,
    pub function_body_end_marker: &'static str,
    pub doc_marker: &'static str,
    pub test_markers: &'static [&'static str],
    pub test_module_markers: &'static [&'static str],
}

const RUST_RULES: FormatterRules = FormatterRules {
    summary_ellipsis: " { ... }",
    function_body_start_marker: "{",
    function_body_end_marker: "}",
    doc_marker: "///",
    test_markers: &["#[test]", "#[cfg(test)]"],
    test_module_markers: &["#[cfg(test)]", "tests"],
};

const PYTHON_RULES: FormatterRules = FormatterRules {
    summary_ellipsis: " ...",
    function_body_start_marker: ":",
    function_body_end_marker: "",
    doc_marker: "#",
    test_markers: &["@pytest", "test_"],
    test_module_markers: &["test_"],
};

const TS_RULES: FormatterRules = FormatterRules {
    summary_ellipsis: " { ... }",
    function_body_start_marker: "{",
    function_body_end_marker: "}",
    doc_marker: "//",
    test_markers: &["@test", "test_"],
    test_module_markers: &["test_"],
};

const C_RULES: FormatterRules = FormatterRules {
    summary_ellipsis: " { ... }",
    function_body_start_marker: "{",
    function_body_end_marker: "}",
    doc_marker: "//",
    test_markers: &["@test", "test_"],
    test_module_markers: &["test_"],
};

const UNKNOWN_RULES: FormatterRules = FormatterRules {
    summary_ellipsis: "...",
    function_body_start_marker: "",
    function_body_end_marker: "",
    doc_marker: "//",
    test_markers: &[],
    test_module_markers: &[],
};

impl FormatterRules {
    #[inline(always)]
    pub fn for_language(lang: LanguageType) -> Self {
        match lang {
            LanguageType::Rust => RUST_RULES,
            LanguageType::Python => PYTHON_RULES,
            LanguageType::TypeScript => TS_RULES,
            LanguageType::C => C_RULES,
            LanguageType::Unknown => UNKNOWN_RULES,
        }
    }

    pub fn is_test_function(&self, attributes: &[String]) -> bool {
        attributes
            .iter()
            .any(|attr| self.test_markers.iter().any(|marker| attr.contains(marker)))
    }

    pub fn is_test_module(&self, name: &str, attributes: &[String]) -> bool {
        self.test_module_markers.iter().any(|marker| {
            name.starts_with(marker) || attributes.iter().any(|attr| attr.contains(marker))
        })
    }

    pub fn format_signature(&self, source: &str, signature: Option<&str>) -> String {
        if let Some(sig) = signature {
            format!("{}{}", sig.trim(), self.summary_ellipsis)
        } else if let Some(idx) = source.find(self.function_body_start_marker) {
            format!("{}{}", source[0..idx].trim(), self.summary_ellipsis)
        } else {
            source.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_rules() {
        let rules = FormatterRules::for_language(LanguageType::Rust);
        assert_eq!(rules.summary_ellipsis, " { ... }");
        assert_eq!(rules.function_body_start_marker, "{");
        assert_eq!(rules.test_markers, &["#[test]", "#[cfg(test)]"]);
        assert_eq!(rules.test_module_markers, &["#[cfg(test)]", "tests"]);
    }

    #[test]
    fn test_python_rules() {
        let rules = FormatterRules::for_language(LanguageType::Python);
        assert_eq!(rules.summary_ellipsis, ": ...");
        assert_eq!(rules.function_body_start_marker, ":");
        assert_eq!(rules.test_markers, &["@pytest", "test_"]);
        assert_eq!(rules.test_module_markers, &["test_"]);
    }

    #[test]
    fn test_unknown_language_rules() {
        let rules = FormatterRules::for_language(LanguageType::Unknown);
        assert_eq!(rules.summary_ellipsis, "...");
        assert_eq!(rules.function_body_start_marker, "");
        assert!(rules.test_markers.is_empty());
        assert!(rules.test_module_markers.is_empty());
    }

    #[test]
    fn test_is_test_function() {
        let rules = FormatterRules::for_language(LanguageType::Rust);

        // Test Rust test function detection
        assert!(rules.is_test_function(&vec!["#[test]".to_string()]));
        assert!(rules.is_test_function(&vec!["#[cfg(test)]".to_string()]));
        assert!(!rules.is_test_function(&vec!["#[derive(Debug)]".to_string()]));

        let rules = FormatterRules::for_language(LanguageType::Python);

        // Test Python test function detection
        assert!(rules.is_test_function(&vec!["@pytest.mark.test".to_string()]));
        assert!(rules.is_test_function(&vec!["test_function".to_string()]));
        assert!(!rules.is_test_function(&vec!["regular_function".to_string()]));
    }

    #[test]
    fn test_is_test_module() {
        let rules = FormatterRules::for_language(LanguageType::Rust);

        // Test Rust test module detection
        assert!(rules.is_test_module("tests", &vec![]));
        assert!(rules.is_test_module("module", &vec!["#[cfg(test)]".to_string()]));
        assert!(!rules.is_test_module("module", &vec![]));

        let rules = FormatterRules::for_language(LanguageType::Python);

        // Test Python test module detection
        assert!(rules.is_test_module("test_module", &vec![]));
        assert!(!rules.is_test_module("regular_module", &vec![]));
    }

    #[test]
    fn test_format_signature() {
        let rules = FormatterRules::for_language(LanguageType::Rust);

        // Test with signature provided
        assert_eq!(
            rules.format_signature("fn test() {}", Some("fn test()")),
            "fn test() { ... }"
        );

        // Test without signature
        assert_eq!(
            rules.format_signature("fn test() {", None),
            "fn test() { ... }"
        );

        // Test without function end marker
        assert_eq!(rules.format_signature("fn test()", None), "fn test()");

        // Test with extra whitespace
        assert_eq!(
            rules.format_signature("fn test()  {", None),
            "fn test() { ... }"
        );

        let rules = FormatterRules::for_language(LanguageType::Python);

        // Test Python function signature
        assert_eq!(
            rules.format_signature("def test():", None),
            "def test(): ..."
        );
    }
}
