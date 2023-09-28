use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::create::{inv_create_after, inv_create_before},
    log_call, log_more, log_res,
    req_rep::{ReplyCreate, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_create(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: &ReplyCreate,
    ) {
        let callid = log_call!(
            "CREATE",
            "parent={},name={:?},mode={:o},umask={:o},flags={:o}",
            parent,
            name,
            mode,
            umask,
            flags
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_create_before(
            callid, &req, &self.root, parent, name, mode, umask, flags, &mut dl,
        );
        let ids = set_ids(callid, req, Some(umask));
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::open(tgt.as_ptr(), flags, mode);
            if res != -1 {
                stat_path(&child).map(|x| {
                    let ino = ip.insert(x.st_ino, child);
                    (x.to_fuse_attr(ino), res)
                })
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_create_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok((attr, fh)) => reply.created(&TTL, &attr, 0, fh.try_into().unwrap(), 0),
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{
        fs::TTL,
        req_rep::{KernelConfig, ReplyCreate, Request},
    };

    #[test]
    fn test_create() {
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
    }
}
