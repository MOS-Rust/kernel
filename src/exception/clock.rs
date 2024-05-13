use mips::registers::cp0::{compare, count};

const TIMER_INTERVAL: u32 = 500000;

#[inline]
pub unsafe fn reset_kclock() {
    compare::write(TIMER_INTERVAL);
    count::write(0);
}