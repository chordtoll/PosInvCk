use std::{ffi::CString, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_opendir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        let callid = log_call!("OPENDIR", "ino={},flags={:x}", ino, flags);
        let ids = set_ids(callid, req);
        let path = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "path={:?}", path);
        let tgt_path = self.base.join(path).clean();
        log_more!(callid, "tgt_path={:?}", tgt_path);
        let res = unsafe {
            let tgt = CString::new(tgt_path.as_os_str().as_bytes()).unwrap();
            let res = libc::opendir(tgt.as_ptr());
            if res != std::ptr::null_mut() {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => {
                let fh = self.dir_fhs.last_entry().map(|x| *x.key()).unwrap_or(0) + 1;
                self.dir_fhs.insert(fh, v);
                reply.opened(fh, 0)
            }
            Err(v) => reply.error(v),
        }
    }
}
