use libc::c_void;

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::write::{inv_write_after, inv_write_before},
    log_call, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_write(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        let callid = log_call!(
            "WRITE",
            "ino={},fh={:x},offset={:x},data=[{:x}],write_flags={:x},flags={:x},lock_owner={:?}",
            ino,
            fh,
            offset,
            data.len(),
            write_flags,
            flags,
            lock_owner
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_write_before(
            callid,
            req,
            &self.root,
            ino,
            fh,
            offset,
            data,
            write_flags,
            flags,
            lock_owner,
            &mut dl,
        );
        let ids = set_ids(callid, req.into(), None);
        let res = unsafe {
            let offs = libc::lseek(fh as i32, offset, libc::SEEK_SET);
            assert_eq!(offs, offset, "failed to seek");
            let res = libc::write(fh as i32, data.as_ptr() as *mut c_void, data.len());
            if res != -1 {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_write_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(v) => reply.written(v.try_into().unwrap()),
            Err(e) => reply.error(e),
        }
    }
}
