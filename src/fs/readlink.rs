use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_readlink(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        let callid = log_call!("READLINK", "ino={}", ino);
        let ids = set_ids(callid, req,&self.root);
        let path = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let mut buf = vec![0u8; (libc::PATH_MAX + 1).try_into().unwrap()];
            let res = libc::readlink(tgt.as_ptr(), buf.as_mut_ptr() as *mut i8, buf.len());
            if res != -1 {
                assert_ne!(res, (libc::PATH_MAX + 1).try_into().unwrap(),"overflowed readlink");
                buf.truncate(res.try_into().unwrap());
                Ok(buf)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.data(&v),
            Err(v) => reply.error(v),
        }
    }
}
