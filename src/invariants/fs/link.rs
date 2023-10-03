use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::MutexGuard,
};

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::FileAttr,
    invariants::{
        common::{common_pre_ino, common_pre_parent_name, CPI, CPPN},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID,
    req_rep::Request,
};

#[derive(Debug)]
#[must_use]
pub struct LinkInv {
    parent: u64,
    name: OsString,
    old_perm: Option<i32>,
    new_perm: Option<i32>,
    toolong: bool,
    old_exists: bool,
    new_exists: bool,
    new_path: PathBuf,
}

pub fn inv_link_before(
    callid: CallID,
    req: &Request,
    base: &Path,
    ino: u64,
    newparent: u64,
    newname: &std::ffi::OsStr,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> LinkInv {
    let CPI {
        inode_path: old_path,
        exists: old_exists,
    } = common_pre_ino(callid, ino, fs_data);
    let CPPN {
        child_path: new_path,
        child_exists: new_exists,
        toolong,
        ..
    } = common_pre_parent_name(newparent, newname, fs_data);

    let old_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &old_path,
        base,
        Access::Lookup,
    );
    let new_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &new_path,
        base,
        Access::Create,
    );

    LinkInv {
        parent: newparent,
        name: newname.to_owned(),
        toolong,
        old_perm,
        new_perm,
        old_exists,
        new_exists,
        new_path,
    }
}
pub fn inv_link_after(
    callid: CallID,
    inv: LinkInv,
    res: &Result<fuser::FileAttr, i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(
                inv.old_perm.is_none() && inv.new_perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(
                inv.old_exists,
                "Failed to return ENOENT on nonexistant source"
            );
            assert!(
                !inv.new_exists,
                "Failed to return EEXIST on existant target"
            );
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                let fa = ic.get_mut(&v.ino).expect("Inode does not exist");
                fa.mtime = v.mtime;
                fa.ctime = v.ctime;
                fa.nlink += 1;
                assert_eq_pretty!(*fa, FileAttr::from(v));
            }
            #[cfg(feature = "check-dirs")]
            {
                let dc = &mut fs_data.INV_DIR_CONTENTS;
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .insert(inv.name, v.ino);
            }
            fs_data.INV_INODE_PATHS.insert(v.ino, inv.new_path);
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::ENOENT) => assert!(!inv.old_exists, "Returned ENOENT on extant source"),
        Err(libc::EACCES) => assert!(
            inv.old_perm == Some(libc::EACCES) || inv.new_perm == Some(libc::EACCES),
            "Returned EACCES on path where we have permission"
        ),
        Err(libc::EPERM) => assert!(
            inv.old_perm == Some(libc::EPERM) || inv.new_perm == Some(libc::EPERM),
            "Returned EPERM on path where we have permission"
        ),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
