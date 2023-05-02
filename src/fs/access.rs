use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_access(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        mask: i32,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("ACCESS", "ino={},mask={:x}", ino, mask);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req);
        let path = self.paths.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::euidaccess(tgt.as_ptr(), mask);
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(()) => reply.ok(),
            Err(v) => reply.error(v),
        }
    }
}
