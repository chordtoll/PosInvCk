use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::link::{inv_link_after, inv_link_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_link(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        newparent: u64,
        newname: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let callid = log_call!(
            "LINK",
            "ino={},newparent={},newname={:?}",
            ino,
            newparent,
            newname,
        );
        let cwd = chdirin(&self.root);
        let inv = inv_link_before(callid, req, ino, newparent, newname);
        let ids = set_ids(callid, req);
        let p_path = self.paths.get(newparent);
        log_more!(callid, "newparent={:?}", p_path);
        let newchild = p_path.join(newname);
        log_more!(callid, "newchild={:?}", newchild);
        let old_file = self.paths.get(ino);
        let res = unsafe {
            let old = CString::new(old_file.as_os_str().as_bytes()).unwrap();
            let new = CString::new(newchild.as_os_str().as_bytes()).unwrap();
            let res = libc::link(old.as_ptr(), new.as_ptr());
            if res == 0 {
                self.paths.insert(ino, newchild.clone());
                stat_path(&newchild).map(|x| x.to_fuse_attr(ino))
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_link_after(callid, inv, &res);
        chdirout(cwd);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}
