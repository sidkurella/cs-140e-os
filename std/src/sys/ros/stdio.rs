use crate::io::{self, IoSlice};

// pub struct Stdin(());
// pub struct Stdout(());
pub struct Stderr(());

// impl Stdin {
//     pub fn new() -> io::Result<Stdin> {
//         Ok(Stdin(()))
//     }
// }
// 
// impl io::Read for Stdin {
//     fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
//         Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
//     }
// 
//     fn read_vectored(&mut self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
//         Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
//     }
// }
// 
// impl Stdout {
//     pub fn new() -> io::Result<Stdout> {
//         Ok(Stdout(()))
//     }
// }
// 
// impl io::Write for Stdout {
//     fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
//         Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
//     }
// 
//     fn write_vectored(&mut self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
//         Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
//     }
// 
//     fn flush(&mut self) -> io::Result<()> {
//         Ok(())
//     }
// }
// 
impl Stderr {
    pub fn new() -> io::Result<Stderr> {
        Ok(Stderr(()))
    }
}

impl io::Write for Stderr {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
    }

    fn write_vectored(&mut self, _bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "unsupported"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
// 
// pub fn is_ebadf(_err: &io::Error) -> bool {
//     false
// }
// 
// pub const STDIN_BUF_SIZE: usize = crate::sys_common::io::DEFAULT_BUF_SIZE;

pub fn panic_output() -> Option<impl io::Write> {
    Stderr::new().ok()
}
