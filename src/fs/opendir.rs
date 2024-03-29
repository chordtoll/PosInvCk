use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
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
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req.into(), None);
        let dl = self.data.lock().unwrap();
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::opendir(tgt.as_ptr());
            if !res.is_null() {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(v) => {
                let fh = self.dir_fhs.iter().last().map(|(x, _)| *x).unwrap_or(0) + 1;
                self.dir_fhs.insert(fh, v);
                reply.opened(fh, 0)
            }
            Err(v) => reply.error(v),
        }
    }
}
