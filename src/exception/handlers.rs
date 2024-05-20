use log::warn;

use crate::pm::{schedule::schedule, ENV_MANAGER};

use super::trapframe::Trapframe;

#[no_mangle]
pub unsafe extern "C" fn do_address_error(tf: *mut Trapframe) {
    // Kill the process
    let tf = &*tf;
    // AdEL for false, AdES for true
    let extype = ((tf.cp0_cause >> 2) & 0x3f ) == 5;
    if let Some(env) = ENV_MANAGER.curenv() {
        let msg = if extype { "AdES" } else { "AdEL" };
        warn!("{:08x}: {} at 0x{:08x} for 0x{:08x}, killing...", env.id, msg, tf.cp0_epc, tf.cp0_badvaddr);
        ENV_MANAGER.clear_curenv();
        ENV_MANAGER.env_destroy(env);
        schedule(true);
    } else {
        panic!("Address error\n {}", tf);
    }
}

#[no_mangle]
pub extern "C" fn do_unhandled(tf: *mut Trapframe) {
    let tf = unsafe { &*tf };
    panic!("Unhandled exception\n {}", tf);
}
