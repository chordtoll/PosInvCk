use libc::c_void;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_res,
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
        let ids = set_ids(callid, req);
        let res = unsafe {
            let offs = libc::lseek(fh as i32, offset, libc::SEEK_SET);
            assert_eq!(offs, offset);
            let mut buf = vec![0u8; size as usize];
            let res = libc::read(fh as i32, buf.as_mut_ptr() as *mut c_void, size as usize);
            buf.truncate(res.try_into().unwrap());
            Ok(buf)
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.data(&v),
            Err(e) => reply.error(e),
        }
    }
}
