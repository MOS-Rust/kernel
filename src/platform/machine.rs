use super::malta::{FPGA_HALT, SERIAL_DATA, SERIAL_DATA_READY, SERIAL_LSR, SERIAL_THR_EMPTY};
use crate::mm::layout::KSEG1;

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
#[inline(always)]
pub unsafe fn ioread_byte(pa: usize) -> u8 {
    let pa = KSEG1 | pa;
    let ptr = pa as *const u8;
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
#[inline(always)]
pub unsafe fn iowrite_byte(pa: usize, data: u8) {
    let pa = KSEG1 | pa;
    let ptr = pa as *mut u8;
    ptr.write_volatile(data);
}

#[inline(always)]
pub unsafe fn ioread_half(pa: usize) -> u16 {
    let pa = KSEG1 | pa;
    let ptr = pa as *const u16;
    ptr.read_volatile()
}

#[inline(always)]
pub unsafe fn iowrite_half(pa: usize, data: u16) {
    let pa = KSEG1 | pa;
    let ptr = pa as *mut u16;
    ptr.write_volatile(data);
}

#[inline(always)]
pub unsafe fn ioread_word(pa: usize) -> u32 {
    let pa = KSEG1 | pa;
    let ptr = pa as *const u32;
    ptr.read_volatile()
}

#[inline(always)]
pub unsafe fn iowrite_word(pa: usize, data: u32) {
    let pa = KSEG1 | pa;
    let ptr = pa as *mut u32;
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
        while ioread_byte(SERIAL_LSR) & SERIAL_THR_EMPTY == 0 {}
        if c.is_ascii() {
            iowrite_byte(SERIAL_DATA, c as u8);
        } else {
            let mut dst = [0; 4];
            c.encode_utf8(&mut dst);
            for &byte in dst.iter() {
                iowrite_byte(SERIAL_DATA, byte);
            }
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
        if ioread_byte(SERIAL_LSR) & SERIAL_DATA_READY != 0 {
            ioread_byte(SERIAL_DATA) as char
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
        iowrite_byte(FPGA_HALT, 0x42);
    }
    unreachable!()
}
