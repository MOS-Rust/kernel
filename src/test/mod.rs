/// This module contains tests for the kernel.
pub mod dispatcher;
mod heap;
mod map;
mod env;

/// A macro for running tests.
///
/// This macro takes a test name as an argument and calls the `dispatcher` function
/// from the `dispatcher` module with the specified test name.
/// 
/// # Note
/// 
/// The `NO_TEST` environment variable can be set to '1' to disable running tests.
/// 
/// # Example
/// 
/// ```
/// NO_TEST=1 cargo run
/// ```
#[macro_export]
macro_rules! test {
    ($name:ident) => {
        match option_env!("NO_TEST") {
            Some("1") => {}
            _ => {
                use $crate::test::dispatcher::{dispatcher, TestName};
                dispatcher(TestName::$name);
            }
        }
    };
}
