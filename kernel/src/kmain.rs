#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

use console::{kprint, kprintln, CONSOLE};
use pi::atags;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    kprintln!("
  ██████╗  ██████╗ ██╗  ██╗██╗   ██╗ ██████╗ ███████╗
  ╚════██╗██╔═████╗╚██╗██╔╝╚██╗ ██╔╝██╔═══██╗██╔════╝
   █████╔╝██║██╔██║ ╚███╔╝  ╚████╔╝ ██║   ██║███████╗
  ██╔═══╝ ████╔╝██║ ██╔██╗   ╚██╔╝  ██║   ██║╚════██║
  ███████╗╚██████╔╝██╔╝ ██╗   ██║   ╚██████╔╝███████║
  ╚══════╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚══════╝
");
    //assert_eq!(1, 2);
    //shell::shell("> ");
    for a in atags::Atags::get() {
        kprintln!("{:#?}", a);
    }
    kprintln!("Atags done");
}
