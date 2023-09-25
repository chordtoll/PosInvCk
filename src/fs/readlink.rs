use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_readlink(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        let callid = log_call!("READLINK", "ino={}", ino);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req, None);
        let dl = self.data.lock().unwrap();
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let mut buf = vec![0u8; (libc::PATH_MAX + 1).try_into().unwrap()];
            let res = libc::readlink(tgt.as_ptr(), buf.as_mut_ptr() as *mut i8, buf.len());
            if res != -1 {
                assert_ne!(
                    res,
                    isize::try_from(libc::PATH_MAX + 1).unwrap(),
                    "overflowed readlink"
                );
                buf.truncate(res.try_into().unwrap());
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
