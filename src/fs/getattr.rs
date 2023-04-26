use path_clean::PathClean;

use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
    pretty_print::PPStat,
};

use super::InvFS;

impl InvFS {
    pub fn do_getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        let callid = log_call!("GETATTR", "ino={}", ino);
        let ids = set_ids(callid, req);
        let res = if let Some(path) = &self
            .paths
            .get(ino as usize)
            .expect("Accessing an inode we haven't seen before")
            .get(0)
        {
            log_more!(callid, "path={:?}", path);
            let tgt_path = self.base.join(path).clean();
            log_more!(callid, "tgt_path={:?}", tgt_path);
            unsafe { stat_path(&tgt_path) }
        } else {
            Err(libc::ENOENT)
        };
        log_res!(callid, "{}", res.ppstat());
        restore_ids(ids);
        match res {
            Ok(v) => reply.attr(&TTL, &v.to_fuse_attr(ino)),
            Err(v) => reply.error(v),
        }
    }
}
