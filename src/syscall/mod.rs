use log::debug;

use crate::exception::trapframe::Trapframe;


#[no_mangle]
pub unsafe extern "C" fn do_syscall(tf: *mut Trapframe) {
    debug!("Syscall\n{}", &*tf);
    let syscall_num: u32 = (*tf).regs[4];
    let arg1: u32 = (*tf).regs[5];
    let arg2: u32 = (*tf).regs[6];
    let arg3: u32 = (*tf).regs[7];
    let sp: u32 = (*tf).regs[29];
    let arg4: u32 = *(sp as *const u32).add(4);
    let arg5: u32 = *(sp as *const u32).add(5);
    debug!("Syscall number: {}", syscall_num);
    debug!("Args: {:x} {:x} {:x} {:x} {:x}", arg1, arg2, arg3, arg4, arg5);
}