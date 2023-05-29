use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::unlink::{inv_unlink_after, inv_unlink_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_unlink(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("UNLINK", "parent={},name={:?}", parent, name);
        let cwd = chdirin(&self.root);
        let inv = inv_unlink_before(callid, req, parent, name);
        let ids = set_ids(callid, req);
        let p_path = self.paths.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::unlink(tgt.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_unlink_after(callid, inv, &res);
        chdirout(cwd);
        match res {
            Ok(()) => {
                self.paths.remove(&child);
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}
