use std::{ffi::CString, mem::MaybeUninit, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_statfs(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        let callid = log_call!("STATFS", "ino={}", ino);
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
                v.f_blocks.try_into().unwrap(),
                v.f_bfree.try_into().unwrap(),
                v.f_bavail.try_into().unwrap(),
                v.f_files.try_into().unwrap(),
                v.f_ffree.try_into().unwrap(),
                v.f_bsize.try_into().unwrap(),
                v.f_namelen.try_into().unwrap(),
                v.f_frsize.try_into().unwrap(),
            ),
            Err(v) => reply.error(v),
        }
    }
}
