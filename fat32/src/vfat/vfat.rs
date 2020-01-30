use std::io;
use std::path::Path;
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

    fn read_cluster(
        &mut self,
        cluster: Cluster,
        buf: &mut [u8]
    ) -> io::Result<usize> {
        match self.fat_entry(cluster)?.status() {
            Status::Data(_) | Status::Eoc(_) => {
                let sector = cluster.into_sector(
                    self.data_start_sector, self.sectors_per_cluster
                );

                let sector_sz = self.device.sector_size() as usize;
                let cluster_sz = sector_sz * self.sectors_per_cluster as usize;
                let sz = min(cluster_sz, buf.len());

                for (i, chunk) in buf[..sz].chunks_mut(sector_sz).enumerate() {
                    let bytes_read = self.device.read_sector(
                        sector + i as u64, chunk
                    )?;

                    // Last chunk may be smaller.
                    let test_sz = min(sector_sz, chunk.len());
                    if bytes_read != test_sz {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "failed to read whole sector"
                        ))
                    }
                }

                Ok(sz)
            },

            _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "attempt to read from cluster with no data"
                 ))
        }
    }

    fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        let mut cluster = start; // The cluster to read.
        let mut sz: usize = 0; // The number of bytes read.

        let sector_sz = self.device.sector_size() as usize;
        let cluster_sz = sector_sz * self.sectors_per_cluster as usize;

        // Loop while we go through the linked list of clusters.
        // Returned out of when the chain ends or on error.
        loop {
            let end = buf.len();
            buf.resize(end + cluster_sz, 0u8);
            sz += self.read_cluster(cluster, &mut buf[end ..])?;
            match self.fat_entry(start)?.status() {
                Status::Data(c) => cluster = c,
                Status::Eoc(_) => return Ok(sz),
                _ => return Err(io::Error::new(
                    io::ErrorKind::InvalidData, "invalid entry in chain"
                ))
            }
        }
    }


    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let len = self.bytes_per_sector;
        let (sector, byte_off, entry_sz) = cluster.fat_entry_offset(
            self.fat_start_sector, self.bytes_per_sector
        );
        let bytes = &self.device.get(sector)?[
            byte_off as usize .. byte_off as usize + entry_sz
        ];
        Ok(&(unsafe { SliceExt::cast(bytes) })[0])
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = ::traits::Dummy;
    type Dir = ::traits::Dummy;
    type Entry = ::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
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
