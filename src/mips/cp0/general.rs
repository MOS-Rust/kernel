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

macro_rules! register_struct_rw {
    ($ident: ident) => {
        #[inline]
        pub unsafe fn read_struct() -> $ident {
            $ident {bits: read()}
        }

        #[inline]
        pub unsafe fn write_struct(value: $ident) {
            write(value.bits);
        }
    }
}

macro_rules! register_set_bit {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() | (1 << $bit));
        }
    };
}

macro_rules! register_clear_bit {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() & !(1 << $bit));
        }
    };
}

macro_rules! register_bit {
    ($bit: expr, $setter: ident, $clearer: ident) => {
        register_set_bit!($setter, $bit);
        register_clear_bit!($clearer, $bit);
    };
}

macro_rules! register_field {
    ($field: ident, $mask: expr, $shift: expr) => {
        #[inline]
        pub unsafe fn get_$field() -> u32 {
            (read() & $mask) >> $shift
        }

        #[inline]
        pub unsafe fn set_$field(value: u32) {
            let mut reg = read();
            reg &= !$mask;
            reg |= value << $shift;
            write(reg);
        }
    };
}
