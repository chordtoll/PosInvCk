use libc::c_void;

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::write::{inv_write_after, inv_write_before},
    log_call, log_res,
    req_rep::{ReplyWrite, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_write(
        &mut self,
        req: Request,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: &ReplyWrite,
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
            &req,
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
        let ids = set_ids(callid, req, None);
        let res = unsafe {
            println!("FH: {:?}", fh);
            let offs = libc::lseek(fh as i32, offset, libc::SEEK_SET);
            assert_eq!(
                offs,
                offset,
                "failed to seek: {}",
                *libc::__errno_location()
            );
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, ReplyOpen, ReplyWrite, Request},
    };

    #[test]
    fn test_write() {
        let mut ifs = crate::test::create_ifs();
        ifs.do_init(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            &KernelConfig::empty(),
        )
        .unwrap();
        let rep = ReplyCreate::new();
        ifs.do_create(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            0,
            0,
            libc::O_CREAT,
            &rep,
        );
        assert_eq!(
            rep.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep.get().unwrap().1.ino,
                    size: 0,
                    blocks: 0,
                    atime: rep.get().unwrap().1.atime,
                    mtime: rep.get().unwrap().1.mtime,
                    ctime: rep.get().unwrap().1.ctime,
                    crtime: rep.get().unwrap().1.crtime,
                    kind: fuser::FileType::RegularFile,
                    perm: 0,
                    nlink: 1,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 4096,
                    flags: 0
                },
                0,
                rep.get().unwrap().3,
                0
            ))
        );
        let o_rep = ReplyOpen::new();
        ifs.do_open(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            rep.get().unwrap().1.ino,
            libc::O_WRONLY,
            &o_rep,
        );
        let w_rep = ReplyWrite::new();
        ifs.do_write(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            rep.get().unwrap().1.ino,
            o_rep.get().unwrap().0,
            0,
            &[b'f', b'o', b'o'],
            0,
            0,
            None,
            &w_rep,
        );
        assert_eq!(w_rep.get(), Ok(3));
        let idlu = ifs.data.lock().unwrap();
        assert_eq!(
            idlu.INV_FILE_CONTENTS.get(&rep.get().unwrap().1.ino),
            Some(&vec![b'f', b'o', b'o'])
        )
    }
}
