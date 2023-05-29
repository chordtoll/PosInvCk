use std::{
    os::linux::fs::MetadataExt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use asserteq_pretty_macros::PrettyDiff;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, PrettyDiff)]
pub struct FileAttr {
    pub ino: u64,
    pub size: u64,
    //pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub crtime: SystemTime,
    pub kind: FileType,
    pub perm: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub blksize: u32,
    pub flags: u32,
}

impl FileAttr {
    pub fn reset_times(&self) -> Self {
        let mut res = self.clone();
        res.atime = SystemTime::UNIX_EPOCH;
        res.mtime = SystemTime::UNIX_EPOCH;
        res.ctime = SystemTime::UNIX_EPOCH;
        res
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum FileType {
    NamedPipe,
    CharDevice,
    BlockDevice,
    Directory,
    RegularFile,
    Symlink,
    Socket,
}

impl asserteq_pretty::PrettyDiff for FileType {
    fn pretty_diff(left: &Self, right: &Self) -> String {
        format!("`{:?}` != `{:?}`", left, right)
    }
}

impl From<u32> for FileType {
    fn from(value: u32) -> Self {
        match value & 0o37777770000 {
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFREG => FileType::RegularFile,
            libc::S_IFLNK => FileType::Symlink,
            v => todo!("Unknown type {:o}", v),
        }
    }
}

impl From<fuser::FileType> for FileType {
    fn from(value: fuser::FileType) -> Self {
        match value {
            fuser::FileType::NamedPipe => FileType::NamedPipe,
            fuser::FileType::CharDevice => FileType::CharDevice,
            fuser::FileType::BlockDevice => FileType::BlockDevice,
            fuser::FileType::Directory => FileType::Directory,
            fuser::FileType::RegularFile => FileType::RegularFile,
            fuser::FileType::Symlink => FileType::Symlink,
            fuser::FileType::Socket => FileType::Socket,
        }
    }
}

impl FileAttr {
    pub fn set_ino(self, ino: u64) -> Self {
        Self { ino, ..self }
    }
}

impl From<std::fs::Metadata> for FileAttr {
    fn from(value: std::fs::Metadata) -> Self {
        Self::from(&value)
    }
}
impl From<&std::fs::Metadata> for FileAttr {
    fn from(value: &std::fs::Metadata) -> Self {
        let kind = FileType::from(value.st_mode());
        FileAttr {
            ino: value.st_ino(),
            size: if kind == FileType::Directory {
                0
            } else {
                value.st_size()
            },
            //blocks: value.st_blocks(),
            atime: UNIX_EPOCH
                + Duration::new(
                    value.st_atime().try_into().unwrap(),
                    value.st_atime_nsec().try_into().unwrap(),
                ),
            mtime: UNIX_EPOCH
                + Duration::new(
                    value.st_mtime().try_into().unwrap(),
                    value.st_mtime_nsec().try_into().unwrap(),
                ),
            ctime: UNIX_EPOCH
                + Duration::new(
                    value.st_ctime().try_into().unwrap(),
                    value.st_ctime_nsec().try_into().unwrap(),
                ),
            crtime: UNIX_EPOCH,
            kind,
            perm: (value.st_mode() & 0o7777).try_into().unwrap(),
            nlink: value.st_nlink().try_into().unwrap(),
            uid: value.st_uid(),
            gid: value.st_gid(),
            rdev: value.st_rdev().try_into().unwrap(),
            blksize: value.st_blksize().try_into().unwrap(),
            flags: 0,
        }
    }
}

impl From<fuser::FileAttr> for FileAttr {
    fn from(value: fuser::FileAttr) -> Self {
        Self::from(&value)
    }
}
impl From<&fuser::FileAttr> for FileAttr {
    fn from(value: &fuser::FileAttr) -> Self {
        FileAttr {
            ino: value.ino,
            size: if FileType::from(value.kind) == FileType::Directory {
                0
            } else {
                value.size
            },
            //blocks: value.blocks,
            atime: value.atime,
            mtime: value.mtime,
            ctime: value.ctime,
            crtime: value.crtime,
            kind: value.kind.into(),
            perm: value.perm,
            nlink: value.nlink,
            uid: value.uid,
            gid: value.gid,
            rdev: value.rdev,
            blksize: value.blksize,
            flags: value.flags,
        }
    }
}
