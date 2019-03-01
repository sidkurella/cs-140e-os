use console::{kprintln, CONSOLE};
use std::fmt::Write;

#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]

pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    kprintln!("
            (
       (      )     )
         )   (    (
        (          `
    .-''^'''^''^'''^''-.
  (//\\\\//\\\\//\\\\//\\\\//\\\\//)
   ~\\^^^^^^^^^^^^^^^^^^/~
     `================`

      The pi is burnt.
");
    kprintln!("
---------- PANIC ----------

FILE: {}
LINE: {}
COL: {}

", file, line, col);

    let mut console = CONSOLE.lock();
    console.write_fmt(fmt).unwrap();

    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
