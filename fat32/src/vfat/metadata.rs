use std::fmt;
use std::str;

use traits;

const BASE_YEAR: usize = 1980;
const SECONDS_MULTIPLIER: u8 = 2;

const READ_ONLY: u8 = 0x1;
const HIDDEN: u8 = 0x2;
const SYSTEM: u8 = 0x4;
const VOLUME_ID: u8 = 0x8;
const DIRECTORY: u8 = 0x10;
const ARCHIVE: u8 = 0x20;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[repr(C, packed)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub time: Time,
    pub date: Date
}

/// Metadata for a directory entry.
#[repr(C, packed)]
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    file_name: [u8; 8],
    file_ext: [u8; 3],
    attributes: Attributes,
    _reserved: u8,
    creation_tenths: u8,
    creation: Timestamp,
    last_access_date: Date,
    first_cluster_high: u16,
    last_modified: Timestamp,
    first_cluster_low: u16,
    file_size: u32
}

impl Date {
    fn month_str(&self) -> &'static str {
        use traits::Date;
        match self.month() {
             1 => "January",
             2 => "February",
             3 => "March",
             4 => "April",
             5 => "May",
             6 => "June",
             7 => "July",
             8 => "August",
             9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
             _ => "<invalid>"
        }
    }
}

impl traits::Date for Date {
    fn year(&self) -> usize {
        // Bits 9-15 are year, starting from 1980.
        (self.0 >> 8) as usize + BASE_YEAR
    }

    fn month(&self) -> u8 {
        // Bits 5-8 are month.
        (self.0 >> 4) as u8 & 0xF
    }

    fn day(&self) -> u8 {
        // Low 4 bits are day.
        (self.0 & 0xF) as u8
    }
}

impl traits::Time for Time {
    fn hour(&self) -> u8 {
        // Bits 11-15 are hours.
        (self.0 >> 10) as u8
    }

    fn minute(&self) -> u8 {
        // Bits 5-10 are minutes.
        (self.0 >> 4) as u8 & 0x3F
    }

    fn second(&self) -> u8 {
        // Low 4 bits are seconds/2.
        (self.0 & 0xF) as u8 * SECONDS_MULTIPLIER
    }
}

impl traits::Date for Timestamp {
    fn year(&self) -> usize {
        self.date.year()
    }

    fn month(&self) -> u8 {
        self.date.month()
    }

    fn day(&self) -> u8 {
        self.date.day()
    }
}

impl traits::Time for Timestamp {
    fn hour(&self) -> u8 {
        self.time.hour()
    }

    fn minute(&self) -> u8 {
        self.time.minute()
    }

    fn second(&self) -> u8 {
        self.time.second()
    }
}

impl traits::Timestamp for Timestamp {
    // Nothing to do.
}

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        (self.attributes.0 & READ_ONLY) != 0
    }

    fn hidden(&self) -> bool {
        (self.attributes.0 & HIDDEN) != 0
    }

    fn created(&self) -> Self::Timestamp {
        self.creation
    }

    fn accessed(&self) -> Self::Timestamp {
        Timestamp {
            time: Time(0),
            date: self.last_access_date
        }
    }

    fn modified(&self) -> Self::Timestamp {
        self.last_modified
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use traits::Date;
        write!(f, "{} {}, {}", self.month_str(), self.day(), self.year())
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use traits::Time;
        write!(f, "{}:{}:{}", self.hour(), self.minute(), self.second())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -- {}", self.date, self.time)
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use traits::Metadata;
        if self.read_only() {
            write!(f, "RO ")?;
        }
        if self.hidden() {
            write!(f, "Hidden ")?;
        }

        let name;
        match str::from_utf8(&self.file_name) {
            // Because ? can't auto-convert this error.
            Ok(c) => name = c,
            Err(_) => return Err(fmt::Error)
        }

        let ext;
        match str::from_utf8(&self.file_ext) {
            // Because ? can't auto-convert this error.
            Ok(c) => ext = c,
            Err(_) => return Err(fmt::Error)
        }

        write!(
            f, "{}.{}\nSize: {}\nCreated {}.{}\nAccessed {}\nModified {}",
            name, ext,
            self.file_size,
            self.created(), self.creation_tenths,
            self.last_access_date, self.modified()
        )
    }
}
