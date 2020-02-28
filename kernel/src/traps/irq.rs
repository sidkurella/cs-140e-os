use pi::interrupt::Interrupt;

use crate::traps::TrapFrame;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    unimplemented!("handle_irq()")
}
