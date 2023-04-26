use crate::{log_call, log_res};

use super::InvFS;

impl InvFS {
    pub fn do_forget(&mut self, _req: &fuser::Request<'_>, _ino: u64, _nlookup: u64) {
        let callid = log_call!("FORGET", "ino={}", _ino);
        log_res!(callid, "We currently take no action here.")
    }
}
