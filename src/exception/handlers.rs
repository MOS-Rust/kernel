use crate::pm::schedule::schedule;

use super::trapframe::Trapframe;

extern "C" {
    pub fn _ret_from_exception() -> !;
}

pub fn handle_int() -> ! {
    schedule(true);
}

pub fn handle_tlb() -> ! {
    extern "C" {
        fn _do_tlb_refill();
    }
    unsafe {
        _do_tlb_refill();
        _ret_from_exception();
    }
}

pub fn handle_syscall(_tf: *mut Trapframe) {
}

pub fn unhandled(tf: *mut Trapframe) -> ! {
    panic!("Unhandled exception.\n{}", unsafe { *tf });
}