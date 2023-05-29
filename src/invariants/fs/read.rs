use std::cmp::min;

use crate::{
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct ReadInv {
    exists: bool,
    perm: bool,
    ino: u64,
    offset: usize,
    size: usize,
}

pub fn inv_read_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    ino: u64,
    _fh: u64,
    offset: i64,
    size: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
) -> ReadInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &inode_path, Access::Lookup);

    ReadInv {
        ino,
        offset: offset.try_into().unwrap(),
        size: size.try_into().unwrap(),
        exists,
        perm,
    }
}
pub fn inv_read_after(callid: CallID, inv: ReadInv, res: &Result<Vec<u8>, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(inv.exists, "Failed to return ENOENT on nonexistant child");
            #[cfg(feature = "check-data")]
            {
                let fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
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
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.exists, "Returned ENOENT on extant path"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
