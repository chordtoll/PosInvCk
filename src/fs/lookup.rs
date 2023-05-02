use crate::{
    fs::{restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_lookup(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let callid = log_call!("LOOKUP", "parent={},name={:?}", parent, name);
        let ids = set_ids(callid, req, &self.root);
        let p_path = self.paths.get(parent);
        log_more!(callid, "parent={:?}", p_path);
        let child = p_path.join(name);
        log_more!(callid, "child={:?}", child);
        let res = unsafe { stat_path(&child) }.map(|v| {
            let ino = self.paths.insert(v.st_ino, child);
            v.to_fuse_attr(ino)
        });
        log_res!(callid, "{:#?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.entry(&TTL, &v, 0),
            Err(v) => reply.error(v),
        }
    }
}
