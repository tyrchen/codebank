# Improvements

There're some improvements to be made to the data structure.

For all the units type, it should have a list of attributes. For example, `#[cfg(test)]` is an attribute for `ModuleUnit`. `#[derive(Debug)]` is an attribute for `StructUnit`, `#[test]` is an attribute for `FunctionUnit`, etc.

For the `FunctionUnit`, it should be able to have more fields:

- `signature` field, which is a `String` that represents the signature of the function.
- `body` field, which is a `String` that represents the source code of the function.

For BankStrategy::Summary:

- for module / function / struct / trait / method, if visibility is not public, it should be skipped.
- for function, only its signature is generated, and the body is skipped.

For BankStrategy::NoTests:

- skip all test functions and test modules.

Below is an example of the data structure currently being parsed:

```rust
FileUnit {
    path: "fixtures/sample.rs",
    modules: [
        ModuleUnit {
            name: "public_module",
            visibility: Public,
            documentation: None,
            functions: [],
            structs: [],
            traits: [],
            impls: [],
            submodules: [],
            source: Some(
                "pub mod public_module {\n    /// This is a public struct with documentation\n    #[derive(Debug, Clone)]\n    pub struct PublicStruct {\n        /// Public field with documentation\n        pub field: String,\n        /// Private field with documentation\n        private_field: i32,\n    }\n\n    /// This is a public trait with documentation\n    pub trait PublicTrait {\n        /// Method documentation\n        fn method(&self) -> String;\n    }\n\n    /// This is a public enum with documentation\n    #[derive(Debug)]\n    pub enum PublicEnum {\n        /// Variant documentation\n        Variant1,\n        /// Another variant documentation\n        Variant2(String),\n        /// Yet another variant documentation\n        Variant3 { field: i32 },\n    }\n\n    impl PublicStruct {\n        /// Constructor documentation\n        pub fn new(field: String, private_field: i32) -> Self {\n            Self {\n                field,\n                private_field,\n            }\n        }\n\n        /// Method documentation\n        pub fn get_private_field(&self) -> i32 {\n            self.private_field\n        }\n    }\n\n    impl PublicTrait for PublicStruct {\n        fn method(&self) -> String {\n            format!(\"{}: {}\", self.field, self.private_field)\n        }\n    }\n}",
            ),
        },
        ModuleUnit {
            name: "private_module",
            visibility: Private,
            documentation: None,
            functions: [],
            structs: [],
            traits: [],
            impls: [],
            submodules: [],
            source: Some(
                "mod private_module {\n    /// Private struct\n    struct PrivateStruct {\n        field: String,\n    }\n\n    /// Private trait\n    trait PrivateTrait {\n        fn method(&self) -> String;\n    }\n\n    /// Private enum\n    enum PrivateEnum {\n        Variant1,\n        Variant2(String),\n    }\n\n    impl PrivateStruct {\n        fn new(field: String) -> Self {\n            Self { field }\n        }\n    }\n\n    impl PrivateTrait for PrivateStruct {\n        fn method(&self) -> String {\n            self.field.clone()\n        }\n    }\n}",
            ),
        },
        ModuleUnit {
            name: "tests",
            visibility: Private,
            documentation: None,
            functions: [],
            structs: [],
            traits: [],
            impls: [],
            submodules: [],
            source: Some(
                "mod tests {\n    use super::*;\n\n    #[test]\n    fn test_public_function() {\n        assert_eq!(public_function(), \"Hello, world!\");\n    }\n}",
            ),
        },
    ],
    functions: [
        FunctionUnit {
            name: "public_function",
            visibility: Public,
            documentation: None,
            parameters: [],
            return_type: None,
            source: Some(
                "pub fn public_function() -> String {\n    \"Hello, world!\".to_string()\n}",
            ),
        },
        FunctionUnit {
            name: "private_function",
            visibility: Private,
            documentation: None,
            parameters: [],
            return_type: None,
            source: Some(
                "fn private_function() -> String {\n    \"Private hello\".to_string()\n}",
            ),
        },
    ],
    structs: [],
    traits: [],
    impls: [],
    source: Some(
        "//! This is a module-level documentation comment\n//! It describes the purpose of this module\n\n/// This is a public module\npub mod public_module {\n    /// This is a public struct with documentation\n    #[derive(Debug, Clone)]\n    pub struct PublicStruct {\n        /// Public field with documentation\n        pub field: String,\n        /// Private field with documentation\n        private_field: i32,\n    }\n\n    /// This is a public trait with documentation\n    pub trait PublicTrait {\n        /// Method documentation\n        fn method(&self) -> String;\n    }\n\n    /// This is a public enum with documentation\n    #[derive(Debug)]\n    pub enum PublicEnum {\n        /// Variant documentation\n        Variant1,\n        /// Another variant documentation\n        Variant2(String),\n        /// Yet another variant documentation\n        Variant3 { field: i32 },\n    }\n\n    impl PublicStruct {\n        /// Constructor documentation\n        pub fn new(field: String, private_field: i32) -> Self {\n            Self {\n                field,\n                private_field,\n            }\n        }\n\n        /// Method documentation\n        pub fn get_private_field(&self) -> i32 {\n            self.private_field\n        }\n    }\n\n    impl PublicTrait for PublicStruct {\n        fn method(&self) -> String {\n            format!(\"{}: {}\", self.field, self.private_field)\n        }\n    }\n}\n\n/// This is a private module\nmod private_module {\n    /// Private struct\n    struct PrivateStruct {\n        field: String,\n    }\n\n    /// Private trait\n    trait PrivateTrait {\n        fn method(&self) -> String;\n    }\n\n    /// Private enum\n    enum PrivateEnum {\n        Variant1,\n        Variant2(String),\n    }\n\n    impl PrivateStruct {\n        fn new(field: String) -> Self {\n            Self { field }\n        }\n    }\n\n    impl PrivateTrait for PrivateStruct {\n        fn method(&self) -> String {\n            self.field.clone()\n        }\n    }\n}\n\n/// This is a public function with documentation\npub fn public_function() -> String {\n    \"Hello, world!\".to_string()\n}\n\n/// This is a private function with documentation\nfn private_function() -> String {\n    \"Private hello\".to_string()\n}\n\n/// This is a public macro with documentation\n#[macro_export]\nmacro_rules! public_macro {\n    ($x:expr) => {\n        println!(\"{}\", $x);\n    };\n}\n\n/// This is a public type alias with documentation\npub type PublicType = String;\n\n/// This is a public constant with documentation\npub const PUBLIC_CONSTANT: &str = \"constant\";\n\n/// This is a public static with documentation\npub static PUBLIC_STATIC: &str = \"static\";\n\n/// This is a public attribute with documentation\n#[derive(Debug)]\npub struct AttributedStruct {\n    #[doc = \"Field documentation\"]\n    pub field: String,\n}\n\n/// This is a public implementation block with documentation\nimpl AttributedStruct {\n    /// Method documentation\n    pub fn new(field: String) -> Self {\n        Self { field }\n    }\n}\n\n/// This is a public generic struct with documentation\npub struct GenericStruct<T> {\n    /// Field documentation\n    pub field: T,\n}\n\n/// This is a public generic trait with documentation\npub trait GenericTrait<T> {\n    /// Method documentation\n    fn method(&self, value: T) -> T;\n}\n\n/// This is a public generic implementation with documentation\nimpl<T> GenericTrait<T> for GenericStruct<T>\nwhere\n    T: Clone,\n{\n    fn method(&self, value: T) -> T {\n        value.clone()\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_public_function() {\n        assert_eq!(public_function(), \"Hello, world!\");\n    }\n}\n",
    ),
}
```
