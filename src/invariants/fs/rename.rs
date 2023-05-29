use std::{ffi::OsString, path::PathBuf};

use crate::{
    file_attr::FileType,
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
pub struct RenameInv {
    new_parent: u64,
    new_name: OsString,
    new_parent_exists: bool,
    new_child_exists: bool,
    new_perm: bool,
    new_toolong: bool,
    new_child_path: PathBuf,
    new_ino: Option<u64>,
    old_parent: u64,
    old_name: OsString,
    old_child_exists: bool,
    old_perm: bool,
    old_toolong: bool,
    old_child_path: PathBuf,
}

pub fn inv_rename_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
    newparent: u64,
    newname: &std::ffi::OsStr,
    _flags: u32,
) -> RenameInv {
    let CPPN {
        child_path: old_child_path,
        child_exists: old_child_exists,
        toolong: old_toolong,
        ..
    } = common_pre_parent_name(parent, name);

    let CPPN {
        child_path: new_child_path,
        parent_exists: new_parent_exists,
        child_exists: new_child_exists,
        toolong: new_toolong,
        ino: new_ino,
        ..
    } = common_pre_parent_name(newparent, newname);

    let old_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &old_child_path,
        Access::Lookup,
    );
    let new_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &new_child_path,
        Access::Lookup,
    );

    RenameInv {
        new_parent: newparent,
        new_name: newname.to_os_string(),
        new_parent_exists,
        new_child_exists,
        new_perm,
        new_toolong,
        new_child_path,
        new_ino,
        old_parent: parent,
        old_name: name.to_os_string(),
        old_child_exists,
        old_perm,
        old_toolong,
        old_child_path,
    }
}
pub fn inv_rename_after(callid: CallID, inv: RenameInv, res: &Result<(), i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(()) => {
            assert!(
                !inv.old_toolong && !inv.new_toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(
                inv.old_perm && inv.new_perm,
                "Failed to return EACCES on permission denied"
            );
            assert!(
                inv.old_child_exists,
                "Failed to return ENOENT on nonexistant child"
            );
            assert!(
                inv.new_parent_exists,
                "Failed to return ENOENT on nonexistant parent"
            );
            #[cfg(feature = "check-dirs")]
            {
                let mut dc = crate::invariants::DIR_CONTENTS.lock().unwrap();
                let ino = dc
                    .get_mut(&inv.old_parent)
                    .expect("Parent does not exist")
                    .remove(&inv.old_name)
                    .expect("No old dir entry to remove");
                dc.get_mut(&inv.new_parent)
                    .expect("Parent does not exist")
                    .insert(inv.new_name, ino);
            }
            if inv.new_child_exists {
                #[cfg(feature = "check-meta")]
                {
                    let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                    let ino = ic
                        .get_mut(&inv.new_ino.unwrap())
                        .expect("Overwriting dest, but no file to delete");
                    assert!(ino.kind == FileType::RegularFile);
                    ino.nlink -= 1;
                    if ino.nlink == 0 {
                        ic.remove(&inv.new_ino.unwrap());
                        #[cfg(feature = "check-data")]
                        {
                            let mut fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
                            fc.remove(&inv.new_ino.unwrap());
                        }
                        #[cfg(feature = "check-xattr")]
                        {
                            let mut xc = crate::invariants::XATTR_CONTENTS.lock().unwrap();
                            xc.remove(&inv.new_ino.unwrap());
                        }
                    }
                }
            }
            let mut ip = INODE_PATHS.lock().unwrap();
            ip.rename(inv.old_child_path, inv.new_child_path);
        }
        Err(libc::ENAMETOOLONG) => assert!(
            inv.old_toolong || inv.new_toolong,
            "Returned ENAMETOOLONG on valid name"
        ),
        Err(libc::EACCES) => assert!(
            !inv.old_perm || inv.new_perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(
            !inv.old_child_exists || !inv.new_parent_exists,
            "Returned ENOENT on extant item"
        ),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
