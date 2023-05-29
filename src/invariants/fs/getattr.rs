use asserteq_pretty::{assert_eq_pretty, PrettyDiff};

use crate::{
    file_attr::{self, FileAttr},
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
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
    perm: bool,
    args: GetattrArgs,
}

pub fn inv_getattr_before(callid: CallID, req: &fuser::Request<'_>, ino: u64) -> GetattrInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &inode_path, Access::Lookup);

    GetattrInv {
        exists,
        perm,
        args: GetattrArgs { ino },
    }
}
pub fn inv_getattr_after(callid: CallID, inv: GetattrInv, res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(inv.exists, "Failed to return ENOENT on nonexistant inode");
            #[cfg(feature = "check-meta")]
            assert_eq_pretty!(
                crate::invariants::INODE_CONTENTS
                    .lock()
                    .unwrap()
                    .get(&inv.args.ino)
                    .map(|x| x.reset_times()),
                Some(FileAttr::from(v).reset_times()),
                "Result did not match expected value"
            );
        }
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
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
