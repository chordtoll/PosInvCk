use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
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
            "parent={},name={:?},mode={:x},umask={:x},flags={:x}",
            parent,
            name,
            mode,
            umask,
            flags
        );
        let ids = set_ids(callid, req,&self.root);
        let p_path = &self
            .paths
            .get(parent as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::open(tgt.as_ptr(), flags, mode);
            if res != -1 {
                stat_path(&child).map(|x| {
                    let ino = self.paths.len().try_into().unwrap();
                    self.paths.push(vec![child]);
                    (x.to_fuse_attr(ino), res)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok((attr, fh)) => reply.created(&TTL, &attr, 0, fh.try_into().unwrap(), 0),
            Err(v) => reply.error(v),
        }
    }
}
