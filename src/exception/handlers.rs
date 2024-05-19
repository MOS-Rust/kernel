use super::trapframe::Trapframe;

#[no_mangle]
pub extern "C" fn do_unhandled(tf: *mut Trapframe) {
    let tf = unsafe { &*tf };
    panic!("Unhandled exception\n {}", tf);
}
