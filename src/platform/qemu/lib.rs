#![allow(dead_code)] // Temporary until all platform functions are used

pub mod reg;
pub mod cp0reg;
mod malta;
mod machine;

pub use machine::*;

use crate::const_export_usize;


pub const KUSEG: usize = 0x00000000;
pub const KSEG0: usize = 0x80000000;
pub const KSEG1: usize = 0xa0000000;
pub const KSEG2: usize = 0xc0000000;

const_export_usize!(KSTACKTOP, 0x80400000);