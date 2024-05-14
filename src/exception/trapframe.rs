use core::{arch::global_asm, fmt::{Display, Formatter, Result}};

use crate::{const_export_usize, mm::addr::VA};

global_asm!(include_str!("../../asm/exception/trapframe.S"));

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Trapframe {
    pub regs: [u32; 32],
    pub cp0_status: u32,
    pub hi: u32,
    pub lo: u32,
    pub cp0_badvaddr: u32,
    pub cp0_cause: u32,
    pub cp0_epc: u32,
}

impl Trapframe {
    pub const fn new() -> Trapframe {
        Trapframe {
            regs: [0; 32],
            cp0_status: 0,
            hi: 0,
            lo: 0,
            cp0_badvaddr: 0,
            cp0_cause: 0,
            cp0_epc: 0,
        }
    }

    pub unsafe fn from_memory(addr: VA) -> *mut Trapframe {
        addr.0 as *mut Trapframe
    }
}

impl Display for Trapframe {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Trapframe {{\n")?;
        write!(f, "    regs: [")?;
        for i in 0..32 {
            write!(f, "{:08x}, ", self.regs[i])?;
        }
        write!(f, "]\n")?;
        write!(f, "    status: {:08x}\n", self.cp0_status)?;
        write!(f, "    hi: {:08x}\n", self.hi)?;
        write!(f, "    lo: {:08x}\n", self.lo)?;
        write!(f, "    badvaddr: {:08x}\n", self.cp0_badvaddr)?;
        write!(f, "    cause: {:08x}\n", self.cp0_cause)?;
        write!(f, "    epc: {:08x}\n", self.cp0_epc)?;
        write!(f, "}}")
    }
}

const_export_usize!(TF_REG0, 0x0);
const_export_usize!(TF_REG1, 0x4);
const_export_usize!(TF_REG2, 0x8);
const_export_usize!(TF_REG3, 0xC);
const_export_usize!(TF_REG4, 0x10);
const_export_usize!(TF_REG5, 0x14);
const_export_usize!(TF_REG6, 0x18);
const_export_usize!(TF_REG7, 0x1C);
const_export_usize!(TF_REG8, 0x20);
const_export_usize!(TF_REG9, 0x24);
const_export_usize!(TF_REG10, 0x28);
const_export_usize!(TF_REG11, 0x2C);
const_export_usize!(TF_REG12, 0x30);
const_export_usize!(TF_REG13, 0x34);
const_export_usize!(TF_REG14, 0x38);
const_export_usize!(TF_REG15, 0x3C);
const_export_usize!(TF_REG16, 0x40);
const_export_usize!(TF_REG17, 0x44);
const_export_usize!(TF_REG18, 0x48);
const_export_usize!(TF_REG19, 0x4C);
const_export_usize!(TF_REG20, 0x50);
const_export_usize!(TF_REG21, 0x54);
const_export_usize!(TF_REG22, 0x58);
const_export_usize!(TF_REG23, 0x5C);
const_export_usize!(TF_REG24, 0x60);
const_export_usize!(TF_REG25, 0x64);
const_export_usize!(TF_REG26, 0x68);
const_export_usize!(TF_REG27, 0x6C);
const_export_usize!(TF_REG28, 0x70);
const_export_usize!(TF_REG29, 0x74);
const_export_usize!(TF_REG30, 0x78);
const_export_usize!(TF_REG31, 0x7C);
const_export_usize!(TF_STATUS, 0x80);
const_export_usize!(TF_HI, 0x84);
const_export_usize!(TF_LO, 0x88);
const_export_usize!(TF_BADVADDR, 0x8C);
const_export_usize!(TF_CAUSE, 0x90);
const_export_usize!(TF_EPC, 0x94);
const_export_usize!(TF_SIZE, 0x98);
