use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
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
        let ids = set_ids(callid, req, &self.root);
        let p_path = &self
            .paths
            .get(newparent as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "newparent={:?}", p_path);
        let newchild = p_path.join(newname);
        log_more!(callid, "newchild={:?}", newchild);
        let old_file = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        let res = unsafe {
            let old = CString::new(old_file.as_os_str().as_bytes()).unwrap();
            let new = CString::new(newchild.as_os_str().as_bytes()).unwrap();
            let res = libc::link(old.as_ptr(), new.as_ptr());
            if res == 0 {
                self.paths
                    .get_mut(ino as usize)
                    .unwrap()
                    .push(newchild.clone());
                stat_path(&newchild).map(|x| x.to_fuse_attr(ino))
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
        restore_ids(ids);
    }
}
