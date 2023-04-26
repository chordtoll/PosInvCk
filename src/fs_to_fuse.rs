use std::{
    os::unix::prelude::{FileTypeExt, MetadataExt, PermissionsExt},
    time::{Duration, UNIX_EPOCH},
};

use fuser::{FileAttr, FileType};

pub trait FsToFuseAttr {
    fn to_fuse_attr(&self, ino: u64) -> FileAttr;
}

impl FsToFuseAttr for std::fs::Metadata {
    fn to_fuse_attr(&self, ino: u64) -> FileAttr {
        FileAttr {
            ino,
            size: self.size(),
            blocks: self.blocks(),
            atime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.atime().try_into().unwrap(),
                    self.atime_nsec().try_into().unwrap(),
                ))
                .unwrap(),
            mtime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.mtime().try_into().unwrap(),
                    self.mtime_nsec().try_into().unwrap(),
                ))
                .unwrap(),
            ctime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.ctime().try_into().unwrap(),
                    self.ctime_nsec().try_into().unwrap(),
                ))
                .unwrap(),
            crtime: UNIX_EPOCH,
            kind: self.file_type().to_fuse_kind(),
            perm: self.permissions().mode().try_into().unwrap(),
            nlink: self.nlink().try_into().unwrap(),
            uid: self.uid(),
            gid: self.gid(),
            rdev: self.rdev().try_into().unwrap(),
            blksize: self.blksize().try_into().unwrap(),
            flags: 0,
        }
    }
}

impl FsToFuseAttr for libc::stat {
    fn to_fuse_attr(&self, ino: u64) -> FileAttr {
        let perm = self.st_mode & 0o7777;
        let kind = match self.st_mode & libc::S_IFMT {
            libc::S_IFDIR => FileType::Directory,
            libc::S_IFREG => FileType::RegularFile,
            libc::S_IFBLK => FileType::BlockDevice,
            libc::S_IFCHR => FileType::CharDevice,
            libc::S_IFIFO => FileType::NamedPipe,
            libc::S_IFLNK => FileType::Symlink,
            v => todo!("Stat type: {:x}", v),
        };

        FileAttr {
            ino,
            size: self.st_size.try_into().unwrap(),
            blocks: self.st_blocks.try_into().unwrap(),
            atime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.st_atime.try_into().unwrap(),
                    self.st_atime_nsec.try_into().unwrap(),
                ))
                .unwrap(),
            mtime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.st_mtime.try_into().unwrap(),
                    self.st_mtime_nsec.try_into().unwrap(),
                ))
                .unwrap(),
            ctime: UNIX_EPOCH
                .checked_add(Duration::new(
                    self.st_ctime.try_into().unwrap(),
                    self.st_ctime_nsec.try_into().unwrap(),
                ))
                .unwrap(),
            crtime: UNIX_EPOCH,
            kind: kind,
            perm: perm.try_into().unwrap(),
            nlink: self.st_nlink.try_into().unwrap(),
            uid: self.st_uid,
            gid: self.st_gid,
            rdev: self.st_rdev.try_into().unwrap(),
            blksize: self.st_blksize.try_into().unwrap(),
            flags: 0,
        }
    }
}

pub trait FsToFuseKind {
    fn to_fuse_kind(&self) -> FileType;
}

impl FsToFuseKind for std::fs::FileType {
    fn to_fuse_kind(&self) -> FileType {
        if self.is_file() {
            return FileType::RegularFile;
        }
        if self.is_dir() {
            return FileType::Directory;
        }
        if self.is_symlink() {
            return FileType::Symlink;
        }
        if self.is_fifo() {
            return FileType::NamedPipe;
        }
        if self.is_block_device() {
            return FileType::BlockDevice;
        }
        if self.is_char_device() {
            return FileType::CharDevice;
        }
        if self.is_socket() {
            return FileType::Socket;
        }
        todo!("Filetype: {:?}", self)
    }
}
