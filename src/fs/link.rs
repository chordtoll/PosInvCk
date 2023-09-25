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
        let mut dl = self.data.lock().unwrap();
        let inv = inv_link_before(callid, req, &self.root, ino, newparent, newname, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(newparent);
        log_more!(callid, "newparent={:?}", p_path);
        let newchild = p_path.join(newname);
        log_more!(callid, "newchild={:?}", newchild);
        let old_file = ip.get(ino);
        let res = unsafe {
            let old = CString::new(old_file.as_os_str().as_bytes()).unwrap();
            let new = CString::new(newchild.as_os_str().as_bytes()).unwrap();
            let res = libc::link(old.as_ptr(), new.as_ptr());
            if res == 0 {
                ip.insert(ino, newchild.clone());
                stat_path(&newchild).map(|x| x.to_fuse_attr(ino))
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_link_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}
