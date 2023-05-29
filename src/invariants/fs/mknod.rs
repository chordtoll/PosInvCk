use crate::{
    invariants::{
        common::{common_pre_parent_name, CPPN},
        perm::{check_perm, Access},
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct MknodInv {}

pub fn inv_mknod_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
    _mode: u32,
    _umask: u32,
    _rdev: u32,
) -> MknodInv {
    let CPPN { child_path, .. } = common_pre_parent_name(parent, name);

    let _perm = check_perm(req.uid(), req.gid(), req.pid(), &child_path, Access::Create);

    MknodInv {}
}
pub fn inv_mknod_after(callid: CallID, inv: MknodInv, _res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    todo!("Mknod not yet implemented");
}
