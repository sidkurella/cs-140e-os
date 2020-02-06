use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
struct Position {
    cluster: Cluster,
    cluster_offset: usize,
    byte_offset: usize,
    total_offset: usize
}

#[derive(Debug)]
pub struct File {
    first_cluster: Cluster,
    vfat: Shared<VFat>,
    name: String,
    size: usize,
    meta: Metadata,
    pos: Position
}

impl File {
    pub fn new(first_cluster: Cluster, meta: Metadata,
               name: String, size: usize, vfat: Shared<VFat>) -> File {
        File {
            first_cluster: first_cluster,
            meta: meta,
            name: name,
            size: size,
            vfat: vfat,
            pos: Position {
                cluster: first_cluster,
                cluster_offset: 0,
                byte_offset: 0,
                total_offset: 0
            }
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn meta(&self) -> &Metadata {
        &self.meta
    }

    /// Calculates the cluster and start byte from a position from the start.
    /// Cluster number indexed where 0 is the first cluster.
    fn offset_cluster(&self, pos: usize) -> (usize, usize) {
        let cluster_sz = self.vfat.borrow().cluster_size();
        let cluster_no = pos / cluster_sz;
        (cluster_no, pos - cluster_no * cluster_sz)
    }
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl traits::File for File {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("read only file system")
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size as u64
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!("LOL")
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!("read only file system")
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("read only file system")
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
        let bytes = match pos {
            SeekFrom::Start(off) => off as usize,
            SeekFrom::End(off) =>
                if off > 0 || (-off as usize) > self.size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid seek"
                    ))
                } else {
                    self.size - (-off as usize)
                },
            SeekFrom::Current(off) =>
                if ((self.size - self.pos.total_offset) as i64) < off
                   || (self.pos.total_offset) < (-off as usize) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid seek"
                    ))
                } else {
                    (self.pos.total_offset as i64 + off) as usize
                },
        };
        let (cluster_offset, byte_offset) = self.offset_cluster(bytes);
        let cluster = match self.vfat.borrow_mut()
            .find_cluster(self.first_cluster, cluster_offset)? {
            Some(c) => c,
            None => return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek"
            )) // This cluster should exist since we checked file size.
        };
        self.pos = Position {
            cluster: cluster,
            cluster_offset: cluster_offset,
            byte_offset: byte_offset,
            total_offset: bytes
        };
        Ok(bytes as u64)
    }
}
