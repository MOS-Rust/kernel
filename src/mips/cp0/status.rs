//! Status register (CP0 Register 12, Select 0) 

#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub bits: u32,
}

impl Status {
    register_struct_bit!(0, ie, set_ie, clear_ie);
    register_struct_bit!(1, exl, set_exl, clear_exl);
    register_struct_bit!(2, erl, set_erl, clear_erl);
    register_struct_bit!(4, um, set_user_mode, set_kernel_mode);

    register_struct_bit!(8, im0, set_im0, clear_im0);
    register_struct_bit!(9, im1, set_im1, clear_im1);
    register_struct_bit!(10, im2, set_im2, clear_im2);
    register_struct_bit!(11, im3, set_im3, clear_im3);
    register_struct_bit!(12, im4, set_im4, clear_im4);
    register_struct_bit!(13, im5, set_im5, clear_im5);
    register_struct_bit!(14, im6, set_im6, clear_im6);
    register_struct_bit!(15, im7, set_im7, clear_im7);
}
register_rw!(12, 0);
register_struct_rw!(Status);

register_bit!(0, ie, set_ie, clear_ie); // Interrupt Enable
register_bit!(1, exl, set_exl, clear_exl); // Exception Level
register_bit!(2, erl, set_erl, clear_erl); // Error Level
register_bit!(4, um, set_user_mode, set_kernel_mode); // User Mode


// Soft interrupt enable bits

register_bit!(8, im0, set_im0, clear_im0);
register_bit!(9, im1, set_im1, clear_im1);

// Hard interrupt enable bits
register_bit!(10, im2, set_im2, clear_im2);
register_bit!(11, im3, set_im3, clear_im3);
register_bit!(12, im4, set_im4, clear_im4);
register_bit!(13, im5, set_im5, clear_im5);
register_bit!(14, im6, set_im6, clear_im6);
register_bit!(15, im7, set_im7, clear_im7);
