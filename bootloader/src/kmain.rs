#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

use std::fmt::Write;
use std::io;

pub mod lang_items;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Pointer to where the bootloader should actually go.
const BOOTLOADER_START: *mut u8 = BOOTLOADER_START_ADDR as *mut u8;

/// Assuming the size of the bootloader!
const BOOTLOADER_SIZE: usize = 0x40000;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

pub fn boot() -> ! {
    let mut console = pi::uart::MiniUart::new();

    loop {
        let output = unsafe { std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) };
        let mut uart = pi::uart::MiniUart::new();
        uart.set_read_timeout(750);

        match xmodem::Xmodem::receive(uart, output) {
            Ok(_) => {
                write!(&mut console, "load complete").unwrap();
                jump_to(BINARY_START)
            },
            Err(e) => if e.kind() != io::ErrorKind::TimedOut {
                write!(&mut console, "failed receive: {:?}\n", e).unwrap();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn kmain() {
    std::ptr::copy(BINARY_START, BOOTLOADER_START, BOOTLOADER_SIZE);

    let addr : usize = std::mem::transmute(boot as *const fn () -> ());
    jump_to(std::mem::transmute(addr - BINARY_START_ADDR + BOOTLOADER_START_ADDR));
}
