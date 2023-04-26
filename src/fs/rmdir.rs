use std::{ffi::CString, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_rmdir(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("RMDIR", "parent={},name={:?}", parent, name);
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
            let res = libc::rmdir(tgt.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(()) => {
                self.paths
                    .iter_mut()
                    .for_each(|x| x.retain(|x| *x != child));
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}
