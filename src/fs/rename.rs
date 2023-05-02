use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_rename(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        newparent: u64,
        newname: &std::ffi::OsStr,
        flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!(
            "RENAME",
            "parent={},name={:?},newparent={},newname={:?},flags={:x}",
            parent,
            name,
            newparent,
            newname,
            flags
        );
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req);
        let old_parent = self.paths.get(parent);
        log_more!(callid, "old_parent={:?}", old_parent);
        let old_child = old_parent.join(name);
        log_more!(callid, "old_child={:?}", old_child);
        let new_parent = self.paths.get(newparent);
        log_more!(callid, "new_parent={:?}", new_parent);
        let new_child = new_parent.join(newname);
        log_more!(callid, "new_child={:?}", new_child);
        let res = unsafe {
            let old = CString::new(old_child.as_os_str().as_bytes()).unwrap();
            let new = CString::new(new_child.as_os_str().as_bytes()).unwrap();
            let res = libc::rename(old.as_ptr(), new.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(()) => {
                log_more!(callid, "{:?}", self.paths);
                self.paths.rename(old_child, new_child);
                log_more!(callid, "{:?}", self.paths);
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}
