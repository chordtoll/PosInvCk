use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_fsync(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("FSYNC", "ino={},fh={},datasync={}", ino, fh, datasync);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req);
        let res = unsafe {
            let res = if datasync {
                libc::fdatasync(fh.try_into().unwrap())
            } else {
                libc::fsync(fh.try_into().unwrap())
            };
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
            Ok(()) => {
                reply.ok();
            }
            Err(v) => reply.error(v),
        }
    }
}
