//! EntryLo0 and EntryLo1 registers (CP0 Register 2 and 3, Select 0)

bitflags! {
    pub struct PteFlags : u32 {
        const G = 1 << 0;
        const V = 1 << 1;
        const D = 1 << 2;
        
        // Only used internally
        const C0 = 1 << 3;
        const C1 = 1 << 4;
        const C2 = 1 << 5;
        
        const Cacheable = PteFlags::C2.bits() | PteFlags::C1.bits();
        const Uncached = PteFlags::C2.bits() & !PteFlags::C1.bits();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EntryLo {
    pub bits: u32,
}

impl EntryLo {
    register_struct_field!(get_ppn, set_ppn, 6, 24);
    register_struct_bit!(0, is_global, set_global, clear_global);
    register_struct_bit!(1, is_valid, set_valid, clear_valid);
    register_struct_bit!(2, is_dirty, set_dirty, clear_dirty);

    pub fn get_flags(&self) -> PteFlags {
        PteFlags::from_bits_truncate(self.bits & 0x3f)
    }

    pub fn set_flags(&mut self, flags: PteFlags) {
        self.bits &= !0x3f;
        self.bits |= flags.bits();
    }

    pub fn set_uncached(&mut self) {
        self.set_flags(self.get_flags() | PteFlags::Uncached);
    }

    pub fn set_cacheable(&mut self) {
        self.set_flags(self.get_flags() | PteFlags::Cacheable);
    }
}

pub mod entrylo0 {
    register_rw!(2, 0);
}

pub mod entrylo1 {
    register_rw!(3, 0);
}

#[inline]
pub unsafe fn get0() -> EntryLo {
    EntryLo { bits: entrylo0::read() }
}

#[inline]
pub unsafe fn set0(value: EntryLo) {
    entrylo0::write(value.bits);
}

#[inline]
pub unsafe fn get1() -> EntryLo {
    EntryLo { bits: entrylo1::read() }
}

#[inline]
pub unsafe fn set1(value: EntryLo) {
    entrylo1::write(value.bits);
}