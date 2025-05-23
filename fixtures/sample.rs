//! This is a file-level documentation comment.
//! It describes the purpose of this sample file which includes a variety of Rust items.

// Example of extern crate
extern crate proc_macro;
extern crate serde as serde_renamed;


// Example of use declarations
use std::collections::HashMap;
use std::fmt::{self, Debug as FmtDebug}; // aliased import
use crate::public_module::PublicStruct; // use item from same crate

/// This is a public module.
/// It has multiple lines of documentation.
#[cfg(feature = "some_feature")]
#[deprecated(note = "This module is old")]
pub mod public_module {
    //! Inner documentation for public_module.

    /// This is a public struct with documentation.
    /// It also has generics and attributes.
    #[derive(Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct PublicStruct<T: FmtDebug + Clone, U>
    where
        U: Default,
    {
        /// Public field with documentation
        pub field: T,
        /// Private field with documentation
        private_field: U, // Changed to U for generic usage
        #[doc="Inner attribute doc for another_field"]
        pub another_field: i32,
    }

    /// This is a public trait with documentation
    #[allow(unused_variables)]
    pub trait PublicTrait<T> {
        /// Method documentation for trait.
        fn method(&self, input: T) -> String;
    }

    /// This is a public enum with documentation
    #[derive(Debug)]
    pub enum PublicEnum {
        /// Variant documentation for Variant1
        Variant1,
        /// Another variant documentation for Variant2
        #[allow(dead_code)] // Attribute on variant
        Variant2(String),
        /// Yet another variant documentation for Variant3
        /*! Block-style inner doc for Variant3 */
        Variant3 { 
            /// Field inside a variant
            #[serde(skip)]
            field: i32 
        },
    }

    // Function with pub(crate) visibility
    pub(crate) fn crate_visible_function() {
        println!("This function is crate visible.");
    }
    
    // Function with pub(super) visibility (relative to current module 'public_module')
    // This would typically be in a nested module to make sense.
    // For demonstration, let's put it here. If it were in `super::public_module::nested_mod`,
    // `pub(super)` would make it visible to `public_module`.
    // Here, it's visible to the crate root (super of public_module is the crate root).
    pub(super) fn super_visible_function() {
        println!("This function is super visible.");
    }


    impl<T: FmtDebug + Clone, U: Default> PublicStruct<T, U> {
        /// Constructor documentation
        pub fn new(field: T, private_field: U, another_field: i32) -> Self {
            Self {
                field,
                private_field,
                another_field,
            }
        }

        /// Method documentation
        pub fn get_private_field(&self) -> &U {
            &self.private_field
        }
    }

    impl<T: FmtDebug + Clone> PublicTrait<T> for PublicStruct<T, String> { // Assuming U = String for this impl
        fn method(&self, input: T) -> String {
            format!("Field: {:?}, Private: {}, Input: {:?}", self.field, self.private_field, input)
        }
    }
    
    pub mod nested_module {
        //! Inner docs for nested_module.
        
        // This function's pub(super) makes it visible to public_module
        pub(super) fn visible_to_public_module() {}

        // This function's pub(crate) makes it visible throughout the crate
        pub(crate) fn crate_visible_from_nested() {}

        struct OnlyInNested {}
    }
}

/// This is a private module
mod private_module {
    /*! Inner block doc for private_module. */
    struct PrivateStruct { field: String }
    trait PrivateTrait { fn method(&self) -> String; }
    enum PrivateEnum { Variant1, Variant2(String) }
    impl PrivateStruct { fn new(field: String) -> Self { Self { field } } }
    impl PrivateTrait for PrivateStruct { fn method(&self) -> String { self.field.clone() } }
}

/// A public function with multiple attributes and docs.
/// Second line of doc.
#[inline]
#[must_use = "Return value should be used"]
pub fn public_function() -> String {
    "Hello, world!".to_string()
}

/// This is a private function with documentation
#[allow(dead_code)]
fn private_function(s: &str) -> String {
    format!("Private hello: {}", s)
}


/// This is a public type alias with documentation
pub type PublicTypeAlias<T> = Result<T, Box<dyn std::error::Error>>;

/// This is a public constant with documentation
pub const PUBLIC_CONSTANT: &str = "constant value";

/// This is a public static with documentation
#[no_mangle]
pub static PUBLIC_STATIC_VAR: i32 = 100;


/// This is a public generic struct with documentation
pub struct GenericStruct<T> {
    /// Field documentation
    pub field: T,
}

/// This is a public generic trait with documentation
#[allow(unused_variables)]
pub trait GenericTrait<T> {
    /// Method documentation for trait
    fn method(&self, value: T) -> T;
}

/// Implementation for GenericStruct.
#[allow(dead_code)]
impl<T> GenericStruct<T> {
    /// Creates a new GenericStruct.
    fn new(value: T) -> Self {
        Self { field: value }
    }
}


/// Implementation of GenericTrait for GenericStruct.
/// Includes a where clause.
impl<T> GenericTrait<T> for GenericStruct<T>
where
    T: Clone + FmtDebug, // Added FmtDebug here
{
    /// Method from GenericTrait.
    fn method(&self, value: T) -> T {
        println!("Value: {:?}", value);
        value.clone()
    }
}

// A module defined in another file (declaration)
mod my_other_module;

#[cfg(test)]
mod tests {
    use super::*; // Imports items from the parent module (the file scope)

    #[test]
    fn test_public_function_output() { // Renamed to avoid conflict
        assert_eq!(public_function(), "Hello, world!");
    }

    #[test]
    fn check_public_struct_instantiation() {
        // This test is more about Rust syntax than parser, but ensures sample code is valid.
        let _ps = public_module::PublicStruct {
            field: "test".to_string(),
            private_field: 10, // Original was i32, now U, assuming i32 for test.
            another_field: 20,
        };
    }
}
