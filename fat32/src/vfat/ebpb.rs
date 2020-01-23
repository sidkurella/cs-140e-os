use std::{fmt, mem};

use traits::BlockDevice;
use vfat::Error;

const BOOT_SIG : u16 = 0xAA55;
const EBPB_SIGS : [u8; 2] = [0x28, 0x29];

#[repr(C, packed)]
pub struct BiosParameterBlock {
    _jmp: [u8; 3],
    oem: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    _max_dir_entries: u16, // 0 for FAT32
    _logical_sectors: u16,
    media_descriptor_type: u8,
    _sectors_per_fat: u16, // 0 for FAT32
    sectors_per_track: u16,
    heads: u16,
    num_hidden_sectors: u32,
    logical_sectors: u32, // Used if greater than 65535.

    // Extended
    sectors_per_fat: u32,
    flags: u16,
    version_number: u16,
    root_dir_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    _reserved: [u8; 12],
    drive_number: u8,
    win_nt_flags: u8,
    signature: u8,
    volid_serial: u32,
    vol_label: [u8; 11],
    sys_id_string: [u8; 8],
    _boot: [u8; 420],
    _boot_sig: u16
}

impl BiosParameterBlock {
    fn valid_signature(&self) -> bool {
        self._boot_sig == BOOT_SIG
            && EBPB_SIGS.iter().any(|&x| x == self.signature)
    }
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        let mut bpb_buf : [u8; 512] = [0; 512];
        device.read_sector(sector, &mut bpb_buf)?;

        let mut bpb : BiosParameterBlock = unsafe { mem::transmute(bpb_buf) };
        if !bpb.valid_signature() {
            return Err(Error::BadSignature)
        }

        if bpb._logical_sectors != 0 { // Less than 65535.
            bpb.logical_sectors = bpb.logical_sectors;
        }

        Ok(bpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BiosPermissionBlock")
            .field("oem", &self.oem)
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("reserved_sectors", &self.reserved_sectors)
            .field("fats", &self.fats)
            .field("_max_dir_entries", &self._max_dir_entries)
            .field("_logical_sectors", &self._logical_sectors)
            .field("media_descriptor_type", &self.media_descriptor_type)
            .field("_sectors_per_fat", &self._sectors_per_fat)
            .field("sectors_per_track", &self.sectors_per_track)
            .field("heads", &self.heads)
            .field("num_hidden_sectors", &self.num_hidden_sectors)
            .field("logical_sectors", &self.logical_sectors)

            .field("sectors_per_fat", &self.sectors_per_fat)
            .field("flags", &self.flags)
            .field("version_number", &self.version_number)
            .field("root_dir_cluster", &self.root_dir_cluster)
            .field("fsinfo_sector", &self.fsinfo_sector)
            .field("backup_boot_sector", &self.backup_boot_sector)
            .field("_reserved", &self._reserved)
            .field("drive_number", &self.drive_number)
            .field("win_nt_flags", &self.win_nt_flags)
            .field("signature", &self.signature)
            .field("volid_serial", &self.volid_serial)
            .field("vol_label", &self.vol_label)
            .field("sys_id_string", &self.sys_id_string)
            .field("_boot_sig", &self._boot_sig)
            .finish()
    }
}
