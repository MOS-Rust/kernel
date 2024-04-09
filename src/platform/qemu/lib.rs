#![allow(dead_code)] // Temporary until all platform functions are used

pub mod malta;
mod machine;

pub use machine::*;

pub const KUSEG: usize = 0x00000000;
pub const KSEG0: usize = 0x80000000;
pub const KSEG1: usize = 0xa0000000;
pub const KSEG2: usize = 0xc0000000;