# Format Trait

```rust
pub trait Formatter {
    fn format(&self, strategy: BankStrategy) -> Result<String>;
}
```

Please implement the `Formatter` trait for all the units, so that they could output the formatted code based on different strategies. Then please refactor/rewrite the code in bank.rs to leverage this trait to format the code bank based on different strategies. Make sure the public interface of `CodeBank` is easy to use and understand.

Make sure you added detailed unit tests for the implemented `Formatter` trait. Also please put units for bank.rs.
