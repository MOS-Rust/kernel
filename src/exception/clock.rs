use mips::registers::cp0::{compare, count};

const TIMER_INTERVAL: u32 = 500_000;

pub unsafe fn reset_kclock() {
    compare::write(TIMER_INTERVAL);
    count::write(0);
}