#![allow(dead_code)] // TODO: Remove this

pub mod reg;
pub mod cp0reg;
mod malta;
mod machine;

pub use machine::*;