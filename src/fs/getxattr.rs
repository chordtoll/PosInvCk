use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};
use libc::c_void;

use super::InvFS;

impl InvFS {
    pub fn do_getxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        let callid = log_call!("GETXATTR", "ino={},name={:?},size={:x}", ino, name, size);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req, None);
        let dl = self.data.lock().unwrap();
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let nm = CString::new(name.as_bytes()).unwrap();
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let mut buf = vec![0u8; size.try_into().unwrap()];
            let res = libc::getxattr(
                tgt.as_ptr(),
                nm.as_ptr(),
                buf.as_mut_ptr() as *mut c_void,
                size.try_into().unwrap(),
            );
            if res != -1 {
                buf.truncate(size.try_into().unwrap());
                Ok(buf)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(v) => reply.data(&v),
            Err(v) => reply.error(v),
        }
    }
}
