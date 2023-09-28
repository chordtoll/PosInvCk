use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::getattr::{inv_getattr_after, inv_getattr_before},
    log_call, log_more, log_res,
    req_rep::{ReplyAttr, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_getattr(&mut self, req: Request, ino: u64, reply: &ReplyAttr) {
        let callid = log_call!("GETATTR", "ino={}", ino);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_getattr_before(callid, &req, &self.root, ino, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);

        let res = unsafe { stat_path(path).map(|x| x.to_fuse_attr(ino)) };

        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_getattr_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(v) => reply.attr(&TTL, &v),
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyAttr, ReplyCreate, Request},
    };

    #[test]
    fn test_getattr() {
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
        let rep_c = ReplyCreate::new();
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
            &rep_c,
        );
        assert_eq!(
            rep_c.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep_c.get().unwrap().1.ino,
                    size: 0,
                    blocks: 0,
                    atime: rep_c.get().unwrap().1.atime,
                    mtime: rep_c.get().unwrap().1.mtime,
                    ctime: rep_c.get().unwrap().1.ctime,
                    crtime: rep_c.get().unwrap().1.crtime,
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
                rep_c.get().unwrap().3,
                0
            ))
        );
        let rep_a = ReplyAttr::new();
        ifs.do_getattr(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            rep_c.get().unwrap().1.ino,
            &rep_a,
        );
        assert_eq!(
            rep_a.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep_c.get().unwrap().1.ino,
                    size: 0,
                    blocks: 0,
                    atime: rep_c.get().unwrap().1.atime,
                    mtime: rep_c.get().unwrap().1.mtime,
                    ctime: rep_c.get().unwrap().1.ctime,
                    crtime: rep_c.get().unwrap().1.crtime,
                    kind: fuser::FileType::RegularFile,
                    perm: 0,
                    nlink: 1,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 4096,
                    flags: 0
                }
            ))
        );
    }
}
