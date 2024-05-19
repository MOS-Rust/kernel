//! Warp a constant definition into a macro invocation.
//! 
//! Because Rust lacks a way to do the same thing as C's `#define` directive,
//! we are using a macro to wrap a constant definition, so that we can use
//! 3-rd party tools to extract the constant definition to be exported and 
//! use them in assembly.

/// Export a usize constant.
#[macro_export]
macro_rules! const_export_usize {
    ($name:ident, $value:expr) => {
        #[allow(dead_code)]
        pub const $name: usize = $value;
    };
}

/// Export a &str constant.
#[macro_export]
macro_rules! const_export_str {
    ($name:ident, $value:expr) => {
        #[allow(dead_code)]
        pub const $name: &str = $value;
    };
}