#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api)]
#![feature(raw_vec_internals)]


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
    ALLOCATOR.initialize();

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

    {
        let p = vec![0; 10];
        kprintln!("{:?}", p);

        let q = vec![0..10];
        kprintln!("{:?}", q);

        let mut r : Vec<usize> = (0..200000).collect();
        r.remove(24);
        kprintln!("{:?}", r[0]);
    }

    kprintln!("OK");
}
