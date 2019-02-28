use std::fmt::Write;

use console::{kprint, kprintln, CONSOLE};
use stack_vec::StackVec;

const MAX_CMDLEN : usize = 512;
const MAX_ARGLEN : usize = 64;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

#[derive(Debug)]
enum HandleError {
    NoSuchCommand
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        assert!(!self.args.is_empty());
        self.args[0]
    }

    fn handle(&self) -> Result<(), HandleError> {
        if self.path() == "echo" {
            use std::ops::Deref;
            let mut first = true;
            for arg in &self.args[1..] {
                if !first {
                    kprint!(" ");
                }
                kprint!("{}", arg);
                first = false;
            }
            kprintln!();
            Ok(())
        } else {
            Err(HandleError::NoSuchCommand)
        }
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let mut input_buf = [0; MAX_CMDLEN];
    let mut input_vec = StackVec::new(&mut input_buf);

    loop {
        {
            let mut console = CONSOLE.lock();
            console.write_str(prefix).expect("failed to write prefix");

            loop {
                let input = console.read_byte();

                if input == b'\n' || input == b'\r' { // newline
                    console.write_byte(b'\r');
                    console.write_byte(b'\n');
                    break
                } else if input == b'\x7f' { // delete / backspace
                    if let Some(_) = input_vec.pop() {
                        console.write_byte(b'\x08');
                        console.write_byte(b' ');
                        console.write_byte(b'\x08');
                    }
                } else if input < 32 { // unprintable uninterpreted
                    console.write_byte(b'\x07');
                } else { // regular character
                    if let Ok(_) = input_vec.push(input) {
                        console.write_byte(input);
                    }
                }
            }
        }

        {
            let input_str = std::str::from_utf8(input_vec.as_slice())
                                    .expect("failed to decode utf8");
            let mut input_args = [input_str; MAX_ARGLEN];

            match Command::parse(&input_str, &mut input_args[..]) {
                Err(Error::Empty) => {},
                Err(Error::TooManyArgs) => kprintln!("too many arguments"),
                Ok(command) => match command.handle() {
                    Ok(_) => { },
                    Err(HandleError::NoSuchCommand) =>
                        kprintln!("unknown command: {}", command.path())
                }
            }
        }

        input_vec.truncate(0);
    }
}
