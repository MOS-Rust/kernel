use log::trace;

use crate::pm::ENV_MANAGER;

#[no_mangle]
pub extern "C" fn schedule(env_yield: bool) -> ! {
    static mut TOTAL: u32 = 100;
    static mut COUNT: u32 = 0;
    unsafe {
        let mut env = ENV_MANAGER.curenv();
        if env_yield || COUNT == 0 || env.is_none() || !env.as_ref().unwrap().runnable() {
            if let Some(env) = env {
                if env.runnable() {
                    ENV_MANAGER.move_to_end(env);
                }
            }
            if TOTAL == 0 {
                while let Some(new_env) = ENV_MANAGER.get_first() {
                    ENV_MANAGER.env_destroy(new_env);
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
        TOTAL -= 1;
        trace!("Scheduling env: {:x}, runs: {}", env.as_ref().unwrap().id, env.as_ref().unwrap().runs);
        ENV_MANAGER.env_run(env.unwrap());
    }
}
