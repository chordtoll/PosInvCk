use std::{path::Path, sync::MutexGuard};

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
pub struct SetxattrInv {}

pub fn inv_setxattr_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    ino: u64,
    _name: &std::ffi::OsStr,
    _value: &[u8],
    _flags: i32,
    _position: u32,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> SetxattrInv {
    let CPI { inode_path, .. } = common_pre_ino(callid, ino, fs_data);

    let _perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &inode_path,
        base,
        Access::Lookup,
    );

    SetxattrInv {}
}
pub fn inv_setxattr_after(callid: CallID, inv: SetxattrInv, _res: &Result<(), i32>) {
    log_more!(callid, "invariant={:?}", inv);

    #[cfg(feature = "check-xattr")]
    {
        compile_error!("XATTR validation is not yet implemented")
    }
}
