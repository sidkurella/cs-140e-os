// use crate::common::IO_BASE;

// use volatile::prelude::*;
// use volatile::{Volatile, ReadVolatile};

// /// The base address for the ARM system timer registers.
// const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

// #[repr(C)]
// #[allow(non_snake_case)]
// struct Registers {
//     CS: Volatile<u32>,
//     CLO: ReadVolatile<u32>,
//     CHI: ReadVolatile<u32>,
//     COMPARE: [Volatile<u32>; 4]
// }

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    // registers: &'static mut CNTControlBase
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            // registers: unsafe { &mut *(CNT_CONTROL_BASE as *mut CNTControlBase) }
        }
    }

    /// Reads the system timer's counter and returns the 64-bit counter value.
    /// The returned value is the number of elapsed microseconds.
    #[cfg(not(target_os = "ros"))]
    pub fn read(&self) -> u64 {
        0
    }

    /// Reads the system timer's counter and returns the 64-bit counter value.
    /// The returned value is the number of elapsed microseconds.
    #[inline(never)]
    #[cfg(target_os = "ros")]
    pub fn read(&self) -> u64 {
        let count: u64;
        let interval: u64;

        unsafe {
            asm!("mrs $0, cntpct_el0" : "=r"(count));
            asm!("mrs $0, cntfrq_el0" : "=r"(interval));
        }

        count / (interval / 1000000)
    }

    #[inline(never)]
    pub fn read_tval(&self) -> u64 {
        let count: u64;
        let interval: u64;

        unsafe {
            asm!("mrs $0, cntp_tval_el0" : "=r"(count));
            asm!("mrs $0, cntfrq_el0" : "=r"(interval));
        }
        count / (interval / 1000000)
    }

    /// Sets up a match in timer 1 to occur `us` microseconds from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `us` microseconds.
    pub fn tick_in(&mut self, us: u32) {
        let interval: u64;

        unsafe {
            asm!("mrs $0, cntfrq_el0" : "=r"(interval));
        }

        let tval = interval * (us as u64) / 1000000;

        unsafe {
            asm!("msr cntp_tval_el0, $0" :: "r"(tval));
        }
    }
}

/// Returns the current time in microseconds.
pub fn current_time() -> u64 {
    Timer::new().read()
}

/// Spins until `us` microseconds have passed.
pub fn spin_sleep_us(us: u64) {
    let t = Timer::new();
    let r = t.read();
    let target = r + us;
    loop {
        if t.read() >= target {
            break
        }
    }
}

/// Spins until `ms` milliseconds have passed.
pub fn spin_sleep_ms(ms: u64) {
    spin_sleep_us(ms * 1000);
}

/// Sets up a match in timer 1 to occur `us` microseconds from now. If
/// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
/// interrupt will be issued in `us` microseconds.
pub fn tick_in(us: u32) {
    Timer::new().tick_in(us);
}
