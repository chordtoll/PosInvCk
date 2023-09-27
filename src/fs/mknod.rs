use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::mknod::{inv_mknod_after, inv_mknod_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEntry, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_mknod(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: &ReplyEntry,
    ) {
        let callid = log_call!(
            "MKNOD",
            "parent={},name={:?},mode={:o},umask={:o},rdev={}",
            parent,
            name,
            mode,
            umask,
            rdev
        );
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_mknod_before(
            callid, &req, &self.root, parent, name, mode, umask, rdev, &mut dl,
        );
        let ids = set_ids(callid, req, Some(umask));
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::mknod(tgt.as_ptr(), mode, rdev.into());
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
        inv_mknod_after(callid, inv, &res, &mut dl);
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
        req_rep::{KernelConfig, ReplyEntry, Request},
    };

    #[test]
    fn test_mknod() {
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
        let rep = ReplyEntry::new();
        ifs.do_mknod(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            0x8000,
            0,
            0,
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
            ))
        );
    }
}
