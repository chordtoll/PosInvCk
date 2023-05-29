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
pub struct SetxattrInv {}

pub fn inv_setxattr_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    ino: u64,
    _name: &std::ffi::OsStr,
    _value: &[u8],
    _flags: i32,
    _position: u32,
) -> SetxattrInv {
    let CPI { inode_path, .. } = common_pre_ino(callid, ino);

    let _perm = check_perm(req.uid(), req.gid(), req.pid(), &inode_path, Access::Lookup);

    SetxattrInv {}
}
pub fn inv_setxattr_after(callid: CallID, inv: SetxattrInv, _res: &Result<(), i32>) {
    log_more!(callid, "invariant={:?}", inv);

    #[cfg(feature = "check-xattr")]
    {
        compile_error!("XATTR validation is not yet implemented")
    }
}
