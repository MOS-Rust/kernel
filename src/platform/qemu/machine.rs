use super::malta::{FPGA_HALT, SERIAL_DATA, SERIAL_DATA_READY, SERIAL_LSR, SERIAL_THR_EMPTY};
use super::KSEG1;

/// Reads a byte from the specified address.
///
/// # Safety
///
/// This function is marked as `unsafe` because it directly reads from a memory address.
///
/// # Arguments
///
/// * `addr` - The address to read from.
///
/// # Returns
///
/// The byte read from the specified address.
unsafe fn read_byte(addr: usize) -> u8 {
    let ptr = addr as *const u8;
    ptr.read_volatile()
}

/// Writes a byte to the specified address.
///
/// # Safety
///
/// This function is marked as `unsafe` because it directly writes to a memory address.
///
/// # Arguments
///
/// * `addr` - The address to write to.
/// * `data` - The byte to write.
unsafe fn write_byte(addr: usize, data: u8) {
    let ptr = addr as *mut u8;
    ptr.write_volatile(data);
}

/// Prints a character to the serial port.
///
/// # Arguments
///
/// * `c` - The character to print.
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

/// Reads a character from the serial port.
///
/// Only characters that are ASCII printable are supported.
/// 
/// # Returns
///
/// The character read from the serial port, or '\0' if no character is available.
pub fn read_char() -> char {
    unsafe {
        if read_byte(KSEG1 + SERIAL_LSR) & SERIAL_DATA_READY != 0 {
            read_byte(KSEG1 + SERIAL_DATA) as char
        } else {
            '\0'
        }
    }
}

/// Halts the system.
///
/// This function writes a specific value to the FPGA_HALT address, causing the system to halt.
/// If halting is not supported on the current platform, this function will loop indefinitely.
pub fn halt() -> ! {
    unsafe {
        write_byte(KSEG1 + FPGA_HALT, 0x42);
    }
    loop {}
}