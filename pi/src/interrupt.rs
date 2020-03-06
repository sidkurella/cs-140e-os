use crate::common::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    EL1PhysTimer = 30,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

impl Interrupt {
    fn to_reg_bit(&self) -> (usize, usize) {
        let num = *self as usize;
        (num / 32, num % 32)
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    /// IRQ basic pending
    IRQ_Pend_Basic: ReadVolatile<u32>,
    /// IRQ pending 1 + 2
    IRQ_Pend: [ReadVolatile<u32>; 2],
    /// FIQ control
    FIQ_Control: Volatile<u32>,
    /// Enable IRQs 1 + 2
    Enable_IRQ: [Volatile<u32>; 2],
    /// Enable Basic IRQs
    Enable_IRQ_Basic: Volatile<u32>,
    /// Disable IRQs 1 + 2
    Disable_IRQ: [Volatile<u32>; 2],
    /// Disable Basic IRQs
    Disable_IRQ_Basic: Volatile<u32>
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let (idx, bit) = int.to_reg_bit();
        let reg = &mut self.registers.Enable_IRQ[idx];
        reg.write(reg.read() | (1 << bit));
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let (idx, bit) = int.to_reg_bit();
        let reg = &mut self.registers.Disable_IRQ[idx];
        reg.write(reg.read() | (1 << bit));
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let (reg, bit) = int.to_reg_bit();
        self.registers.IRQ_Pend[reg].read() & (1 << bit) != 0
    }
}
