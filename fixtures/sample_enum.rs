/// Module documentation for the enum sample file.
use std::fmt::Debug;

/// This is a public enum with documentation
#[derive(Debug)]
pub enum PublicEnum {
    /// Variant documentation
    Variant1,
    /// Another variant documentation
    #[allow(dead_code)] // Attribute on variant
    Variant2(String),
    /// Yet another variant documentation
    Variant3 { field: i32 },
}

// A private enum
enum PrivateEnum {
    Internal,
}

impl PublicEnum {
    // An associated function (treated like a method)
    pub fn describe(&self) {
        println!("Enum variant: {:?}", self);
    }
}
