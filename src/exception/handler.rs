#[no_mangle]
pub extern "C" fn do_unhandled() {
    panic!("Unhandled exception");
}
