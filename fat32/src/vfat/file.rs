use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
struct Position {
    cluster: Cluster,
    offset: usize
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
                offset: 0
            }
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
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("read only file system")
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size as u64
    }
}

fn offset_to_cluster(offset: usize, cluster_size: usize) -> (usize, usize) {
    (offset / cluster_size, offset % cluster_size)
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut vfat = self.vfat.borrow_mut();

        let cluster_size = vfat.cluster_size();

        // Figure out the maximum number of clusters to be read by consulting
        // the current position and the length of the buffer.
        let (start_cluster, _) = offset_to_cluster(self.pos.offset, cluster_size);
        let end_cluster = match offset_to_cluster(self.pos.offset + buf.len(), cluster_size) {
            (c, 0) => c - 1,
            (c, _) => c
        };

        let mut bytes_read: usize = 0;
        for _ in (start_cluster..=end_cluster) {
            // Find the current offset into the cluster for partial first read.
            let (_, current_offset) = offset_to_cluster(self.pos.offset, cluster_size);

            // Compute how many bytes are available in the file and cluster.
            let bytes_available = min(cluster_size - current_offset, self.size - self.pos.offset);

            let bytes_remaining = buf.len() - bytes_read;
            let bytes_to_read = min(bytes_remaining, bytes_available);
            if bytes_to_read > 0 {
                let bytes_added = vfat.read_cluster(
                    self.pos.cluster,
                    current_offset,
                    &mut buf[bytes_read..bytes_read + bytes_to_read]
                )?;

                if bytes_added != bytes_to_read {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to read all bytes of request"));
                }

                bytes_read += bytes_added;
                self.pos.offset += bytes_added;
            }

            // If the number of bytes read is the number availabe, then this
            // cluster must then be exhausted.
            if bytes_to_read == bytes_available {
                self.pos.cluster = match vfat.next_cluster(self.pos.cluster)? {
                    Some(c) => c,
                    None => break
                };
            }
        }

        // Finally, we return the total number of bytes read.
        Ok(bytes_read)
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
                if ((self.size - self.pos.offset) as i64) < off
                   || (self.pos.offset) < (-off as usize) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid seek"
                    ))
                } else {
                    (self.pos.offset as i64 + off) as usize
                },
        };

        let cluster = {
            let mut vfat = self.vfat.borrow_mut();
            let cluster_size = vfat.cluster_size();
            let (cluster_idx, _) = offset_to_cluster(bytes, cluster_size);

            match self.vfat.borrow_mut().find_cluster(self.first_cluster, cluster_idx)? {
                Some(c) => c,
                // This cluster should exist since we checked file size.
                None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))
            }
        };

        self.pos = Position {
            cluster: cluster,
            offset: bytes
        };

        Ok(bytes as u64)
    }
}
