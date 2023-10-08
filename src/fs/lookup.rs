use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::lookup::{inv_lookup_after, inv_lookup_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEntry, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_lookup(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: &ReplyEntry,
    ) {
        let callid = log_call!("LOOKUP", "parent={},name={:?}", parent, name);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_lookup_before(callid, &req, &self.root, parent, name, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe { stat_path(&child) }.map(|v| {
            let ino = ip.insert(v.st_ino, child);
            v.to_fuse_attr(ino)
        });
        log_res!(callid, "{:#?}", res);
        restore_ids(ids);
        inv_lookup_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(v) => reply.entry(&TTL, &v, 0),
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
    fn test_lookup() {
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
        assert!(rep_c.get().is_ok());
        let rep_l = ReplyEntry::new();
        ifs.do_lookup(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            &rep_l,
        );
        assert_eq!(
            rep_l.get(),
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
