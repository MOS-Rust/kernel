#![allow(dead_code)] // TODO: Remove this
#![allow(unused_imports)] // TODO: Remove this

use core::cmp::{PartialEq, Eq, Ord, Ordering};
use core::convert::{From, Into};
use core::ops::{Add, Sub};

use super::layout::{PAGE_SIZE, PAGE_SIZE_BITS};

macro_rules! from_usize {
    ($to:ident) => {
        impl From<usize> for $to {
            fn from(x: usize) -> Self {
                $to(x)
            }
        }
    };
}

macro_rules! into_usize {
    ($from:ident) => {
        impl Into<usize> for $from {
            fn into(self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! impl_usize {
    ($name:ident) => {
        from_usize!($name);
        into_usize!($name);
    };

}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PA(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VA(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PPN(pub usize);

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VPN(pub usize);

impl_usize!(PA);
impl_usize!(VA);
impl_usize!(PPN);
impl_usize!(VPN);








