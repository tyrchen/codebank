# Instructions

VAN: based on @0001-initial-idea.md spec and the existing code base (I've already defined the basic data structure, trait etc.), please initialize memory bank.

Please help to improve data structure and fix the bank logic based on requirements.

Please fix the parser, looks like for `fixtures/sample.rs` the functions, structs, traits, impls, etc. hasn't been parsed successfully and they are all empty.

Data structure for various units has improved. Please help to improve parser to parse those information. Update unit tests accordingly.

Module Unit should also parse its own functions, traits, impls, etc. Please help to fix and add relevant unit tests accordingly

Please refactor code based on @0004-simplify.md spec. The unit data structure in ./src/parser/mod.rs has been updated, please update relevant implementation. Remove unnecessary code.

no-tests formatter should include all declare / struct / trait / impl / function / etc., it only removes the test module and test functions / cases. Please fix it and add proper unit test case to cover that.

if impl has empty function list, just return empty string. Also put a test case for this (e.g. impl contains all private function, under summary strategy, it should return empty string.

please implement based on the spec @0005-mcp.md.

Please add the support for python parser. Make sure you provide good unit test coverage and make sure `cargo test` passes and `cargo clippy` has not lint errors.

Please improve formatter for example NoTest logic should allow different implementation for rust / python / etc., similar as summary for function signature only, rust is `{ ... }` python should be ` ...`, etc. I've moved formatters.rs to a mod under formatter. Please provide specific functions to @rust.rs or @python.rs . And put python formatter tests to @python.rs

Please support typescript parser. Please proceed until you finished parser. Then add enough unit test to make sure things are working. Make sure all test passes and `cargo clippy` works.

Now please implement cpp parser @cpp.rs using tree-sitter. Please add test cases based on @sample.cpp . Make sure all relevant grammar are properly parsed.
