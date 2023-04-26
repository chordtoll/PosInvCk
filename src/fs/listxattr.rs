use std::{ffi::CString, os::unix::prelude::OsStrExt};

use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_listxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        let callid = log_call!("LISTXATTR", "ino={},size={:x}", ino, size);
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
            let mut buf = vec![0u8; size.try_into().unwrap()];
            let res = libc::listxattr(
                tgt.as_ptr(),
                buf.as_mut_ptr() as *mut i8,
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
        match res {
            Ok(v) => reply.data(&v),
            Err(v) => reply.error(v),
        }
    }
}
