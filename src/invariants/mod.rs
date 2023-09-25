use std::{collections::BTreeMap, ffi::OsString};

use crate::{file_attr::FileAttr, inode_mapper::InodeMapper};

#[derive(Default)]
#[allow(non_snake_case)]
pub struct FSData {
    pub INODE_PATHS: InodeMapper,

    pub INV_INODE_PATHS: InodeMapper,

    #[cfg(feature = "check-meta")]
    pub INV_INODE_CONTENTS: BTreeMap<u64, FileAttr>,

    #[cfg(feature = "check-dirs")]
    pub INV_DIR_CONTENTS: BTreeMap<u64, BTreeMap<OsString, u64>>,

    #[cfg(feature = "check-data")]
    pub INV_FILE_CONTENTS: BTreeMap<u64, Vec<u8>>,

    #[cfg(feature = "check-xattr")]
    pub INV_XATTR_CONTENTS: BTreeMap<u64, BTreeMap<OsString, Vec<u8>>>,
}

impl FSData {
    pub fn new() -> Self {
        Self::default()
    }
}

pub mod common;
pub mod fs;
pub mod perm;
