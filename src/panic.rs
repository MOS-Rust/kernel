
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}