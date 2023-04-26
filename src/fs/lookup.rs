use path_clean::PathClean;

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
        let ids = set_ids(callid, req);
        let path = &self
            .paths
            .get(parent as usize)
            .expect("Accessing an inode we haven't seen before")[0];
        log_more!(callid, "parent={:?}", path);
        let child = path.join(name);
        log_more!(callid, "child={:?}", child);
        let tgt_path = self.base.join(child.clone()).clean();
        log_more!(callid, "tgt_path={:?}", tgt_path);
        let res = if let Some((ino, _)) = self
            .paths
            .iter()
            .enumerate()
            .find(|(_, e)| e.contains(&child))
        {
            log_more!(callid, "existing inode: {}", ino);
            let res = unsafe { stat_path(&tgt_path) };
            match res {
                Ok(v) => {
                    let attr = v.to_fuse_attr(ino.try_into().unwrap());
                    Ok(attr)
                }
                Err(v) => Err(v),
            }
        } else {
            let res = unsafe { stat_path(&tgt_path) };
            match res {
                Ok(v) => {
                    let ino = self.paths.len();
                    self.paths.push(vec![child]);
                    let attr = v.to_fuse_attr(ino.try_into().unwrap());
                    Ok(attr)
                }
                Err(v) => Err(v),
            }
        };
        log_res!(callid, "{:#?}", res);
        restore_ids(ids);
        match res {
            Ok(v) => reply.entry(&TTL, &v, 0),
            Err(v) => reply.error(v),
        }
    }
}
