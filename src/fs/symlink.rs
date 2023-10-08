use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::symlink::{inv_symlink_after, inv_symlink_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEntry, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_symlink(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        link: &std::path::Path,
        reply: &ReplyEntry,
    ) {
        let callid = log_call!(
            "SYMLINK",
            "parent={},name={:?},link={:?}",
            parent,
            name,
            link
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_symlink_before(callid, &req, &self.root, parent, name, link, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let lk = CString::new(link.as_os_str().as_bytes()).unwrap();
            let res = libc::symlink(lk.as_ptr(), tgt.as_ptr());
            if res == 0 {
                stat_path(&child).map(|x| {
                    let ino = ip.insert(x.st_ino, child);
                    x.to_fuse_attr(ino)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_symlink_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(attr) => reply.entry(&TTL, &attr, 0),
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, ReplyEntry, Request},
    };

    #[test]
    fn test_symlink() {
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
        ifs.do_symlink(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("bar"),
            &PathBuf::from("foo"),
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
                0
            ))
        );
        assert_eq!(
            rep_new.get(),
            Ok((
                TTL,
                fuser::FileAttr {
                    ino: rep_l.get().unwrap().1.ino,
                    size: 3,
                    blocks: 0,
                    atime: rep_l.get().unwrap().1.atime,
                    mtime: rep_l.get().unwrap().1.mtime,
                    ctime: rep_l.get().unwrap().1.ctime,
                    crtime: rep_l.get().unwrap().1.crtime,
                    kind: fuser::FileType::Symlink,
                    perm: 0o777,
                    nlink: 1,
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
