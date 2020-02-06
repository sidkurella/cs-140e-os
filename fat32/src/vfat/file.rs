use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    first_cluster: Cluster,
    vfat: Shared<VFat>,
    name: String,
    size: usize,
    meta: Metadata
}

impl File {
    pub fn new(first_cluster: Cluster, meta: Metadata,
               name: String, size: usize, vfat: Shared<VFat>) -> File {
        File {
            first_cluster: first_cluster,
            meta: meta,
            name: name,
            size: size,
            vfat: vfat
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn meta(&self) -> &Metadata {
        &self.meta
    }
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl traits::File for File {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> { unimplemented!("lol") }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 { unimplemented!("lol") }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!("LOL")
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!("LOL")
    }
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("LOL")
    }
}

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}
