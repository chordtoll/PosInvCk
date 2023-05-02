use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        let callid = log_call!("GETATTR", "ino={}", ino);
        let ids = set_ids(callid, req, &self.root);
        let path = self.paths.get(ino);
        log_more!(callid, "path={:?}", path);

        let res = unsafe { stat_path(path).map(|x| x.to_fuse_attr(ino)) };

        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.attr(&TTL, &v),
            Err(v) => reply.error(v),
        }
    }
}
