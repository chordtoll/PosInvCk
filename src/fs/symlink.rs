use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::symlink::{inv_symlink_after, inv_symlink_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_symlink(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        link: &std::path::Path,
        reply: fuser::ReplyEntry,
    ) {
        let callid = log_call!(
            "SYMLINK",
            "parent={},name={:?},link={:?}",
            parent,
            name,
            link
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_symlink_before(callid, req, &self.root, parent, name, link, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let lk = CString::new(link.as_os_str().as_bytes()).unwrap();
            let res = libc::symlink(lk.as_ptr(), tgt.as_ptr());
            if res == 0 {
                stat_path(&child).map(|x| {
                    let ino = ip.insert(x.st_ino, child);
                    x.to_fuse_attr(ino)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_symlink_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}
