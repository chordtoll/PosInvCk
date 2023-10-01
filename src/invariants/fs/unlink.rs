use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::MutexGuard,
};

use crate::{
    invariants::{
        common::{common_pre_parent_name, CPPN},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID, req_rep::Request,
};

#[derive(Debug)]
#[must_use]
pub struct UnlinkInv {
    parent: u64,
    name: OsString,
    child_exists: bool,
    perm: Option<i32>,
    toolong: bool,
    child_path: PathBuf,
    ino: Option<u64>,
}

pub fn inv_unlink_before(
    _callid: CallID,
    req: &Request,
    base: &Path,
    parent: u64,
    name: &std::ffi::OsStr,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> UnlinkInv {
    let CPPN {
        child_path,
        ino,
        child_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name, fs_data);

    let perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &child_path,
        base,
        Access::Delete,
    );

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
pub fn inv_unlink_after(
    callid: CallID,
    inv: UnlinkInv,
    res: &Result<(), i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(()) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(
                inv.child_exists,
                "Failed to return ENOENT on nonexistant directory"
            );
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                let fa = ic.get_mut(&inv.ino.unwrap()).unwrap();
                fa.nlink -= 1;
                if fa.nlink == 0 {
                    ic.remove(&inv.ino.unwrap());
                    #[cfg(feature = "check-data")]
                    {
                        let fc = &mut fs_data.INV_FILE_CONTENTS;
                        fc.remove(&inv.ino.unwrap());
                    }
                    #[cfg(feature = "check-xattr")]
                    {
                        let xc = &mut fs_data.INV_XATTR_CONTENTS;
                        xc.remove(&inv.ino.unwrap());
                    }
                }
            }
            #[cfg(feature = "check-dirs")]
            {
                let dc = &mut fs_data.INV_DIR_CONTENTS;
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .remove(&inv.name);
            }
            fs_data.INV_INODE_PATHS.remove(&inv.child_path);
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::EACCES) => assert_eq!(
            inv.perm,
            Some(libc::EACCES),
            "Returned EACCES on path where we have permission"
        ),
        Err(libc::EPERM) => assert_eq!(
            inv.perm,
            Some(libc::EPERM),
            "Returned EPERM on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.child_exists, "Returned ENOENT on extant directory"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
