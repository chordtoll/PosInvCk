use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_release(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!(
            "RELEASE",
            "ino={},fh={},flags={},lock_owner={:?},flush={}",
            ino,
            fh,
            flags,
            lock_owner,
            flush
        );
        let ids = set_ids(callid, req, &self.root);
        let res = unsafe {
            let res = libc::close(fh.try_into().unwrap());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(()) => reply.ok(),
            Err(e) => reply.error(e),
        }
    }
}
