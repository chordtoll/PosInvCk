use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_open(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        let callid = log_call!("OPEN", "ino={},flags={:x}", ino, flags);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req);
        let path = self.paths.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::open(tgt.as_ptr(), flags);
            if res != -1 {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(v) => reply.opened(v.try_into().unwrap(), 0),
            Err(v) => reply.error(v),
        }
    }
}
