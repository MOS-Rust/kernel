//! Inspired by Harry-Chen's work [rust-mips](https://github.com/Harry-Chen/rust-mips)
#![allow(unused_macros)]


macro_rules! register_read {
    ($id:expr, $select:expr) => {
        #[inline]
        pub unsafe fn read() -> u32 {
            let value: u32;
            core::arch::asm!("mfc0 {0}, ${1}, {2}", out(reg) value, const $id, const $select);
            value
        }
    };
}

macro_rules! register_write {
    ($id:expr, $select:expr) => {
        #[inline]
        pub unsafe fn write(value: u32) {
            core::arch::asm!("mtc0 {0}, ${1}, {2}", in(reg) value, const $id, const $select);
        }
    };
}

macro_rules! register_rw {
    ($id:expr, $select:expr) => {
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

macro_rules! register_bit_get {
    ($getter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $getter() -> bool {
            (read() & (1 << $bit)) != 0
        }
    };
}

macro_rules! register_bit_set {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() | (1 << $bit));
        }
    };
}

macro_rules! register_bit_clear {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub unsafe fn $setter() {
            write(read() & !(1 << $bit));
        }
    };
}

macro_rules! register_bit {
    ($getter: ident, $setter: ident, $clearer: ident, $bit: expr) => {
        register_bit_get!($getter, $bit);
        register_bit_set!($setter, $bit);
        register_bit_clear!($clearer, $bit);
    };
}

macro_rules! register_struct_field_get {
    ($getter: ident, $lobit: expr, $hibit: expr) => {
        #[inline]
        pub fn $getter(&self) -> u32 {
            (self.bits >> $lobit) & ((1 << ($hibit - $lobit + 1)) - 1)
        }
    };
}

macro_rules! register_struct_field_set {
    ($setter: ident, $lobit: expr, $hibit: expr) => {
        #[inline]
        pub fn $setter(&mut self,value: u32) {
            let mut reg = self.bits;
            reg &= !(((1 << ($hibit - $lobit + 1)) - 1) << $lobit);
            reg |= value << $lobit;
            self.bits = reg;
        }
    };
}

macro_rules! register_struct_field {
    ($getter: ident, $setter: ident, $lobit: expr, $hibit: expr) => {
        register_struct_field_get!($getter, $lobit, $hibit);
        register_struct_field_set!($setter, $lobit, $hibit);
    };
}

macro_rules! register_struct_bit_get {
    ($getter: ident, $bit: expr) => {
        #[inline]
        pub fn $getter(&self) -> bool {
            (self.bits & (1 << $bit)) != 0
        }
    };
}

macro_rules! register_struct_bit_set {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub fn $setter(&mut self) {
            self.bits |= 1 << $bit;
        }
    };
}

macro_rules! register_struct_bit_clear {
    ($setter: ident, $bit: expr) => {
        #[inline]
        pub fn $setter(&mut self) {
            self.bits &= !(1 << $bit);
        }
    };
}

macro_rules! register_struct_bit {
    ($getter: ident, $setter: ident, $clearer: ident, $bit: expr) => {
        register_struct_bit_get!($getter, $bit);
        register_struct_bit_set!($setter, $bit);
        register_struct_bit_clear!($clearer, $bit);
    };
}