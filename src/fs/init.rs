use libc::c_int;

use super::InvFS;

impl InvFS {
    pub fn do_init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        Ok(())
    }
}
