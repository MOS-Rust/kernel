use crate::pm::ENV_MANAGER;

pub fn schedule(r#yield: bool) {
    static mut COUNT: u32 = 0;
    unsafe {
        let mut env = ENV_MANAGER.curenv();
        if r#yield || COUNT == 0 || env.is_none() || !env.as_ref().unwrap().runnable() {
            if env.as_ref().is_some() && env.as_ref().unwrap().runnable() {
                ENV_MANAGER.move_to_end(env.unwrap());
            }
            if let Some(new_env) = ENV_MANAGER.get_first() {
                COUNT = new_env.priority; 
                env = Some(new_env);
            } else {
                panic!("No runnable envs")
            }
        }
        COUNT -= 1;
        ENV_MANAGER.env_run(env.unwrap());
    }
}
