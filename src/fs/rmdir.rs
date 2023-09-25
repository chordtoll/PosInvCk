use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::rmdir::{inv_rmdir_after, inv_rmdir_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_rmdir(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("RMDIR", "parent={},name={:?}", parent, name);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_rmdir_before(callid, req, &self.root, parent, name, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::rmdir(tgt.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_rmdir_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(()) => {
                dl.INODE_PATHS.remove(&child);
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}
