use std::{
    collections::BTreeMap,
    ffi::OsString,
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
pub struct MkdirInv {
    uid: u32,
    gid: u32,
    parent: u64,
    name: OsString,
    parent_exists: bool,
    perm: Option<i32>,
    toolong: bool,
    child_path: PathBuf,
    mode: u32,
}

pub fn inv_mkdir_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    parent: u64,
    name: &std::ffi::OsStr,
    mode: u32,
    _umask: u32,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> MkdirInv {
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

    MkdirInv {
        uid: req.uid(),
        gid: req.gid(),
        parent,
        name: name.to_owned(),
        parent_exists,
        toolong,
        perm,
        child_path,
        mode,
    }
}
pub fn inv_mkdir_after(
    callid: CallID,
    inv: MkdirInv,
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
                size: 0,
                //blocks: 8,
                atime: v.atime,
                mtime: v.mtime,
                ctime: v.ctime,
                crtime: v.crtime,
                kind: FileType::Directory,
                perm: (inv.mode & 0o7777).try_into().unwrap(),
                nlink: 2,
                uid: inv.uid,
                gid: inv.gid,
                rdev: 0,
                blksize: 4096,
                flags: 0,
            };
            assert_eq_pretty!(FileAttr::from(v), fa);
            #[cfg(feature = "check-meta")]
            {
                fs_data.INV_INODE_CONTENTS.insert(v.ino, fa);
            }
            #[cfg(feature = "check-dirs")]
            {
                fs_data.INV_DIR_CONTENTS.insert(v.ino, BTreeMap::new());
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
            #[cfg(feature = "check-meta")]
            {
                let dc = &mut fs_data.INV_INODE_CONTENTS;
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .nlink += 1;
            }
            println!("III {}->{:?}", v.ino, inv.child_path);
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
