use std::cell::Cell;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::str;

use crate::console::{kprint, kprintln, CONSOLE};
use crate::FILE_SYSTEM;
use stack_vec::StackVec;
use fat32::traits::{FileSystem, Dir, Entry, File};
use fat32::vfat;

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
    NoSuchCommand,
    Terminate
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

    fn handle(&self, working: &mut Cell<PathBuf>) -> Result<(), HandleError> {
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
        } else if self.path() == "pwd" {
            kprintln!("{}", working.get_mut().display());
            Ok(())
        } else if self.path() == "cd" {
            if self.args.len() == 2 {
                let mut new = working.get_mut().clone();
                new.push(self.args[1]);

                match FILE_SYSTEM.open_dir(&new) {
                    Ok(_) => working.set(new),
                    Err(error) => kprintln!("cd: {}", error)
                }
            } else {
                kprintln!("cd: improper number of arguments");
            }
            Ok(())
        } else if self.path() == "ls" {
            let mut all = false;
            let mut dir = ".";

            if self.args.len() == 2 {
                dir = &self.args[1];
            } else if self.args.len() == 3 && self.args[1] == "-a" {
                all = true;
                dir = &self.args[2];
            } else if self.args.len() != 1 {
                kprintln!("ls: improper number of arguments");
                return Ok(())
            }

            let mut new = working.get_mut().clone();
            new.push(dir);

            match FILE_SYSTEM.open_dir(new).and_then(|dir| dir.entries()) {
                Ok(iter) => for e in iter { kprintln!("{}", e.name()); },
                Err(error) => kprintln!("ls: {}", error)
            }

            Ok(())
        } else if self.path() == "cat" {
            if self.args.len() >= 2 {
                for name in self.args[1..].iter() {
                    let mut new = working.get_mut().clone();
                    new.push(name);

                    match FILE_SYSTEM.open_file(new).and_then(|mut file| {
                                let mut buf = vec![0; file.size() as usize];
                                file.read(&mut buf).map(|_| buf)
                            }) {
                        Ok(buf) =>
                            match str::from_utf8(&buf[..]) {
                                Ok(contents) => kprint!("{}", contents),
                                Err(error) => kprintln!("ls: {}", error)
                            },
                        Err(error) => kprintln!("ls: {}", error)
                    }
                }
            } else {
                kprintln!("cat: improper number of arguments");
            }

            Ok(())
        } else if self.path() == "exit" {
            Err(HandleError::Terminate)
        } else {
            Err(HandleError::NoSuchCommand)
        }
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) {
    let mut input_buf = [0; MAX_CMDLEN];
    let mut input_vec = StackVec::new(&mut input_buf);

    let mut working = Cell::new(PathBuf::from("/"));

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
            let input_str = str::from_utf8(input_vec.as_slice())
                                    .expect("failed to decode utf8");
            let mut input_args = [input_str; MAX_ARGLEN];

            match Command::parse(&input_str, &mut input_args[..]) {
                Err(Error::Empty) => {},
                Err(Error::TooManyArgs) => kprintln!("too many arguments"),
                Ok(command) => match command.handle(&mut working) {
                    Ok(_) => { },
                    Err(HandleError::Terminate) =>
                        break,
                    Err(HandleError::NoSuchCommand) =>
                        kprintln!("unknown command: {}", command.path())
                }
            }
        }

        input_vec.truncate(0);
    }
}

#[no_mangle]
pub extern "C" fn run_shell() {
    kprintln!("starting user shell");
    shell("user>");
    unsafe { asm!("brk 1" :::: "volatile"); }
    loop { shell("user2>"); }
}