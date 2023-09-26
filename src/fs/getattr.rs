use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::getattr::{inv_getattr_after, inv_getattr_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        let callid = log_call!("GETATTR", "ino={}", ino);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_getattr_before(callid, req, &self.root, ino, &mut dl);
        let ids = set_ids(callid, req.into(), None);
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
