use crate::mm::layout::KSEG1;
use super::malta::{FPGA_HALT, SERIAL_DATA, SERIAL_DATA_READY, SERIAL_LSR, SERIAL_THR_EMPTY};

unsafe fn read_byte(addr: usize) -> u8 {
    let ptr = addr as *const u8;
    ptr.read_volatile()
}

unsafe fn write_byte(addr: usize, data: u8) {
    let ptr = addr as *mut u8;
    ptr.write_volatile(data);
}

pub fn print_char(c: char) {
    if c == '\n' {
        print_char('\r');
    }
    unsafe {
        while read_byte(KSEG1 + SERIAL_LSR) & SERIAL_THR_EMPTY == 0 {}
        let mut buf = [0; 4];
        c.encode_utf8(&mut buf);
        for byte in buf {
            write_byte(KSEG1 + SERIAL_DATA, byte);
        }
    }
}

pub fn read_char() -> char {
    unsafe {
        if read_byte(KSEG1 + SERIAL_LSR) & SERIAL_DATA_READY != 0 {
            read_byte(KSEG1 + SERIAL_DATA) as char
        } else {
            '\0'
        }
    }
}

pub fn halt() -> ! {
    unsafe {
        write_byte(KSEG1 + FPGA_HALT, 0x42);
    }
    loop {}
}