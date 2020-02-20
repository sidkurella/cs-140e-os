pub mod sd;

use std::io;
use std::ops::DerefMut;
use std::path::Path;

use fat32::vfat::{self, Shared, VFat};
pub use fat32::traits;

use crate::mutex::Mutex;
use self::sd::Sd;

pub struct FileSystem(Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        let option = &mut self.0.lock();
        if option.is_none() {
            let sd = Sd::new().expect("SD failed to initialize");
            let vfat = VFat::from(sd).expect("VFat failed to initialize");
            option.replace(Shared::from(vfat));
        }
    }

    pub fn with_borrow<R, F: FnOnce(&Shared<VFat>) -> R>(&self, func: F) -> R {
        func(self.0.lock().as_ref().expect("Filesystem not initialized"))
    }
}

impl <'a> traits::FileSystem for &'a FileSystem where {
    type File = <&'a Shared<VFat> as traits::FileSystem>::File;
    type Dir = <&'a Shared<VFat> as traits::FileSystem>::Dir;
    type Entry = <&'a Shared<VFat> as traits::FileSystem>::Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        self.with_borrow(|vfat| vfat.open(path))
    }
    fn create_file<P: AsRef<Path>>(self, path: P) -> io::Result<Self::File> {
        self.with_borrow(|vfat| vfat.create_file(path))
    }
    fn create_dir<P: AsRef<Path>>(self, path: P, children: bool) -> io::Result<Self::Dir> {
        self.with_borrow(|vfat| vfat.create_dir(path, children))
    }
    fn rename<P: AsRef<Path>, Q: AsRef<Path>>(self, from: P, to: Q) -> io::Result<()> {
        self.with_borrow(|vfat| vfat.rename(from, to))
    }
    fn remove<P: AsRef<Path>>(self, path: P, children: bool) -> io::Result<()> {
        self.with_borrow(|vfat| vfat.remove(path, children))
    }
}
