use std::{ffi::OsStr, os::linux::fs::MetadataExt, path::PathBuf};

use crate::{invariants::INODE_PATHS, log_more, logging::CallID};

pub struct CPPN {
    pub child_path: PathBuf,
    pub ino: Option<u64>,
    pub parent_exists: bool,
    pub child_exists: bool,
    pub toolong: bool,
}

pub fn common_pre_parent_name(parent: u64, name: &OsStr) -> CPPN {
    let ip = INODE_PATHS.lock().unwrap();
    let parent_paths = ip
        .get_all(parent)
        .expect("Called lookup on unknown parent inode");
    let parent_path = parent_paths.first().expect("Parent has no paths");
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

pub fn common_pre_ino(callid: CallID, ino: u64) -> CPI {
    let ip = INODE_PATHS.lock().unwrap();
    let inode_paths = ip.get_all(ino).expect("Called lookup on unknown inode");
    log_more!(callid, "Inode {} has paths {:?}", ino, inode_paths);
    let inode_path = inode_paths.first().expect("Inode has no paths").clone();
    let exists = inode_path.symlink_metadata().is_ok();

    CPI { inode_path, exists }
}
