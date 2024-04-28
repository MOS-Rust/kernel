//! MIPS General Purpose Registers

use core::arch::asm;

pub trait GeneralPurposeRegister {
    const INDEX: usize;
}

macro_rules! impl_gpr {
    ($name:ident, $index:expr) => {
        #[allow(non_camel_case_types)]
        pub struct $name;

        impl GeneralPurposeRegister for $name {
            const INDEX: usize = $index;
        }
    };
}

#[inline]
pub unsafe fn read<T: GeneralPurposeRegister>() -> u32 {
    let value: u32;
    asm!("move {0}, ${1}",
         out(reg) value,
         const T::INDEX,
    );
    value
}

#[inline]
pub unsafe fn write<T: GeneralPurposeRegister>(value: u32) {
    asm!("move ${0}, {1}",
         const T::INDEX,
         in(reg) value,
    );
}

impl_gpr!(zero, 0);
impl_gpr!(at, 1);
impl_gpr!(v0, 2);
impl_gpr!(v1, 3);
impl_gpr!(a0, 4);
impl_gpr!(a1, 5);
impl_gpr!(a2, 6);
impl_gpr!(a3, 7);
impl_gpr!(t0, 8);
impl_gpr!(t1, 9);
impl_gpr!(t2, 10);
impl_gpr!(t3, 11);
impl_gpr!(t4, 12);
impl_gpr!(t5, 13);
impl_gpr!(t6, 14);
impl_gpr!(t7, 15);
impl_gpr!(s0, 16);
impl_gpr!(s1, 17);
impl_gpr!(s2, 18);
impl_gpr!(s3, 19);
impl_gpr!(s4, 20);
impl_gpr!(s5, 21);
impl_gpr!(s6, 22);
impl_gpr!(s7, 23);
impl_gpr!(t8, 24);
impl_gpr!(t9, 25);
impl_gpr!(jp, 25); 
impl_gpr!(k0, 26);
impl_gpr!(k1, 27);
impl_gpr!(gp, 28);
impl_gpr!(sp, 29);
impl_gpr!(fp, 30);
impl_gpr!(s8, 30);
impl_gpr!(ra, 31);



use crate::const_export_str;

const_export_str!(ZERO, "$0");
const_export_str!(AT, "$1");
const_export_str!(V0, "$2");
const_export_str!(V1, "$3");
const_export_str!(A0, "$4");
const_export_str!(A1, "$5");
const_export_str!(A2, "$6");
const_export_str!(A3, "$7");
const_export_str!(T0, "$8");
const_export_str!(T1, "$9");
const_export_str!(T2, "$10");
const_export_str!(T3, "$11");
const_export_str!(T4, "$12");
const_export_str!(T5, "$13");
const_export_str!(T6, "$14");
const_export_str!(T7, "$15");
const_export_str!(S0, "$16");
const_export_str!(S1, "$17");
const_export_str!(S2, "$18");
const_export_str!(S3, "$19");
const_export_str!(S4, "$20");
const_export_str!(S5, "$21");
const_export_str!(S6, "$22");
const_export_str!(S7, "$23");
const_export_str!(T8, "$24");
const_export_str!(T9, "$25");
const_export_str!(JP, "$25");
const_export_str!(K0, "$26");
const_export_str!(K1, "$27");
const_export_str!(GP, "$28");
const_export_str!(SP, "$29");
const_export_str!(FP, "$30");
const_export_str!(S8, "$30");
const_export_str!(RA, "$31");