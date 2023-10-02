use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::rename::{inv_rename_after, inv_rename_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEmpty, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_rename(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        newparent: u64,
        newname: &std::ffi::OsStr,
        flags: u32,
        reply: &ReplyEmpty,
    ) {
        let callid = log_call!(
            "RENAME",
            "parent={},name={:?},newparent={},newname={:?},flags={:x}",
            parent,
            name,
            newparent,
            newname,
            flags
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_rename_before(
            callid, &req, &self.root, parent, name, newparent, newname, flags, &mut dl,
        );
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let old_parent = ip.get(parent);
        log_more!(callid, "old_parent={:?}", old_parent);
        let old_child = old_parent.join(name);
        log_more!(callid, "old_child={:?}", old_child);
        let new_parent = ip.get(newparent);
        log_more!(callid, "new_parent={:?}", new_parent);
        let new_child = new_parent.join(newname);
        log_more!(callid, "new_child={:?}", new_child);
        let res = unsafe {
            let old = CString::new(old_child.as_os_str().as_bytes()).unwrap();
            let new = CString::new(new_child.as_os_str().as_bytes()).unwrap();
            let res = libc::rename(old.as_ptr(), new.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_rename_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(()) => {
                dl.INODE_PATHS.rename(old_child, new_child);
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, ReplyEmpty, ReplyEntry, Request},
    };

    #[test]
    fn test_rename() {
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
        let rep_r = ReplyEmpty::new();
        ifs.do_rename(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            1,
            &OsString::from("bar"),
            0,
            &rep_r,
        );
        assert_eq!(rep_r.get(),Ok(()));
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
        assert_eq!(rep_old.get(), Err(libc::ENOENT));
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
        )
    }
}
