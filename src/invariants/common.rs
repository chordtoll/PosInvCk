use std::{ffi::OsStr, os::linux::fs::MetadataExt, path::PathBuf, sync::MutexGuard};

use crate::{log_more, logging::CallID};

use super::FSData;

pub struct CPPN {
    pub child_path: PathBuf,
    pub ino: Option<u64>,
    pub parent_exists: bool,
    pub child_exists: bool,
    pub toolong: bool,
}

pub fn common_pre_parent_name(
    parent: u64,
    name: &OsStr,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> CPPN {
    let ip = &fs_data.INV_INODE_PATHS;
    let parent_paths = ip
        .get_all(parent)
        .unwrap_or_else(|| panic!("Called lookup on unknown parent inode: {}", parent));
    let parent_path = parent_paths.iter().next().expect("Parent has no paths");
    assert!(
        parent_path.exists(),
        "Parent {:?} does not exist",
        parent_path
    );
    let ino = parent_path
        .symlink_metadata()
        .expect("Failed to get parent metadata")
        .st_ino();
    for i in parent_paths {
        assert!(i.exists(), "Parent {:?} does not exist", i);
        assert_eq!(
            ino,
            i.symlink_metadata()
                .expect("Failed to get parent metadata")
                .st_ino(),
            "Mismatched inode number between {:?} and {:?}",
            parent_path,
            i
        );
    }
    let parent_exists = parent_path.symlink_metadata().is_ok();
    let child_path = parent_path.join(name);
    let child_exists = child_path.symlink_metadata().is_ok();
    let toolong = name.len() > 255;

    let ino = child_path.symlink_metadata().ok().map(|x| x.st_ino());

    CPPN {
        child_path,
        ino,
        parent_exists,
        child_exists,
        toolong,
    }
}

pub struct CPI {
    pub inode_path: PathBuf,
    pub exists: bool,
}

pub fn common_pre_ino(callid: CallID, ino: u64, fs_data: &mut MutexGuard<'_, FSData>) -> CPI {
    let ip = &fs_data.INV_INODE_PATHS;
    let inode_paths = ip.get_all(ino).expect("Called lookup on unknown inode");
    log_more!(callid, "Inode {} has paths {:?}", ino, inode_paths);
    let inode_path = inode_paths
        .iter()
        .next()
        .expect("Inode has no paths")
        .clone();
    let exists = inode_path.symlink_metadata().is_ok();

    CPI { inode_path, exists }
}
