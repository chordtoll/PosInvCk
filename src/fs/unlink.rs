use std::{ffi::CString, os::unix::prelude::OsStrExt};

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids},
    invariants::fs::unlink::{inv_unlink_after, inv_unlink_before},
    log_call, log_more, log_res,
    req_rep::{ReplyEmpty, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_unlink(
        &mut self,
        req: Request,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: &ReplyEmpty,
    ) {
        let callid = log_call!("UNLINK", "parent={},name={:?}", parent, name);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_unlink_before(callid, &req, &self.root, parent, name, &mut dl);
        let ids = set_ids(callid, req, None);
        let ip = &mut dl.INODE_PATHS;
        let p_path = ip.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe {
            let tgt = CString::new(child.as_os_str().as_bytes()).unwrap();
            let res = libc::unlink(tgt.as_ptr());
            if res == 0 {
                Ok(())
            } else {
                Err(*libc::__errno_location())
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        inv_unlink_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(()) => {
                dl.INODE_PATHS.remove(&child);
                reply.ok()
            }
            Err(v) => reply.error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::req_rep::{KernelConfig, ReplyCreate, ReplyEmpty, Request};

    #[test]
    fn test_unlink() {
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
        assert!(rep.get().is_ok());
        let rep = ReplyEmpty::new();
        ifs.do_unlink(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            1,
            &OsString::from("foo"),
            &rep,
        );
    }
}
