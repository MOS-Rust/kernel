#![allow(dead_code)] // TODO: Remove this

pub mod clock;
pub mod trapframe;
mod handler;

use core::{arch::{asm, global_asm}, ptr::addr_of_mut};

global_asm!(include_str!("../../asm/exception/exception_entry.S"));
global_asm!(include_str!("../../asm/exception/handlers.S"));

#[repr(C)]
pub struct Vector {
    pub handler: unsafe extern "C" fn(),
}

extern "C" {
    fn _handle_int();
    fn _handle_tlb();
    // fn handle_syscall();
    fn _handle_unhandled();
}

#[no_mangle]
pub static exception_handlers: [Vector; 32] = [
    Vector { handler: _handle_int }, // 0: Int
    Vector { handler: _handle_unhandled  }, // 1
    Vector { handler: _handle_tlb }, // 2: TLBL
    Vector { handler: _handle_tlb }, // 3: TLBS
    Vector { handler: _handle_unhandled  }, // 4 
    Vector { handler: _handle_unhandled  }, // 5
    Vector { handler: _handle_unhandled  }, // 6
    Vector { handler: _handle_unhandled  }, // 7
    // Vector { handler: handle_syscall }, // 8: Syscall
    Vector { handler: _handle_unhandled  }, // 8
    Vector { handler: _handle_unhandled  }, // 9
    Vector { handler: _handle_unhandled  }, // 10
    Vector { handler: _handle_unhandled  }, // 11
    Vector { handler: _handle_unhandled  }, // 12
    Vector { handler: _handle_unhandled  }, // 13
    Vector { handler: _handle_unhandled  }, // 14
    Vector { handler: _handle_unhandled  }, // 15
    Vector { handler: _handle_unhandled  }, // 16
    Vector { handler: _handle_unhandled  }, // 17
    Vector { handler: _handle_unhandled  }, // 18
    Vector { handler: _handle_unhandled  }, // 19
    Vector { handler: _handle_unhandled  }, // 20
    Vector { handler: _handle_unhandled  }, // 21
    Vector { handler: _handle_unhandled  }, // 22
    Vector { handler: _handle_unhandled  }, // 23
    Vector { handler: _handle_unhandled  }, // 24
    Vector { handler: _handle_unhandled  }, // 25
    Vector { handler: _handle_unhandled  }, // 26
    Vector { handler: _handle_unhandled  }, // 27
    Vector { handler: _handle_unhandled  }, // 28
    Vector { handler: _handle_unhandled  }, // 29
    Vector { handler: _handle_unhandled  }, // 30
    Vector { handler: _handle_unhandled  }, // 31
];


pub fn init() {
    extern "C" {
        static mut _exception_entry: u8;
    }
    unsafe {
        asm!(
            ".set noat",
            "mtc0 {}, $15, 1",
            ".set at",
            in(reg) addr_of_mut!(_exception_entry) as u32,
        );
    }
}
