use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::link::{inv_link_after, inv_link_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEntry, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_link(
        &mut self,
        req: Request,
        ino: u64,
        newparent: u64,
        newname: &std::ffi::OsStr,
        reply: &ReplyEntry,
    ) {
        let callid = log_call!(
            "LINK",
            "ino={},newparent={},newname={:?}",
            ino,
            newparent,
            newname,
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_link_before(callid, &req, &self.root, ino, newparent, newname, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(newparent);
        log_more!(callid, "newparent={:?}", p_path);
        let newchild = p_path.join(newname);
        log_more!(callid, "newchild={:?}", newchild);
        let old_file = ip.get(ino);
        let res = unsafe {
            let old = CString::new(old_file.as_os_str().as_bytes()).unwrap();
            let new = CString::new(newchild.as_os_str().as_bytes()).unwrap();
            let res = libc::link(old.as_ptr(), new.as_ptr());
            if res == 0 {
                ip.insert(ino, newchild.clone());
                stat_path(&newchild).map(|x| x.to_fuse_attr(ino))
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_link_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, ReplyEntry, Request},
    };

    #[test]
    fn test_link() {
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
        let rep_l = ReplyEntry::new();
        ifs.do_link(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            rep_c.get().unwrap().1.ino,
            1,
            &OsString::from("bar"),
            &rep_l,
        );
        let rep_old = ReplyEntry::new();
        ifs.do_lookup(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            &rep_old,
        );
        let rep_new = ReplyEntry::new();
        ifs.do_lookup(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("bar"),
            &rep_new,
        );
        assert_eq!(
            rep_old.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep_c.get().unwrap().1.ino,
                    size: 0,
                    blocks: 0,
                    atime: rep_c.get().unwrap().1.atime,
                    mtime: rep_c.get().unwrap().1.mtime,
                    ctime: rep_new.get().unwrap().1.ctime,
                    crtime: rep_c.get().unwrap().1.crtime,
                    kind: fuser::FileType::RegularFile,
                    perm: 0,
                    nlink: 2,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 4096,
                    flags: 0
                },
                0
            ))
        );
        assert_eq!(
            rep_new.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep_c.get().unwrap().1.ino,
                    size: 0,
                    blocks: 0,
                    atime: rep_c.get().unwrap().1.atime,
                    mtime: rep_c.get().unwrap().1.mtime,
                    ctime: rep_new.get().unwrap().1.ctime,
                    crtime: rep_c.get().unwrap().1.crtime,
                    kind: fuser::FileType::RegularFile,
                    perm: 0,
                    nlink: 2,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 4096,
                    flags: 0
                },
                0
            ))
        )
    }
}
