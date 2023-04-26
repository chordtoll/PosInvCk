use std::{ffi::CString, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_mknod(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        let callid = log_call!(
            "MKNOD",
            "parent={},name={:?},mode={:x},umask={:x},rdev={}",
            parent,
            name,
            mode,
            umask,
            rdev
        );
        let ids = set_ids(callid, req);
        let path = &self
            .paths
            .get(parent as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "parent={:?}", path);
        let child = path.join(name);
        log_more!(callid, "child={:?}", child);
        let tgt_path = self.base.join(child.clone()).clean();
        log_more!(callid, "tgt_path={:?}", tgt_path);
        let res = unsafe {
            let tgt = CString::new(tgt_path.as_os_str().as_bytes()).unwrap();
            let res = libc::mknod(tgt.as_ptr(), mode, rdev.into());
            if res == 0 {
                stat_path(&tgt_path).map(|x| {
                    let ino = self.paths.len().try_into().unwrap();
                    self.paths.push(vec![child]);
                    x.to_fuse_attr(ino)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}
