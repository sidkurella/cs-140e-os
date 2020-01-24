use crate::os::raw::c_char;

pub fn decode_error_kind(_errno: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Other
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Void {}

pub fn strlen(string: *const c_char) -> usize {
    let mut size = 0;
    while unsafe { *(string.offset(size as isize)) } != 0 {
        size += 1;
    }

    size
}

pub mod alloc;
pub mod io;
pub mod os;
pub mod os_str;
pub mod path;
pub mod stdio;

pub unsafe fn abort_internal() -> ! {
    panic!("ABORT");
}
