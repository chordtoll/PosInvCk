use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::removexattr::{inv_removexattr_after, inv_removexattr_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_removexattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("REMOVEXATTR", "ino={},name={:?}", ino, name);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_removexattr_before(callid, req, &self.root, ino, name, &mut dl);
        let ids = set_ids(callid, req.into(), None);
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let nm = CString::new(name.as_bytes()).unwrap();
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::removexattr(tgt.as_ptr(), nm.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);

        restore_ids(ids);
        inv_removexattr_after(callid, inv, &res);
        chdirout(cwd);
        match res {
            Ok(()) => reply.ok(),
            Err(v) => reply.error(v),
        }
    }
}
