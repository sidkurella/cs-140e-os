use vfat::*;
use std::mem::size_of;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }

}

// TODO: Implement any useful helper methods on `Cluster`.
impl Cluster {
    pub fn into_sector(&self, data_start_sector: u64, sectors_per_cluster: u8) -> u64 {
        self.0 as u64 * sectors_per_cluster as u64 + data_start_sector
    }

    pub fn fat_entry_offset(
        &self, fat_start_sector: u64, bytes_per_sector: u16
    ) -> (u64, u64, usize) {
        let sector_off = self.0 as u64 * size_of::<u32>() as u64
                         / bytes_per_sector as u64;
        let byte_off = self.0 as u64 * size_of::<u32>() as u64
                       % bytes_per_sector as u64;
        (sector_off + fat_start_sector, byte_off, size_of::<u32>())
    }
}
