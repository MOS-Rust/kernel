//! Malta board specific definitions.
//! Copied from MOS include/malta.h

/*
 * QEMU MMIO address definitions.
 */
const PCIIO_BASE: usize = 0x18000000;
const FPGA_BASE: usize = 0x1f000000;

/*
 * 16550 Serial UART device definitions.
 */
pub const SERIAL_BASE: usize = PCIIO_BASE + 0x3f8;
pub const SERIAL_DATA: usize = SERIAL_BASE;
pub const SERIAL_LSR: usize = SERIAL_BASE + 0x5;
pub const SERIAL_DATA_READY: u8 = 0x1;
pub const SERIAL_THR_EMPTY: u8 = 0x20;

/*
 * Intel PIIX4 IDE Controller device definitions.
 */
pub const IDE_BASE: usize = PCIIO_BASE + 0x01f0;
pub const IDE_DATA: usize = IDE_BASE;
pub const IDE_ERR: usize = IDE_BASE + 0x01;
pub const IDE_NSECT: usize = IDE_BASE + 0x02;
pub const IDE_LBAL: usize = IDE_BASE + 0x03;
pub const IDE_LBAM: usize = IDE_BASE + 0x04;
pub const IDE_LBAH: usize = IDE_BASE + 0x05;
pub const IDE_DEVICE: usize = IDE_BASE + 0x06;
pub const IDE_STATUS: usize = IDE_BASE + 0x07;
pub const IDE_LBA: u8 = 0xE0;
pub const IDE_BUSY: u8 = 0x80;
pub const IDE_CMD_PIO_READ: u8 = 0x20;
pub const IDE_CMD_PIO_WRITE: u8 = 0x30;

/*
 * MALTA Power Management device definitions.
 */
pub const FPGA_HALT: usize = FPGA_BASE + 0x500;
