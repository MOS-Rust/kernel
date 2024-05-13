use crate::pm::ENV_MANAGER;

pub fn schedule(r#yield: bool) {
    static mut COUNT: u32 = 0;
    unsafe {
        if r#yield
            || COUNT == 0
            || ENV_MANAGER.curenv().is_none()
            || !ENV_MANAGER.curenv().unwrap().runnable()
        {

        }
        COUNT-=1;
    }
}
