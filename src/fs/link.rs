use std::{ffi::CString, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
    pretty_print::PPStat,
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
        let ids = set_ids(callid, req);
        let path = &self
            .paths
            .get(newparent as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "newparent={:?}", path);
        let child = path.join(newname);
        log_more!(callid, "newchild={:?}", child);
        let new_path = self.base.join(child.clone()).clean();
        log_more!(callid, "new_path={:?}", new_path);
        let old_file = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        let old_path = self.base.join(old_file.clone()).clean();
        log_more!(callid, "old_path={:?}", old_path);
        let res = unsafe {
            let old = CString::new(old_path.as_os_str().as_bytes()).unwrap();
            let new = CString::new(new_path.as_os_str().as_bytes()).unwrap();
            let res = libc::link(old.as_ptr(), new.as_ptr());
            if res == 0 {
                self.paths
                    .get_mut(ino as usize)
                    .unwrap()
                    .push(child.clone());
                stat_path(&new_path)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{}", res.ppstat());
        match res {
            Ok(attr) => reply.entry(&TTL, &attr.to_fuse_attr(ino), 0),
            Err(v) => reply.error(v),
        }
        restore_ids(ids);
    }
}
