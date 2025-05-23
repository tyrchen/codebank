//! File for advanced Rust constructs.

pub mod level1 {
    pub mod level2 {
        /// Struct deep inside modules.
        #[derive(Default)]
        pub(in crate::level1) struct DeepStruct { // pub(in path) visibility
            pub field_a: String,
        }

        impl DeepStruct {
            pub fn new() -> Self {
                Self::default()
            }
        }
    }

    /// Function with complex generics and where clause.
    pub fn complex_generic_function<'a, T, U>(param_t: T, param_u: &'a U) -> Result<T, U::Error>
    where
        T: std::fmt::Debug + Clone + Send + 'static,
        U: std::error::Error + ?Sized, // ?Sized bound
        for<'b> &'b U: Send, // Higher-rank trait bound
    {
        println!("T: {:?}, U: {}", param_t, param_u);
        Ok(param_t.clone())
    }
}

// Struct with lifetime and multiple generic parameters with bounds
pub struct AdvancedGenericStruct<'a, A, B> 
where 
    A: AsRef<[u8]> + ?Sized, 
    B: 'a + Send + Sync,
{
    data_a: &'a A,
    data_b: B,
    pub simple_field: i32,
}

impl<'a, A, B> AdvancedGenericStruct<'a, A, B>
where 
    A: AsRef<[u8]> + ?Sized, 
    B: 'a + Send + Sync,
{
    pub fn new(data_a: &'a A, data_b: B) -> Self {
        Self { data_a, data_b, simple_field: 0 }
    }
}

// Enum with generics and where clause
pub enum GenericResult<S, E> 
where S: Send, E: std::fmt::Debug 
{
    Success(S),
    Failure(E),
}

// Trait with associated types and complex bounds
pub trait AdvancedTrait {
    type Item: Copy + Default; // Associated type with bounds
    const VERSION: &'static str;

    fn process(&self, item: Self::Item) -> Result<Self::Item, String>;
    fn version() -> &'static str { Self::VERSION }
}

// Impl for a specific type using the advanced trait
struct MyTypeForAdvancedTrait;

impl AdvancedTrait for MyTypeForAdvancedTrait {
    type Item = u32;
    const VERSION: &'static str = "1.0-advanced";

    fn process(&self, item: Self::Item) -> Result<Self::Item, String> {
        if item > 100 {
            Err("Value too large".to_string())
        } else {
            Ok(item * 2)
        }
    }
}

// Unit struct for testing edge cases
pub struct MyUnitStruct;

// Empty struct for testing edge cases
pub struct EmptyStruct {}

// Struct with no fields, but curly braces
struct NoFieldsStruct {}
