use core::fmt;

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

use timer;
use common::IO_BASE;
use gpio::{Gpio, Function};

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IO: Volatile<u8>,
    __r0: [Reserved<u8>; 3],
    IER: Volatile<u8>,
    __r1: [Reserved<u8>; 3],
    IIR: Volatile<u8>,
    __r2: [Reserved<u8>; 3],
    LCR: Volatile<u8>,
    __r3: [Reserved<u8>; 3],
    MCR: Volatile<u8>,
    __r4: [Reserved<u8>; 3],
    LSR: ReadVolatile<u8>,
    __r5: [Reserved<u8>; 3],
    MSR: ReadVolatile<u8>,
    __r6: [Reserved<u8>; 3],
    SCRATCH: Volatile<u8>,
    __r7: [Reserved<u8>; 3],
    CNTL: Volatile<u8>,
    __r8: [Reserved<u8>; 3],
    STAT: ReadVolatile<u32>,
    BAUD: Volatile<u16>,
    __r9: Reserved<u16>
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<u32>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // Set UART to 8 bit mode.
        registers.LCR.write(0b11);
        // Set baud rate to 115200 (divisor of 270).
        registers.BAUD.write(270);
        // Enable UART TX and RX.
        registers.CNTL.write(0b11);

        // Set GPIO pins 14 and 15 to Alt 5 function.
        Gpio::new(14).into_alt(Function::Alt5);
        Gpio::new(15).into_alt(Function::Alt5);

        MiniUart {
            registers: registers,
            timeout: None
        }
    }

    /// Set the read timeout to `milliseconds` milliseconds.
    pub fn set_read_timeout(&mut self, milliseconds: u32) {
        self.timeout = Some(milliseconds);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        // Would it be better to just wait for the whole FIFO to be empty?

        // How many bits are left to write out.
        let mut bit_ct = 8;
        // Bits to write, shifting off written bits.
        let mut bits = byte;

        while bit_ct > 0 {
            while self.registers.LSR.read() & LsrStatus::TxAvailable as u8 == 0 {
                // Spin while TX FIFO is full.
            }

            // Get how much space is left in TX FIFO.
            let tx_space = 8 - ((self.registers.STAT.read() >> 24) & 0b111);

            // How many bits can we maximally add to the FIFO?
            let ct = if bit_ct > tx_space { tx_space } else { bit_ct };

            // Get that many bits from the target byte.
            let to_write = bits & !(1 << ct);
            bits >>= ct;
            bit_ct -= ct;

            // Add to FIFO.
            self.registers.IO.write(to_write);
        }
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        // Get how much space is left in RX FIFO.
        ((self.registers.STAT.read() >> 16) & 0b111) == 8
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        let start = timer::current_time();
        let target = match self.timeout {
            Some(ms) => start + (ms as u64 * 1000),
            None => (1 << 63) // This will not lapse for 300000 years.
        };

        while timer::current_time() <= target {
            // Spin while timeout pending.
            if self.has_byte() {
                return Ok(());
            }
        }
        // Timeout lapsed.
        Err(())
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {
            // Spin while waiting for a byte.
        }
        self.registers.IO.read()
    }
}

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for b in s.as_bytes() {
            if *b == b'\n' {
                self.write_byte(b'\r');
                self.write_byte(b'\n');
            } else {
                self.write_byte(*b);
            }
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
mod uart_io {
    use std::io;
    use super::MiniUart;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.

    impl io::Read for MiniUart {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            // How do I know when the data stream is over?
            match self.wait_for_byte() {
                Err(()) => Err(io::Error::new(
                            io::ErrorKind::TimedOut, "Read timed out."
                        )),
                Ok(()) => for b in buf.iter_mut() {
                    *b = self.read_byte();
                }
            }
            Ok(buf.len())
        }
    }

    impl io::Write for MiniUart {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for b in buf {
                self.write_byte(*b);
            }
            Ok(buf.len())
        }
    }
}
