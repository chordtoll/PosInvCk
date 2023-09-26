use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::create::{inv_create_after, inv_create_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_create(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        let callid = log_call!(
            "CREATE",
            "parent={},name={:?},mode={:o},umask={:o},flags={:o}",
            parent,
            name,
            mode,
            umask,
            flags
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_create_before(
            callid, req, &self.root, parent, name, mode, umask, flags, &mut dl,
        );
        let ids = set_ids(callid, req.into(), Some(umask));
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::open(tgt.as_ptr(), flags, mode);
            if res != -1 {
                stat_path(&child).map(|x| {
                    let ino = ip.insert(x.st_ino, child);
                    (x.to_fuse_attr(ino), res)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_create_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok((attr, fh)) => reply.created(&TTL, &attr, 0, fh.try_into().unwrap(), 0),
            Err(v) => reply.error(v),
        }
    }
}
