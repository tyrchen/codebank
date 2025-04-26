//! This is a module-level documentation comment
//! It describes the purpose of this module

/// This is a public module
pub mod public_module {
    /// This is a public struct with documentation
    #[derive(Debug, Clone)]
    pub struct PublicStruct {
        /// Public field with documentation
        pub field: String,
        /// Private field with documentation
        private_field: i32,
    }

    /// This is a public trait with documentation
    pub trait PublicTrait {
        /// Method documentation
        fn method(&self) -> String;
    }

    /// This is a public enum with documentation
    #[derive(Debug)]
    pub enum PublicEnum {
        /// Variant documentation
        Variant1,
        /// Another variant documentation
        Variant2(String),
        /// Yet another variant documentation
        Variant3 { field: i32 },
    }

    impl PublicStruct {
        /// Constructor documentation
        pub fn new(field: String, private_field: i32) -> Self {
            Self {
                field,
                private_field,
            }
        }

        /// Method documentation
        pub fn get_private_field(&self) -> i32 {
            self.private_field
        }
    }

    impl PublicTrait for PublicStruct {
        fn method(&self) -> String {
            format!("{}: {}", self.field, self.private_field)
        }
    }
}

/// This is a private module
mod private_module {
    /// Private struct
    struct PrivateStruct {
        field: String,
    }

    /// Private trait
    trait PrivateTrait {
        fn method(&self) -> String;
    }

    /// Private enum
    enum PrivateEnum {
        Variant1,
        Variant2(String),
    }

    impl PrivateStruct {
        fn new(field: String) -> Self {
            Self { field }
        }
    }

    impl PrivateTrait for PrivateStruct {
        fn method(&self) -> String {
            self.field.clone()
        }
    }
}

/// This is a public function with documentation
pub fn public_function() -> String {
    "Hello, world!".to_string()
}

/// This is a private function with documentation
fn private_function() -> String {
    "Private hello".to_string()
}

/// This is a public macro with documentation
#[macro_export]
macro_rules! public_macro {
    ($x:expr) => {
        println!("{}", $x);
    };
}

/// This is a public type alias with documentation
pub type PublicType = String;

/// This is a public constant with documentation
pub const PUBLIC_CONSTANT: &str = "constant";

/// This is a public static with documentation
pub static PUBLIC_STATIC: &str = "static";

/// This is a public attribute with documentation
#[derive(Debug)]
pub struct AttributedStruct {
    #[doc = "Field documentation"]
    pub field: String,
}

/// This is a public implementation block with documentation
impl AttributedStruct {
    /// Method documentation
    pub fn new(field: String) -> Self {
        Self { field }
    }
}

/// This is a public generic struct with documentation
pub struct GenericStruct<T> {
    /// Field documentation
    pub field: T,
}

/// This is a public generic trait with documentation
pub trait GenericTrait<T> {
    /// Method documentation
    fn method(&self, value: T) -> T;
}

/// This is a public generic implementation with documentation
impl<T> GenericTrait<T> for GenericStruct<T>
where
    T: Clone,
{
    fn method(&self, value: T) -> T {
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_function() {
        assert_eq!(public_function(), "Hello, world!");
    }
}
