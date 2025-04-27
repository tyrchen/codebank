/// Documentation for the struct
pub struct StructWithFields {
    /// A public field documentation
    pub public_field: String,

    /// A private field documentation
    #[allow(dead_code)]
    _private_field: i32,

    // A field without docs
    #[cfg(feature = "some_feature")]
    conditional_field: bool,

    pub another_field: u64,
}

impl StructWithFields {
    pub fn new() -> Self {
        Self {
            public_field: "hello".to_string(),
            _private_field: 42,
            #[cfg(feature = "some_feature")]
            conditional_field: true,
            another_field: 100,
        }
    }
}
