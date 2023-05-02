use std::{ffi::CString, os::unix::prelude::OsStrExt};

use libc::c_void;

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_setxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        value: &[u8],
        flags: i32,
        position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!(
            "SETXATTR",
            "ino={},name={:?},value={},flags={},position={}",
            ino,
            name,
            String::from_utf8_lossy(value),
            flags,
            position
        );
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req);
        let path = self.paths.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let nm = CString::new(name.as_bytes()).unwrap();
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::setxattr(
                tgt.as_ptr(),
                nm.as_ptr(),
                value.as_ptr() as *mut c_void,
                value.len(),
                flags,
            );
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
