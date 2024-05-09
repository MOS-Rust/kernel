pub struct Trapframe {
    regs: [u32; 32],

    cp0_status: u32,
    hi: u32,
    lo: u32,
    cp0_badvaddr: u32,
    cp0_cause: u32,
    cp0_epc: u32,
}