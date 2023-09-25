use std::{cmp::min, path::Path, sync::MutexGuard};

use crate::{
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct ReadInv {
    exists: bool,
    perm: Option<i32>,
    ino: u64,
    offset: usize,
    size: usize,
}

pub fn inv_read_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    ino: u64,
    _fh: u64,
    offset: i64,
    size: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> ReadInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino, fs_data);

    let perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &inode_path,
        base,
        Access::Lookup,
    );

    ReadInv {
        ino,
        offset: offset.try_into().unwrap(),
        size: size.try_into().unwrap(),
        exists,
        perm,
    }
}
pub fn inv_read_after(
    callid: CallID,
    inv: ReadInv,
    res: &Result<Vec<u8>, i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(inv.exists, "Failed to return ENOENT on nonexistant child");
            #[cfg(feature = "check-data")]
            {
                let fc = &fs_data.INV_FILE_CONTENTS;
                let exp_content = fc
                    .get(&inv.ino)
                    .unwrap_or_else(|| panic!("Unknown file {}", inv.ino));
                let start = inv.offset;
                let end = inv.offset + inv.size;
                let end = min(end, exp_content.len());
                let start = min(start, end);
                assert_eq!(&exp_content[start..end], v, "File contents differ")
            }
        }
        Err(libc::EACCES) => assert_eq!(
            inv.perm,
            Some(libc::EACCES),
            "Returned EACCES on path where we have permission"
        ),
        Err(libc::EPERM) => assert_eq!(
            inv.perm,
            Some(libc::EPERM),
            "Returned EPERM on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.exists, "Returned ENOENT on extant path"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
