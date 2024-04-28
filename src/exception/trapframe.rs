use core::{arch::global_asm, fmt::{Display, Formatter, Result}};

use crate::const_export_usize;

global_asm!(include_str!("../../asm/exception/trapframe.S"));

#[repr(C)]
struct Trapframe {
    regs: [u32; 32],
    status: u32,
    hi: u32,
    lo: u32,
    badvaddr: u32,
    cause: u32,
    epc: u32,
}

impl Display for Trapframe {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Trapframe {{\n")?;
        write!(f, "    regs: [")?;
        for i in 0..32 {
            write!(f, "{:08x}, ", self.regs[i])?;
        }
        write!(f, "]\n")?;
        write!(f, "    status: {:08x}\n", self.status)?;
        write!(f, "    hi: {:08x}\n", self.hi)?;
        write!(f, "    lo: {:08x}\n", self.lo)?;
        write!(f, "    badvaddr: {:08x}\n", self.badvaddr)?;
        write!(f, "    cause: {:08x}\n", self.cause)?;
        write!(f, "    epc: {:08x}\n", self.epc)?;
        write!(f, "}}")
    }
}

const_export_usize!(TF_REG0, 0);
const_export_usize!(TF_REG1, TF_REG0 + 4);
const_export_usize!(TF_REG2, TF_REG1 + 4);
const_export_usize!(TF_REG3, TF_REG2 + 4);
const_export_usize!(TF_REG4, TF_REG3 + 4);
const_export_usize!(TF_REG5, TF_REG4 + 4);
const_export_usize!(TF_REG6, TF_REG5 + 4);
const_export_usize!(TF_REG7, TF_REG6 + 4);
const_export_usize!(TF_REG8, TF_REG7 + 4);
const_export_usize!(TF_REG9, TF_REG8 + 4);
const_export_usize!(TF_REG10, TF_REG9 + 4);
const_export_usize!(TF_REG11, TF_REG10 + 4);
const_export_usize!(TF_REG12, TF_REG11 + 4);
const_export_usize!(TF_REG13, TF_REG12 + 4);
const_export_usize!(TF_REG14, TF_REG13 + 4);
const_export_usize!(TF_REG15, TF_REG14 + 4);
const_export_usize!(TF_REG16, TF_REG15 + 4);
const_export_usize!(TF_REG17, TF_REG16 + 4);
const_export_usize!(TF_REG18, TF_REG17 + 4);
const_export_usize!(TF_REG19, TF_REG18 + 4);
const_export_usize!(TF_REG20, TF_REG19 + 4);
const_export_usize!(TF_REG21, TF_REG20 + 4);
const_export_usize!(TF_REG22, TF_REG21 + 4);
const_export_usize!(TF_REG23, TF_REG22 + 4);
const_export_usize!(TF_REG24, TF_REG23 + 4);
const_export_usize!(TF_REG25, TF_REG24 + 4);
const_export_usize!(TF_REG26, TF_REG25 + 4);
const_export_usize!(TF_REG27, TF_REG26 + 4);
const_export_usize!(TF_REG28, TF_REG27 + 4);
const_export_usize!(TF_REG29, TF_REG28 + 4);
const_export_usize!(TF_REG30, TF_REG29 + 4);
const_export_usize!(TF_REG31, TF_REG30 + 4);
const_export_usize!(TF_STATUS, TF_REG31 + 4);
const_export_usize!(TF_HI, TF_STATUS + 4);
const_export_usize!(TF_LO, TF_HI + 4);
const_export_usize!(TF_BADVADDR, TF_LO + 4);
const_export_usize!(TF_CAUSE, TF_BADVADDR + 4);
const_export_usize!(TF_EPC, TF_CAUSE + 4);
const_export_usize!(TF_SIZE, TF_EPC + 4);
