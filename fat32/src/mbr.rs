use std::{fmt, io, mem};

use traits::BlockDevice;

const MBR_SIGNATURE : u16 = 0xAA55;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CHS {
    head: u8,
    sector: u8,
    cylinder: u16
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct InnerCHS {
    head: u8,
    cylinder_sector: u16
}

impl<'a> From<&'a InnerCHS> for CHS {
    fn from(ichs: &'a InnerCHS) -> CHS {
        CHS {
            head: ichs.head,
            sector: (ichs.cylinder_sector & 0x3f) as u8,
            cylinder: ichs.cylinder_sector >> 6
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    indicator: u8,
    _chs_start: InnerCHS,
    kind: u8,
    _chs_end: InnerCHS,
    lba_start: u32,
    lba_sectors: u32
}

impl PartitionEntry {
    fn valid_indicator(&self) -> bool {
        (self.indicator & 0x7f) == 0
    }
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    _boot: [u8; 446],
    partitions: [PartitionEntry; 4],
    signature: u16
}

impl MasterBootRecord {
    fn valid_signature(&self) -> bool {
        self.signature == MBR_SIGNATURE
    }
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl From<io::Error> for Error {
    fn from(from: io::Error) -> Error {
        Error::Io(from)
    }
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut mbr_buf : [u8; 512] = [0; 512];
        device.read_sector(0, &mut mbr_buf)?;

        let mbr : MasterBootRecord = unsafe { mem::transmute(mbr_buf) };
        if !mbr.valid_signature() {
            return Err(Error::BadSignature)
        }

        for i in 0..4 {
            if !mbr.partitions[i].valid_indicator() {
                return Err(Error::UnknownBootIndicator(i as u8))
            }
        }

        Ok(mbr)
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MasterBootRecord")
           .field("partitions", &self.partitions)
           .field("signature", &self.signature)
           .finish()
    }
}
