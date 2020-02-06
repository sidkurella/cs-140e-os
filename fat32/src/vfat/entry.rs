use traits;
use vfat::{File, Dir, Metadata, Cluster};

// TODO: You may need to change this definition.
#[derive(Debug)]
pub enum Entry {
    FileKind(File),
    DirKind(Dir)
}

// TODO: Implement any useful helper methods on `Entry`.

impl traits::Entry for Entry {
    type File = File;
    type Dir = Dir;
    type Metadata = Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str {
        match self {
            Entry::FileKind(file) => file.name().as_str(),
            Entry::DirKind(dir) => dir.name().as_str()
        }
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::FileKind(file) => file.meta(),
            Entry::DirKind(dir) => dir.meta()
        }
    }

    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&Self::File> {
        match self {
            Entry::FileKind(file) => Some(&file),
            Entry::DirKind(_) => None
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            Entry::FileKind(_) => None,
            Entry::DirKind(dir) => Some(&dir)
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::FileKind(file) => Some(file),
            Entry::DirKind(_) => None
        }
    }

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::FileKind(_) => None,
            Entry::DirKind(dir) => Some(dir)
        }
    }
}
