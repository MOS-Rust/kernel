//! Clock module for handling timer interrupt.
use core::arch::asm;

const TIMER_INTERVAL: u32 = 500_000;

/// Reset the CP0 Count and Compare registers for timer interrupt.
/// 
/// # Safety
/// 
/// This function is unsafe because it uses inline assembly.
#[inline(always)]
pub unsafe fn reset_kclock() {
    asm!(
        ".set noat",
        "mtc0 {}, $11",
        "mtc0 $zero, $9",
        ".set at",
        in(reg) TIMER_INTERVAL,
    );
}
