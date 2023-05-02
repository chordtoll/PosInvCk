use std::path::PathBuf;

use maplit::btreeset;

use crate::{fs::InvFS, logging::CallID};

use super::INODE_PATHS;

pub struct InitInv {
    root: PathBuf,
}

pub fn inv_init_before(
    callid: CallID,
    fs: &InvFS,
    req: &fuser::Request<'_>,
    config: &fuser::KernelConfig,
) -> InitInv {
    InitInv {
        root: fs.root.clone(),
    }
}
pub fn inv_init_after(callid: CallID, inv: InitInv, res: Result<(), i32>) {
    let mut ip = INODE_PATHS.lock().unwrap();
    ip.insert(1, btreeset![inv.root]);
}
