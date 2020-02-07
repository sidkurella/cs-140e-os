use std::ffi::OsStr;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::borrow::Cow;
use std::io;
use std::slice;

use traits;
use util::VecExt;
use util::SliceExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};
use vfat::metadata;

#[derive(Debug)]
pub struct Dir {
    first_cluster: Cluster,
    vfat: Shared<VFat>,
    name: String,
    meta: Metadata
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
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

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    seq_number: u8,
    name_1: [u16; 5],
    attributes: Attributes,
    kind: u8,
    checksum: u8,
    name_2: [u16; 6],
    _zeros: u16,
    name_3: [u16; 2]
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    id: u8,
    _unknown_start: [u8; 10],
    attributes: Attributes,
    _unknown_end: [u8; 20]
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

pub struct DirIter {
    vfat: Shared<VFat>,
    data: Vec<u8>,
    off: usize
}

impl DirIter {
    fn read_entry(&mut self) -> Option<VFatUnknownDirEntry> {
        if self.off + 32 > self.data.len() {
            None
        } else {
            let dir = unsafe {
                self.data[self.off .. self.off + 32].cast()
            }[0];
            self.off = self.off + 32;
            Some(dir)
        }
    }
}

impl Iterator for DirIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut dir = self.read_entry()?;
        if dir.id == 0x00 {
            return None
        }

        let mut name: String = String::new();
        let mut buf = [0u16; 260]; // Max long filename.

        if dir.attributes.is_lfn() {
            // While we're seeing LFN entries.
            while dir.attributes.is_lfn() {
                let lfn_entry = unsafe {
                    VFatDirEntry {
                        unknown: dir
                    }.long_filename
                };
                let seq = lfn_entry.seq_number & 0xF;
                let base = seq as usize * 13;
                for (i, c) in lfn_entry.name_1.iter()
                                .chain(lfn_entry.name_2.iter())
                                .chain(lfn_entry.name_3.iter())
                                .take_while(|r| **r == 0x00 || **r == 0xFF)
                                .enumerate() {
                    buf[base + i] = *c;
                }

                dir = self.read_entry()?;
            }
            use std::iter::Extend;
            name.extend(
                decode_utf16(buf.iter().map(|r| *r)
                    .take_while(|r| *r == 0x00 || *r == 0xFF)
                ).map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
            );
        }

        let regular = unsafe {
            VFatDirEntry {
                unknown: dir
            }.regular
        };

        // At the last entry for this file.
        if name.len() <= 0 { // Use DOS name.
            let dos_name;
            match std::str::from_utf8(&regular.file_name) {
                Ok(c) => dos_name = c,
                Err(_) => return None
            }

            let dos_ext;
            match std::str::from_utf8(&regular.file_ext) {
                Ok(c) => dos_ext = c,
                Err(_) => return None
            }
            use std::fmt::Write;
            write!(&mut name, "{}.{}", dos_name, dos_ext);
        }

        let first_cluster = Cluster::from(
            ((regular.first_cluster_high as u32) << 16)
            | regular.first_cluster_low  as u32
        );

        if regular.attributes.is_dir() {
            Some(Entry::DirKind(Dir {
                first_cluster: first_cluster,
                meta: Metadata {
                    attributes: regular.attributes,
                    creation: regular.creation,
                    last_access_date: regular.last_access_date,
                    last_modified: regular.last_modified
                },
                name: name,
                vfat: self.vfat.clone()
            }))
        } else {
            Some(Entry::FileKind(File::new(
                first_cluster,
                Metadata {
                    attributes: regular.attributes,
                    creation: regular.creation,
                    last_access_date: regular.last_access_date,
                    last_modified: regular.last_modified
                },
                name,
                regular.file_size as usize,
                self.vfat.clone()
            )))
        }
    }
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use traits::Dir;
        let name_str = match name.as_ref().to_str() {
            Some(x) => x,
            None => return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "name is invalid utf8"
            ))
        };

        for entry in self.entries()? {
            use traits::Entry;
            if entry.name().eq_ignore_ascii_case(name_str) {
                return Ok(entry)
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "find target not found"
        ))
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn meta(&self) -> &Metadata {
        &self.meta
    }

    pub fn root(cluster: Cluster, vfat: Shared<VFat>) -> Dir {
        Dir {
            first_cluster: cluster,
            vfat: vfat,
            name: String::from(""),
            // Not sure if this is right but not sure what the metadata
            // is for the root dir.
            meta: Metadata {
                attributes: Attributes::new(metadata::DIRECTORY),
                creation: Timestamp::new(),
                last_access_date: Date::new(),
                last_modified: Timestamp::new()
            }
        }
    }
}

impl traits::Dir for Dir {
    type Entry = Entry;
    type Iter = DirIter;

    fn entries(&self) -> io::Result<Self::Iter> {
        let mut vec = Vec::new();
        self.vfat.borrow_mut().read_chain(self.first_cluster, &mut vec)?;
        Ok(DirIter {
            vfat: self.vfat.clone(),
            data: vec,
            off: 0
        })
    }
}
