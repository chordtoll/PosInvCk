use libc::c_void;

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::read::{inv_read_after, inv_read_before},
    log_call, log_res,
    logwrapper::LogWrapper,
};

use super::InvFS;

impl InvFS {
    pub fn do_read(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        let callid = log_call!(
            "READ",
            "ino={},fh={:x},offset={:x},size={:x},flags={:x},lock_owner={:?}",
            ino,
            fh,
            offset,
            size,
            flags,
            lock_owner
        );
        let cwd = chdirin(&self.root);
        let inv = inv_read_before(callid, req, ino, fh, offset, size, flags, lock_owner);
        let ids = set_ids(callid, req);
        let res = unsafe {
            let offs = libc::lseek(fh as i32, offset, libc::SEEK_SET);
            assert_eq!(offs, offset, "failed to seek");
            let mut buf = vec![0u8; size as usize];
            let res = libc::read(fh as i32, buf.as_mut_ptr() as *mut c_void, size as usize);
            buf.truncate(res.try_into().unwrap());
            Ok(buf)
        };
        log_res!(callid, "{}", res.lw());
        restore_ids(ids);
        inv_read_after(callid, inv, &res);
        chdirout(cwd);
        match res {
            Ok(v) => reply.data(&v),
            Err(e) => reply.error(e),
        }
    }
}
