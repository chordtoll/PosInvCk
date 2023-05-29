use std::{ffi::OsString, path::PathBuf};

use crate::{
    invariants::{
        common::{common_pre_parent_name, CPPN},
        perm::{check_perm, Access},
        INODE_PATHS,
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct UnlinkInv {
    parent: u64,
    name: OsString,
    child_exists: bool,
    perm: bool,
    toolong: bool,
    child_path: PathBuf,
    ino: Option<u64>,
}

pub fn inv_unlink_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
) -> UnlinkInv {
    let CPPN {
        child_path,
        ino,
        child_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &child_path, Access::Lookup);

    UnlinkInv {
        parent,
        name: name.to_owned(),
        child_path,
        ino,
        child_exists,
        toolong,
        perm,
    }
}
pub fn inv_unlink_after(callid: CallID, inv: UnlinkInv, res: &Result<(), i32>) {
    log_more!(callid, "invariant={:?}", inv);
    let mut ip = INODE_PATHS.lock().unwrap();
    ip.remove(&inv.child_path);
    match res {
        Ok(()) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(
                inv.child_exists,
                "Failed to return ENOENT on nonexistant directory"
            );
            #[cfg(feature = "check-meta")]
            {
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                let fa = ic.get_mut(&inv.ino.unwrap()).unwrap();
                fa.nlink -= 1;
                if fa.nlink == 0 {
                    ic.remove(&inv.ino.unwrap());
                    #[cfg(feature = "check-data")]
                    {
                        let mut fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
                        fc.remove(&inv.ino.unwrap());
                    }
                    #[cfg(feature = "check-xattr")]
                    {
                        let mut xc = crate::invariants::XATTR_CONTENTS.lock().unwrap();
                        xc.remove(&inv.ino.unwrap());
                    }
                }
            }
            #[cfg(feature = "check-dirs")]
            {
                let mut dc = crate::invariants::DIR_CONTENTS.lock().unwrap();
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .remove(&inv.name);
            }
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.child_exists, "Returned ENOENT on extant directory"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
