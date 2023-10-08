use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    log_call, log_more, log_res,
    req_rep::{ReplyOpen, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_open(&mut self, req: Request, ino: u64, flags: i32, reply: &ReplyOpen) {
        let callid = log_call!("OPEN", "ino={},flags={:x}", ino, flags);
        let cwd = chdirin(&self.root);
        let ids = set_ids(callid, req, None);
        let dl = self.data.lock().unwrap();
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            let res = libc::open(tgt.as_ptr(), flags);
            if res != -1 {
                Ok(res)
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        chdirout(cwd);
        match res {
            Ok(v) => reply.opened(v.try_into().unwrap(), 0),
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, ReplyOpen, Request},
    };

    #[test]
    fn test_open() {
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
            0,
            &o_rep,
        );
        assert!(o_rep.get().is_ok());
        assert_ne!(o_rep.get().unwrap().0, 0);
        assert_eq!(o_rep.get().unwrap().1, 0);
    }
}
