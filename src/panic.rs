//! Panic handler

use crate::println;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Kernel Panicked: \"{}\" at {}", _info.message().unwrap(), _info.location().unwrap());
    loop {}
}