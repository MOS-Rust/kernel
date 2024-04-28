//! EntryHi register (CP0 Register 10, Select 0)

#[derive(Clone, Copy, Debug)]
pub struct EntryHi {
    pub bits: u32,
}

impl EntryHi {
    register_struct_field!(get_asid, set_asid, 0, 8);
    register_struct_field!(get_vpn, set_vpn, 13, 19);
}

register_rw!(10, 0);


#[inline]
pub fn set_entry(vpn: u32, asid: u32) {
    unsafe {
        write((vpn << 13) | (asid << 8));
    }
}