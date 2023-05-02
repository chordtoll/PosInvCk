use std::{ffi::CString, mem::MaybeUninit, os::unix::prelude::OsStrExt};

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_statfs(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        let callid = log_call!("STATFS", "ino={}", ino);
        let ids = set_ids(callid, req, &self.root);
        let path = self.paths.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let mut res = MaybeUninit::zeroed().assume_init();
            let rc = libc::statfs(tgt.as_ptr(), &mut res);
            if rc == 0 {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.statfs(
                v.f_blocks,
                v.f_bfree,
                v.f_bavail,
                v.f_files,
                v.f_ffree,
                v.f_bsize.try_into().unwrap(),
                v.f_namelen.try_into().unwrap(),
                v.f_frsize.try_into().unwrap(),
            ),
            Err(v) => reply.error(v),
        }
    }
}
