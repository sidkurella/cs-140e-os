#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(exclusive_range_pattern)]
#![feature(allocator_api)]
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

// use console::{kprint, kprintln, CONSOLE};
use console::kprintln;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    ALLOCATOR.initialize();
    kprintln!("Initialize OK");

    kprintln!("
  ██████╗  ██████╗ ██╗  ██╗██╗   ██╗ ██████╗ ███████╗
  ╚════██╗██╔═████╗╚██╗██╔╝╚██╗ ██╔╝██╔═══██╗██╔════╝
   █████╔╝██║██╔██║ ╚███╔╝  ╚████╔╝ ██║   ██║███████╗
  ██╔═══╝ ████╔╝██║ ██╔██╗   ╚██╔╝  ██║   ██║╚════██║
  ███████╗╚██████╔╝██╔╝ ██╗   ██║   ╚██████╔╝███████║
  ╚══════╝ ╚═════╝ ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚══════╝
");

    {
        let mut v = vec![1, 2, 3];
        kprintln!("{:?}", v);
        kprintln!("Alloc OK");

        v.push(4);
        kprintln!("{:?}", v);
        kprintln!("Push OK");

        v.pop();
        kprintln!("{:?}", v);
        kprintln!("Pop OK");
    }

    kprintln!("Drop OK");

    {
        let mut v = vec![1, 2, 3];
        kprintln!("{:?}", v);
        kprintln!("Alloc OK");

        v.push(4);
        kprintln!("{:?}", v);
        kprintln!("Push OK");

        v.pop();
        kprintln!("{:?}", v);
        kprintln!("Pop OK");
    }

    kprintln!("Drop OK");

    loop { unsafe { asm!("wfe" :::: "volatile"); } }
}
