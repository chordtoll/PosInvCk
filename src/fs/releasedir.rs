use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_releasedir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("RELEASEDIR", "ino={},fh={},flags={}", ino, fh, flags);
        let ids = set_ids(callid, req);
        let dirp = self.dir_fhs.remove(&fh).unwrap();
        let res = unsafe {
            let res = libc::closedir(dirp);
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
