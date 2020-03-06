use pi::interrupt::Interrupt;
use pi::timer;

use crate::traps::TrapFrame;
use crate::process::TICK;
use crate::console::kprintln;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    kprintln!("Interrupt: {:?}", interrupt);
    timer::tick_in(TICK);
}
