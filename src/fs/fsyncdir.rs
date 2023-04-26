use crate::{log_call, log_res};

use super::InvFS;

impl InvFS {
    pub fn do_fsyncdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        let callid = log_call!("FSYNC", "ino={},fh={},datasync={}", ino, fh, datasync);
        log_res!(
            callid,
            "We assume the underlying filesystem syncs directories."
        );
        reply.ok()
    }
}
