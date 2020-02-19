use std::io;
use fat32::traits::BlockDevice;
use pi::timer;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

#[no_mangle]
fn wait_micros(us: u32) {
    timer::spin_sleep_us(us as u64);
}

#[derive(Debug)]
pub enum Error {
    TimedOut,
    CommandFailure,
    Other(i64)
}

impl From<i64> for Error {
    fn from(e: i64) -> Error {
        match e {
            -1 => Error::TimedOut,
            -2 => Error::CommandFailure,
             c => Error::Other(c)
        }
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        match e {
            Error::TimedOut => io::Error::new(
                io::ErrorKind::TimedOut, "SD timeout"
            ),
            Error::CommandFailure => io::Error::new(
                io::ErrorKind::Other, "SD command send failure"
            ),
            Error::Other(code) => io::Error::new(
                io::ErrorKind::Other, "SD miscellaneous error"
            ),
        }
    }
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    pub fn new() -> Result<Sd, Error> {
        let code = unsafe { sd_init() };
        if code == 0 {
            Ok(Sd)
        } else {
            Err(From::from(code as i64))
        }
    }
}

impl BlockDevice for Sd {
    /// Reads sector `n` from the SD card into `buf`. On success, the number of
    /// bytes read is returned.
    ///
    /// # Errors
    ///
    /// An I/O error of kind `InvalidInput` is returned if `buf.len() < 512` or
    /// `n > 2^31 - 1` (the maximum value for an `i32`).
    ///
    /// An error of kind `TimedOut` is returned if a timeout occurs while
    /// reading from the SD card.
    ///
    /// An error of kind `Other` is returned for all other errors.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 512 {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput, "SD read: buffer is too short"
            ))
        } else if n > (1 << 31) - 1 {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput, "read is too large"
            ))
        } else {
            match unsafe { sd_readsector(n as i32, buf.as_mut_ptr()) } {
                0 => Err(From::<Error>::from(From::from(unsafe { sd_err }))),
                bytes => Ok(bytes as usize)
            }
        }
    }

    fn write_sector(&mut self, _n: u64, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("SD card and file system are read only")
    }
}
