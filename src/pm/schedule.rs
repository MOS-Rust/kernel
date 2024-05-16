use crate::{pm::ENV_MANAGER, println};

#[no_mangle]
pub extern "C" fn schedule(env_yield: bool) -> ! {
    unsafe { println!("TOTAL: {}", TOTAL); }
    static mut TOTAL: u32 = 100;
    static mut COUNT: u32 = 0;
    unsafe {
        let mut env = ENV_MANAGER.curenv();
        if env_yield || COUNT == 0 || env.is_none() || !env.as_ref().unwrap().runnable() {
            if env.as_ref().is_some() && env.as_ref().unwrap().runnable() {
                ENV_MANAGER.move_to_end(env.unwrap());
            }
            if TOTAL == 0 {
                if let Some(new_env) = ENV_MANAGER.get_first() {
                    ENV_MANAGER.env_destroy(new_env);
                }
                TOTAL = 100;
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
        ENV_MANAGER.env_run(env.unwrap());
    }
}
