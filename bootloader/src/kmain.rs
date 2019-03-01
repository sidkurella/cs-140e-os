#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

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

pub fn boot() {
    let mut mu = pi::uart::MiniUart::new();
    loop {
        use std::fmt::Write;
        mu.write_str("a");
    }
}

#[no_mangle]
pub unsafe extern "C" fn kmain() {
    std::ptr::copy(BINARY_START, BOOTLOADER_START, BOOTLOADER_SIZE);
    jump_to(std::mem::transmute(boot as *const fn () -> ()));
}
