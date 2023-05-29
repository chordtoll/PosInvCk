use std::{collections::BTreeMap, ffi::OsString, sync::Mutex};

use lazy_static::lazy_static;

use crate::{file_attr::FileAttr, inode_mapper::InodeMapper};

lazy_static! {
    pub static ref INODE_PATHS: Mutex<InodeMapper> = Mutex::new(InodeMapper::new());
}
#[cfg(feature = "check-meta")]
lazy_static! {
    pub static ref INODE_CONTENTS: Mutex<BTreeMap<u64, FileAttr>> = Mutex::new(BTreeMap::new());
}
#[cfg(feature = "check-dirs")]
lazy_static! {
    pub static ref DIR_CONTENTS: Mutex<BTreeMap<u64, BTreeMap<OsString, u64>>> =
        Mutex::new(BTreeMap::new());
}
#[cfg(feature = "check-data")]
lazy_static! {
    pub static ref FILE_CONTENTS: Mutex<BTreeMap<u64, Vec<u8>>> = Mutex::new(BTreeMap::new());
}
#[cfg(feature = "check-xattr")]
lazy_static! {
    pub static ref XATTR_CONTENTS: Mutex<BTreeMap<u64, BTreeMap<OsString, Vec<u8>>>> =
        Mutex::new(BTreeMap::new());
}

pub mod common;
pub mod fs;
pub mod perm;
