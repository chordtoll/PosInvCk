use std::{
    ffi::OsString,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    sync::MutexGuard,
};

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::{FileAttr, FileType},
    invariants::{
        common::{common_pre_parent_name, CPPN},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct SymlinkInv {
    uid: u32,
    gid: u32,
    parent: u64,
    name: OsString,
    parent_exists: bool,
    perm: Option<i32>,
    toolong: bool,
    child_path: PathBuf,
    link: PathBuf,
}
pub fn inv_symlink_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    parent: u64,
    name: &std::ffi::OsStr,
    link: &std::path::Path,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> SymlinkInv {
    let CPPN {
        child_path,
        parent_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name, fs_data);

    let perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &child_path,
        base,
        Access::Create,
    );

    SymlinkInv {
        uid: req.uid(),
        gid: req.gid(),
        parent,
        name: name.to_owned(),
        parent_exists,
        toolong,
        perm,
        child_path,
        link: link.to_path_buf(),
    }
}
pub fn inv_symlink_after(
    callid: CallID,
    inv: SymlinkInv,
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
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(
                inv.parent_exists,
                "Failed to return ENOENT on nonexistant parent"
            );
            let fa = FileAttr {
                ino: v.ino,
                size: inv.link.as_os_str().len().try_into().unwrap(),
                //blocks: 0,
                atime: v.atime,
                mtime: v.mtime,
                ctime: v.ctime,
                crtime: v.crtime,
                kind: FileType::Symlink,
                perm: 0o777,
                nlink: 1,
                uid: inv.uid,
                gid: inv.gid,
                rdev: 0,
                blksize: 4096,
                flags: 0,
            };
            assert_eq_pretty!(FileAttr::from(v), fa);
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                ic.insert(v.ino, fa);
            }
            #[cfg(feature = "check-data")]
            {
                let fc = &mut fs_data.INV_FILE_CONTENTS;
                fc.insert(v.ino, inv.link.as_os_str().as_bytes().to_vec());
            }
            #[cfg(feature = "check-xattr")]
            {
                let xc = &mut fs_data.INV_XATTR_CONTENTS;
                xc.insert(v.ino, BTreeMap::new());
            }
            #[cfg(feature = "check-dirs")]
            {
                let dc = &mut fs_data.INV_DIR_CONTENTS;
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .insert(inv.name, v.ino);
            }
            fs_data.INV_INODE_PATHS.insert(v.ino, inv.child_path);
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
        Err(libc::ENOENT) => assert!(!inv.parent_exists, "Returned ENOENT on extant parent"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
