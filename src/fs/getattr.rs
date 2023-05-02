use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        let callid = log_call!("GETATTR", "ino={}", ino);
        let ids = set_ids(callid, req,&self.root);
        let res = if let Some(path) = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")
            .get(0)
        {
            log_more!(callid, "path={:?}", path);
            unsafe { stat_path(path).map(|x| x.to_fuse_attr(ino)) }
        } else {
            Err(libc::ENOENT)
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.attr(&TTL, &v),
            Err(v) => reply.error(v),
        }
    }
}
