mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use crate::console::kprintln;
use crate::shell::shell;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    if info.kind == Kind::Irq {
        let interrupts = [
            Interrupt::Timer1,
            Interrupt::Timer3,
            Interrupt::Usb,
            Interrupt::EL1PhysTimer,
            Interrupt::Gpio0,
            Interrupt::Gpio1,
            Interrupt::Gpio2,
            Interrupt::Gpio3,
            Interrupt::Uart
        ];
        let controller = Controller::new();
        for int in interrupts.iter() {
            if controller.is_pending(*int) {
                return handle_irq(*int, tf);
            }
        }
    } else {
        let syn = Syndrome::from(esr);

        kprintln!("== EXCEPTION HANDLER ==");
        kprintln!("SYNDROME: {:?}", syn);
        kprintln!("INFO: {:#?}", info);
        kprintln!("TF: {:#?}", tf);

        shell("except> ");

        tf.elr += 4;
    }
}
