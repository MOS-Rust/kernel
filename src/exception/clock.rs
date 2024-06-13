use core::arch::asm;

const TIMER_INTERVAL: u32 = 500_000;

/// reset clock with inline asm
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
