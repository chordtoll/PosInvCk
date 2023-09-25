use libc::c_int;

use crate::{
    invariants::fs::init::{inv_init_after, inv_init_before},
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
        self.data
            .lock()
            .unwrap()
            .INODE_PATHS
            .insert(1, self.root.clone());
        let res = Ok(());
        inv_init_after(callid, inv, &res, &mut self.data.lock().unwrap());
        res
    }
}
