use log::warn;

use crate::pm::{env_destroy, schedule, ENV_MANAGER};
use crate::mutex::Mutex;

use super::trapframe::Trapframe;

/// Execute when address error occurs
#[no_mangle]
pub unsafe extern "C" fn do_address_error(tf: *mut Trapframe) {
    // Kill the process
    let tf = &*tf;
    // AdEL for false, AdES for true
    let extype = ((tf.cp0_cause >> 2) & 0x3f) == 5;
    if let Some(env) = ENV_MANAGER.lock().curenv() {
        let msg = if extype { "AdES" } else { "AdEL" };
        warn!(
            "{:08x}: {} at 0x{:08x} for 0x{:08x}, killing...",
            env.id, msg, tf.cp0_epc, tf.cp0_badvaddr
        );
        env_destroy(env);
        schedule(true);
    } else {
        panic!("Address error\n {}", tf);
    }
}

/// Execute when undefined error occurs
#[no_mangle]
pub extern "C" fn do_unhandled(tf: *mut Trapframe) {
    let tf = unsafe { &*tf };
    panic!(
        "Unhandled exception,\n Exception type: {},\n {}",
        ((tf.cp0_cause >> 2) & 0x3f),
        tf
    );
}
