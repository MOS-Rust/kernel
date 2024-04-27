//! Inspired by Harry-Chen's work [rust-mips](https://github.com/Harry-Chen/rust-mips)
#![allow(unused_macros)]


macro_rules! register_read {
    ($id:expr, $select:expr) => {
        #[inline]
        pub unsafe fn read() -> u32 {
            let value: u32;
            asm!("mfc0 {0}, ${1}, {2}", out(reg) value, const $id, const $select);
            value
        }
    };
}

macro_rules! register_write {
    ($id:expr, $select:expr) => {
        #[inline]
        pub unsafe fn write(value: u32) {
            asm!("mtc0 {0}, ${1}, {2}", in(reg) value, const $id, const $select);
        }
    };
}

macro_rules! register_rw {
    ($id:expr, $select:expr) => {
        use core::arch::asm;
        register_read!($id, $select);
        register_write!($id, $select);
    };
}

macro_rules! set_bit {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() | (1 << $bit));
        }
    };
}

macro_rules! clear_bit {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() & !(1 << $bit));
        }
    };
}

macro_rules! manipulate_bit {
    ($bit: expr, $setter: ident, $clearer: ident) => {
        set_bit!($setter, $bit);
        clear_bit!($clearer, $bit);
    };
}
