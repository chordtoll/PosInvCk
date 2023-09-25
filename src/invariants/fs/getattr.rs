use std::{path::Path, sync::MutexGuard};

use asserteq_pretty::{assert_eq_pretty, PrettyDiff};

use crate::{
    file_attr::{self, FileAttr},
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
pub struct GetattrArgs {
    ino: u64,
}

#[derive(Debug)]
#[must_use]
pub struct GetattrInv {
    exists: bool,
    perm: Option<i32>,
    args: GetattrArgs,
}

pub fn inv_getattr_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    ino: u64,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> GetattrInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino, fs_data);

    let perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &inode_path,
        base,
        Access::Lookup,
    );

    GetattrInv {
        exists,
        perm,
        args: GetattrArgs { ino },
    }
}
pub fn inv_getattr_after(
    callid: CallID,
    inv: GetattrInv,
    res: &Result<fuser::FileAttr, i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(inv.exists, "Failed to return ENOENT on nonexistant inode");
            #[cfg(feature = "check-meta")]
            assert_eq_pretty!(
                fs_data
                    .INV_INODE_CONTENTS
                    .get(&inv.args.ino)
                    .map(|x| x.reset_times()),
                Some(FileAttr::from(v).reset_times()),
                "Result did not match expected value"
            );
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

// TODO: we need a blanket &T implementation in asserteq_pretty
impl PrettyDiff for &file_attr::FileAttr {
    fn pretty_diff(left: &Self, right: &Self) -> String {
        PrettyDiff::pretty_diff(*left, *right)
    }
}
