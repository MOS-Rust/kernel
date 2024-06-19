use core::sync::atomic::{AtomicU32, Ordering};

use crate::mutex::Mutex;
use crate::pm::env::env_run;

use super::ENV_MANAGER;
use log::trace;

/// Implementation of schedule in mos
/// Implement a round-robin scheduling to select a runnable env and schedule it using 'env_run'
#[no_mangle]
pub extern "C" fn schedule(env_yield: bool) -> ! {
    static COUNT: AtomicU32 = AtomicU32::new(0);
    let mut env = ENV_MANAGER.lock().curenv();
    if env_yield
        || COUNT.load(Ordering::SeqCst) == 0
        || env.is_none()
        || !env.as_ref().unwrap().runnable()
    {
        if let Some(env) = env {
            if env.runnable() {
                ENV_MANAGER.lock().move_to_end(env);
            }
        }
        if let Some(new_env) = ENV_MANAGER.lock().get_first() {
            COUNT.store(new_env.priority, Ordering::SeqCst);
            env = Some(new_env);
        } else {
            panic!("No runnable envs")
        }
    }
    COUNT.fetch_sub(1, Ordering::SeqCst);
    trace!(
        "Scheduling env: {:08x}, runs: {}",
        env.as_ref().unwrap().id,
        env.as_ref().unwrap().runs
    );
    env_run(env.unwrap())
}
