use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::mkdir::{inv_mkdir_after, inv_mkdir_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_mkdir(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        let callid = log_call!(
            "MKDIR",
            "parent={},name={:?},mode={:x},umask={:x}",
            parent,
            name,
            mode,
            umask
        );
        let cwd = chdirin(&self.root);
        let inv = inv_mkdir_before(callid, req, parent, name, mode, umask);
        let ids = set_ids(callid, req);
        let p_path = self.paths.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::mkdir(tgt.as_ptr(), mode);
            if res == 0 {
                stat_path(&child).map(|x| {
                    let ino = self.paths.insert(x.st_ino, child);
                    x.to_fuse_attr(ino)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_mkdir_after(callid, inv, &res);
        chdirout(cwd);
        match res {
            Ok(v) => reply.entry(&TTL, &v, 0),
            Err(v) => reply.error(v),
        }
    }
}
