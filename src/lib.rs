//! # Test helpers should be simple.
//!
//! You don't want to have to worry about bugs in your test suite or unexpected behaviour leading
//! to failing tests silently passing. With that in mind, `simple_test_case` aims to do the bare
//! minimum to eliminate the boilerplate of writing parameterised tests and no more.
//!
//! The `test_case` attribute macro handles generating multiple test functions for you which are
//! parameterised by the inputs you provide. You still need to provide the `#[test]` attribute (or
//! an alternative such as `#[tokio::test]`) and all test cases _must_ be provided before any
//! additional attribute macros you wish to apply.
//!
//! And that's it.
//!
//! There is no additional support for custom assertions, fixtures etc. That said, if you want or
//! need a more complicated testing set up, additional attribute macros should play nice with
//! `simple_test_case` provided you follow the advice below.
//!
//!
//! ## Usage
//!
//! ### Valid
//! Here the `#[test]` attribute is provided after all instances of `test_case`. This will work.
//! ```rust
//! use simple_test_case::test_case;
//!
//! fn double(n: usize) -> usize {
//!     n * 2
//! }
//!
//! #[test_case(1, 2; "case 1")]
//! #[test_case(3, 6; "case 2")]
//! #[test]
//! fn double_test(n: usize, double: usize) {
//!     assert_eq!(double(n), double)
//! }
//! ```
//!
//! ### Invalid
//! Here the `#[test]` attribute is provided before all instances of `test_case`. This will cause
//! the compiler to complain about functions used as tests not being allowed to have any arguments.
//! ```ignore
//! use simple_test_case::test_case;
//!
//! fn double(n: usize) -> usize {
//!     n * 2
//! }
//!
//! #[test]
//! #[test_case(1, 2; "case 1")]
//! #[test_case(3, 6; "case 2")]
//! fn double_test(n: usize, double: usize) {
//!     assert_eq!(double(n), double)
//! }
//! ```
//!
//! ## Additional attributes
//!
//! `test_case` preserves all attributes beneath it, forwarding them on to the individual generated
//! test functions. As an example, the standard library `should_panic` attribute works just fine as
//! shown below (just make sure to provide your test cases first as described above):
//!
//! ```rust
//! use simple_test_case::test_case;
//!
//! #[test_case(1, 2; "case 1")]
//! #[test_case(3, 6; "case 2")]
//! #[test]
//! #[should_panic(expected = "this works")]
//! fn panic_test(n: usize, double: usize) {
//!     assert_eq!(double(a), b);
//!     panic!("this works")
//! }
//! ```
//!
//! ### Async tests
//!
//! Async tests are supported in the same way that all other attributes are supported: add your
//! tests cases first and then apply the async testing macro of your choice beneath.
//! ```rust
//! use simple_test_case::test_case;
//!
//! async fn async_double(n: usize) -> usize {
//!     n * 2
//! }
//!
//! #[test_case(1, 2; "case 1")]
//! #[test_case(3, 6; "case 2")]
//! #[tokio::test]
//! async fn double_test(n: usize, double: usize) {
//!     assert_eq!(double(n).await, double)
//! }
//! ```
//!
//! ## How does it work?
//!
//! You are encouraged to read the source of the macro itself (the macro plus associated helper
//! functions are under 150 lines of code) but the general idea is as follows:
//!
//! - Collect all `test_case` (or `simple_test_case::test_case`) attributes, each of which maps a
//!   set of function arguments to a test case name.
//! - For each test case create a copy of the original test function with the function arguments
//!   replaced with explicit variable bindings at the top of the function body.
//! - Write out each of the cases as their own test inside of a new module that is named using the
//!   original test function name.
//!
//! You can use [cargo expand](https://github.com/dtolnay/cargo-expand) to see what the generated
//! tests look like using the example provided in the `examples` directory like so:
//!
//! ```bash
//! $ cargo expand --example=expand_me --tests
//!   Compiling simple_test_case v0.1.0 (/home/innes/repos/personal/simple_test_case)
//!    Finished test [unoptimized + debuginfo] target(s) in 0.12s
//!
//! #![feature(prelude_import)]
//! #[prelude_import]
//! use std::prelude::rust_2021::*;
//! #[macro_use]
//! extern crate std;
//! use simple_test_case::test_case;
//! mod example {
//!     #[allow(unused_imports)]
//!     use super::*;
//!     extern crate test;
//!     #[cfg(test)]
//!     #[rustc_test_marker]
//!     pub const small_example: test::TestDescAndFn = test::TestDescAndFn {
//!         desc: test::TestDesc {
//!             name: test::StaticTestName("example::small_example"),
//!             ignore: false,
//!             allow_fail: false,
//!             compile_fail: false,
//!             no_run: false,
//!             should_panic: test::ShouldPanic::No,
//!             test_type: test::TestType::Unknown,
//!         },
//!         testfn: test::StaticTestFn(|| test::assert_test_result(small_example())),
//!     };
//!     fn small_example() {
//!         let a: usize = 1;
//!         let b: usize = 2;
//!         if !(a < b) {
//!             ::core::panicking::panic("assertion failed: a < b")
//!         }
//!     }
//!     extern crate test;
//!     #[cfg(test)]
//!     #[rustc_test_marker]
//!     pub const large_example: test::TestDescAndFn = test::TestDescAndFn {
//!         desc: test::TestDesc {
//!             name: test::StaticTestName("example::large_example"),
//!             ignore: false,
//!             allow_fail: false,
//!             compile_fail: false,
//!             no_run: false,
//!             should_panic: test::ShouldPanic::No,
//!             test_type: test::TestType::Unknown,
//!         },
//!         testfn: test::StaticTestFn(|| test::assert_test_result(large_example())),
//!     };
//!     fn large_example() {
//!         let a: usize = 100;
//!         let b: usize = 200;
//!         if !(a < b) {
//!             ::core::panicking::panic("assertion failed: a < b")
//!         }
//!     }
//! }
//! #[allow(dead_code)]
//! fn main() {}
//! #[rustc_main]
//! pub fn main() -> () {
//!     extern crate test;
//!     test::test_main_static(&[&small_example, &large_example])
//! }
//! ```
use proc_macro::TokenStream;

mod dir_cases;
mod test_case;
mod util;

/// A simple parameterised test helper
///
/// See the main module documentation for usage details.
#[proc_macro_attribute]
pub fn test_case(args: TokenStream, input: TokenStream) -> TokenStream {
    test_case::inner(args, input)
}

/// Generate a set of parameterised tests from files in present in a given directory.
///
/// The path(s) given will be resolved relative to the root of your cargo workspace and the test
/// function that you provide must accept to `&str` arguments: the path to the file loaded for the
/// test case and the contents of that file. The files are read at compile time so in order for
/// adding/removing/modifying files in the given directory to trigger a recompile of your
/// tests you will need to set up a build.rs file using [rerun-if-changed][0].
///
/// In its simplest use this macro will generate a test case for each file found in the given
/// directory, providing the path and contents of that file as arguments to your test function.
///
/// # Example
/// If we have the following directory contents:
/// ```bash
/// $ ls resources/my-test-data
///   foo.txt
///   bar-with-dashes.json
///   baz_with_underscores.yaml
/// ```
///
/// The the following test case will be expanded into three test cases:
/// ```ignore
/// #[dir_cases("resources/my-test-data")]
/// #[test]
/// fn example(path: &str, contents: &str) {
///   // your test logic using the contents of the file
/// }
/// ```
///
/// ```bash
/// $ cargo test
///   ...
///   running 3 tests
///   test example::foo ... ok
///   test example::bar_with_dashes ... ok
///   test example::baz_with_underscores ... ok
///
///   test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
/// ```
///
/// If you wish, you may specify a comma delimited list of directories to use for generating test
/// cases instead of just a single directory. If you do, you will need to ensure that there are no
/// duplicate filenames between the provided directories in order to avoid attempting to generate
/// multiple tests with the same name.
///
///   [0]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
#[proc_macro_attribute]
pub fn dir_cases(args: TokenStream, input: TokenStream) -> TokenStream {
    dir_cases::inner(args, input)
}
