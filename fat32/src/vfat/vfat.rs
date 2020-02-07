use std::io;
use std::path::{Path, Component};
use std::mem::{size_of, transmute};
use std::cmp::min;

use util::SliceExt;
use mbr::MasterBootRecord;
use vfat::{Shared, Cluster, File, Dir, Entry, FatEntry, Error, Status};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use traits::{FileSystem, BlockDevice};

#[derive(Debug)]
pub struct VFat {
    device: CachedDevice,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
        where T: BlockDevice + 'static
    {
        let mbr = MasterBootRecord::from(&mut device)?;
        for partition in &mbr.partitions {
            if partition.kind == 0xB || partition.kind == 0xC {
                let ebpb = BiosParameterBlock::from(&mut device, partition.lba_start.into())?;

                let logical = Partition {
                    start: partition.lba_start.into(),
                    sector_size: ebpb.bytes_per_sector.into()
                };
                let cache = CachedDevice::new(device, logical);

                return Ok(Shared::new(VFat {
                    device: cache,
                    bytes_per_sector: ebpb.bytes_per_sector,
                    sectors_per_cluster: ebpb.sectors_per_cluster,
                    sectors_per_fat: ebpb.sectors_per_fat,
                    fat_start_sector: ebpb.reserved_sectors as u64,
                    data_start_sector: ebpb.reserved_sectors as u64 + ebpb.sectors_per_fat as u64 * ebpb.fats as u64,
                    root_dir_cluster: Cluster::from(ebpb.root_dir_cluster)
                }));
            }
        }

        Err(Error::NotFound)
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    //    fn read_cluster(
    //        &mut self,
    //        cluster: Cluster,
    //        offset: usize,
    //        buf: &mut [u8]
    //    ) -> io::Result<usize>;
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    //    fn read_chain(
    //        &mut self,
    //        start: Cluster,
    //        buf: &mut Vec<u8>
    //    ) -> io::Result<usize>;
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;

    pub fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8]
    ) -> io::Result<usize> {
        match self.fat_entry(cluster)?.status() {
            Status::Data(_) | Status::Eoc(_) => {
                // Calculate starting sector by adding offset.
                let sector = cluster.into_sector(
                    self.data_start_sector, self.sectors_per_cluster
                ) + (offset as u64) / (self.bytes_per_sector as u64);

                // Calculate within-sector offset.
                let off = offset % (self.bytes_per_sector as usize);

                let sector_sz = self.device.sector_size() as usize;
                let cluster_sz = self.cluster_size();

                // Size for first, possibly smaller than sector read.
                let first_sz = min(sector_sz - off, buf.len());
                // Calculate total expected size to read.
                let total_sz = min(cluster_sz - off, buf.len());

                // Read in first part, from offset to end of sector.
                let first_read = self.device.read_offset(
                    sector as u64, off, &mut buf[..first_sz]
                )?;
                if first_read != first_sz {
                    // Short-read first sector.
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "failed to read expected number of bytes from sector"
                    ))
                }

                // Now read whole sectors.
                for (i, chunk) in buf[first_sz..total_sz]
                                  .chunks_mut(sector_sz).enumerate() {
                    let bytes_read = self.device.read_sector(
                        sector + i as u64, chunk
                    )?;

                    // Last chunk may be smaller.
                    let test_sz = min(sector_sz, chunk.len());
                    if bytes_read != test_sz {
                        // Short-read a sector.
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "failed to read expected number of bytes from sector"
                        ))
                    }
                }

                Ok(total_sz)
            },

            _ =>  Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "attempt to read from cluster with no data"
                 ))
        }
    }

    pub fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        let mut cluster = start; // The cluster to read.
        let mut sz: usize = 0; // The number of bytes read.

        let cluster_sz = self.cluster_size();

        // Loop while we go through the linked list of clusters.
        // Returned out of when the chain ends or on error.
        loop {
            let end = buf.len();
            buf.resize(end + cluster_sz, 0u8);
            sz += self.read_cluster(cluster, 0, &mut buf[end ..])?;
            match self.next_cluster(cluster)? {
                Some(c) => cluster = c,
                None => return Ok(sz)
            }
        }
    }

    /// Get the next cluster number from the current cluster number.
    pub fn next_cluster(&mut self, cluster: Cluster) -> io::Result<Option<Cluster>> {
        match self.fat_entry(cluster)?.status() {
            Status::Data(c) => Ok(Some(c)),
            Status::Eoc(_) => Ok(None),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData, "invalid entry in chain"
            ))
        }
    }

    /// Finds the cluster that is offset clusters from the start.
    pub fn find_cluster(&mut self, start: Cluster, offset: usize)
        -> io::Result<Option<Cluster>> {
        let mut cluster = start;
        for _ in 0..offset {
            match self.fat_entry(cluster)?.status() {
                Status::Data(c) => cluster = c,
                Status::Eoc(_) => return Ok(None),
                _ => return Err(io::Error::new(
                    io::ErrorKind::InvalidData, "invalid entry in chain"
                ))
            }
        }
        Ok(Some(cluster))
    }

    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let len = self.bytes_per_sector;
        let (sector, byte_off, entry_sz) = cluster.fat_entry_offset(
            self.fat_start_sector, self.bytes_per_sector
        );
        //println!("{:?} {} {} {}", self, sector, byte_off, entry_sz);
        let bytes = &self.device.get(sector)?[
            byte_off as usize .. byte_off as usize + entry_sz
        ];
        Ok(&(unsafe { SliceExt::cast(bytes) })[0])
    }

    pub fn cluster_size(&self) -> usize {
        let sector_sz = self.device.sector_size() as usize;
        sector_sz * self.sectors_per_cluster as usize
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = File;
    type Dir = Dir;
    type Entry = Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let mut entries: Vec<Self::Entry> = vec![
            Entry::DirKind(
                Dir::root(self.borrow().root_dir_cluster, self.clone())
            )
        ];
        for component in path.as_ref().components() {
            match component {
                Component::ParentDir => entries.truncate(entries.len() - 1),
                Component::Normal(s) => {
                    use traits::Entry;
                    let cur_dir = &(entries[entries.len() - 1]);
                    let entry = match cur_dir.as_dir() {
                        Some(dir) => dir.find(s)?,
                        None => return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "path spec is not a directory"
                        ))
                    };
                    entries.push(entry);
                }
                _ => () // Don't care about CurDir or Prefix or RootDir
            }
        }
        Ok(entries.remove(entries.len() - 1))
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
        where P: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
        where P: AsRef<Path>, Q: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
