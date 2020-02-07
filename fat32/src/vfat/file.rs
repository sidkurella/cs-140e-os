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
        let mut bytes_read: usize = 0;
        let cluster_sz = self.vfat.borrow().cluster_size();

        // Calculate size for first read (maybe smaller than whole cluster).
        let first_sz = min(cluster_sz - self.pos.byte_offset, buf.len());
        // Calculate size for whole read.
        let total_sz = min(self.size - self.pos.total_offset, buf.len());

        if total_sz == 0 {
            return Ok(0)
        }

        // Read first part, which may be less than a cluster.
        let first_read = self.vfat.borrow_mut().read_cluster(
            self.pos.cluster,
            self.pos.byte_offset,
            &mut buf[..first_sz]
        )?;
        if first_read != first_sz {
            // Short-read first sector.
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "failed to read expected number of bytes from cluster"
            ))
        }

        let mut bytes_read: usize = first_sz;

        // Update position after first read.
        let cluster = match self.vfat.borrow_mut().next_cluster(self.pos.cluster)? {
            Some(c) => c,
            None => return Ok(bytes_read) // Should this be an error? Size is checked...
        };
        self.pos = Position {
            cluster: cluster,
            cluster_offset: self.pos.cluster_offset + 1,
            byte_offset: 0,
            total_offset: self.pos.total_offset + first_sz
        };


        // Read whole clusters.
        for chunk in buf[first_sz..total_sz].chunks_mut(cluster_sz) {
            let cluster_read = self.vfat.borrow_mut().read_cluster(
                self.pos.cluster,
                self.pos.byte_offset, // Always 0.
                chunk
            )?;

            let test_sz = min(cluster_sz, chunk.len());
            if cluster_read != test_sz {
                // Short-read a sector.
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to read expected number of bytes from cluster"
                ))
            }
            bytes_read = bytes_read + cluster_read;

            // Update position information.
            let next_cluster = match self.vfat.borrow_mut()
                                     .next_cluster(self.pos.cluster)? {
                Some(c) => c,
                None => return Ok(bytes_read) // Should this be an error? Size is checked...
            };
            self.pos = Position {
                cluster: next_cluster,
                cluster_offset: self.pos.cluster_offset + 1,
                byte_offset: 0,
                total_offset: self.pos.total_offset + chunk.len()
            };
        }

        Ok(total_sz)
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
