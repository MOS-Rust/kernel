//! Macros

mod bitops {
    /// Round up `a` to the nearest multiple of `n`.
    #[macro_export]
    macro_rules! round {
        ($a:expr, $n:expr) => {
            ($a + $n - 1) & !($n - 1)
        };
    }

    /// Round down `a` to the nearest multiple of `n`.
    #[macro_export]
    macro_rules! round_down {
        ($a:expr, $n:expr) => {
            $a & !($n - 1)
        };
    }
}

mod export {
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
            /// return value
            pub const $name: usize = $value;
        };
    }

    /// Export a &str constant.
    #[macro_export]
    macro_rules! const_export_str {
        ($name:ident, $value:expr) => {
            #[allow(dead_code)]
            /// return value
            pub const $name: &str = $value;
        };
    }
}

/// Include bytes
pub mod include_bytes {
    /// Align struct
    #[repr(C)]
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        /// Align value
        pub _align: [Align; 0],
        /// Align bytes
        pub bytes: Bytes,
    }

    /// Include a file as a byte slice aligned as a specific type.
    #[macro_export]
    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {{
            use $crate::macros::include_bytes::AlignedAs;

            static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($path),
            };

            &ALIGNED.bytes
        }};
    }
}
