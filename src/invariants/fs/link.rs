use std::{ffi::OsString, path::PathBuf};

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::FileAttr,
    invariants::{
        common::{common_pre_ino, common_pre_parent_name, CPI, CPPN},
        perm::{check_perm, Access},
        INODE_PATHS,
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct LinkInv {
    parent: u64,
    name: OsString,
    old_perm: bool,
    new_perm: bool,
    toolong: bool,
    old_exists: bool,
    new_exists: bool,
    new_path: PathBuf,
}

pub fn inv_link_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    ino: u64,
    newparent: u64,
    newname: &std::ffi::OsStr,
) -> LinkInv {
    let CPI {
        inode_path: old_path,
        exists: old_exists,
    } = common_pre_ino(callid, ino);
    let CPPN {
        child_path: new_path,
        child_exists: new_exists,
        toolong,
        ..
    } = common_pre_parent_name(newparent, newname);

    let old_perm = check_perm(req.uid(), req.gid(), req.pid(), &old_path, Access::Lookup);
    let new_perm = check_perm(req.uid(), req.gid(), req.pid(), &new_path, Access::Lookup);

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
pub fn inv_link_after(callid: CallID, inv: LinkInv, res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(
                inv.old_perm && inv.new_perm,
                "Failed to return EACCES on permission denied"
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
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                let fa = ic.get_mut(&v.ino).expect("Inode does not exist");
                fa.mtime = v.mtime;
                fa.ctime = v.ctime;
                fa.nlink += 1;
                assert_eq_pretty!(*fa, FileAttr::from(v));
            }
            #[cfg(feature = "check-dirs")]
            {
                let mut dc = crate::invariants::DIR_CONTENTS.lock().unwrap();
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .insert(inv.name, v.ino);
            }
            let mut ip = INODE_PATHS.lock().unwrap();
            ip.insert(v.ino, inv.new_path);
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::EACCES) => assert!(
            !inv.old_perm || !inv.new_perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.old_exists, "Returned ENOENT on extant source"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
