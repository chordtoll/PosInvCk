use libc::c_int;

use crate::{
    invariants::init::{inv_init_after, inv_init_before},
    log_call,
};

use super::InvFS;

impl InvFS {
    pub fn do_init(
        &mut self,
        req: &fuser::Request<'_>,
        config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        let callid = log_call!("INIT", "config={:?}", config);
        let inv = inv_init_before(callid, self, req, config);
        let res = Ok(());
        inv_init_after(callid, inv, res);
        res
    }
}
