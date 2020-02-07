use std::{io, fmt, cmp};
use std::collections::HashMap;

use traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64
}

pub struct CachedDevice {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedDevice
        where T: BlockDevice + 'static
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        let factor = self.partition.sector_size / self.device.sector_size();
        let logical_offset = virt;
        let physical_offset = logical_offset * factor;
        let physical_sector = self.partition.start + physical_offset;
        (physical_sector, factor)
    }

    fn load_to_cache(&mut self, sector: u64) -> io::Result<()> {
        let (phys, len) = self.virtual_to_physical(sector);
        let sector_sz : usize = self.device.sector_size() as usize;
        let mut c = CacheEntry {
            data: vec![0; len as usize * sector_sz],
            dirty: false
        };

        for (i, chunk) in c.data.chunks_mut(sector_sz).enumerate() {
            let x = self.device.read_sector(phys + i as u64, chunk)?;
            if x != sector_sz {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to read whole sector")
                )
            }
        }
        let contains = self.cache.insert(sector, c);
        assert_eq!(contains.is_none(), true);
        Ok(())
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        if !self.cache.contains_key(&sector) {
            self.load_to_cache(sector)?;
        }

        let c = self.cache.get_mut(&sector).unwrap();
        c.dirty = true;
        Ok(&mut c.data)
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        if !self.cache.contains_key(&sector) {
            self.load_to_cache(sector)?;
        }

        let c = self.cache.get(&sector).unwrap();
        Ok(&c.data)
    }

    /// Read from a specified byte offset within a sector.
    pub fn read_offset(&mut self, n: u64, off: usize, buf: &mut [u8])
                       -> io::Result<usize> {
        let data = self.get(n)?;
        if off > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid read offset"
            ))
        }
        let sz = cmp::min(buf.len(), data.len() - off);
        buf[..sz].copy_from_slice(&data[off..off + sz]);

        Ok(sz)
    }
}

impl BlockDevice for CachedDevice {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        let data = self.get(n)?;
        let sz = cmp::min(buf.len(), data.len());
        buf[..sz].copy_from_slice(&data[..sz]);

        Ok(sz)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        let data = self.get_mut(n)?;
        let sz = cmp::min(buf.len(), data.len());
        data[..sz].copy_from_slice(&buf[..sz]);

        Ok(sz)
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
