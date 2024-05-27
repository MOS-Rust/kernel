use super::ENV_MANAGER;
use log::trace;

/// Implementation of schedule in mos
/// Implement a round-robin scheduling to select a runnable env and schedule it using 'env_run'
#[no_mangle]
pub unsafe extern "C" fn schedule(env_yield: bool) -> ! {
    static mut COUNT: u32 = 0;
    let mut env = ENV_MANAGER.curenv();
    if env_yield || COUNT == 0 || env.is_none() || !env.as_ref().unwrap().runnable() {
        if let Some(env) = env {
            if env.runnable() {
                ENV_MANAGER.move_to_end(env);
            }
        }
        if let Some(new_env) = ENV_MANAGER.get_first() {
            COUNT = new_env.priority;
            env = Some(new_env);
        } else {
            panic!("No runnable envs")
        }
    }
    COUNT -= 1;
    trace!(
        "Scheduling env: {:08x}, runs: {}",
        env.as_ref().unwrap().id,
        env.as_ref().unwrap().runs
    );
    ENV_MANAGER.env_run(env.unwrap());
}
