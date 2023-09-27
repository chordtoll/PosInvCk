use std::{
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
    req_rep::Request,
};

#[derive(Debug)]
#[must_use]
pub struct CreateInv {
    uid: u32,
    gid: u32,
    parent: u64,
    name: OsString,
    parent_exists: bool,
    perm: Option<i32>,
    toolong: bool,
    mode: u32,
    child_path: PathBuf,
}

pub fn inv_create_before(
    _callid: CallID,
    req: &Request,
    base: &Path,
    parent: u64,
    name: &std::ffi::OsStr,
    mode: u32,
    _umask: u32,
    _flags: i32,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> CreateInv {
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

    CreateInv {
        uid: req.uid(),
        gid: req.gid(),
        parent,
        name: name.to_owned(),
        parent_exists,
        toolong,
        perm,
        mode,
        child_path,
    }
}
pub fn inv_create_after(
    callid: CallID,
    inv: CreateInv,
    res: &Result<(fuser::FileAttr, i32), i32>,
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
                ino: v.0.ino,
                size: 0,
                //blocks: 0,
                atime: v.0.atime,
                mtime: v.0.mtime,
                ctime: v.0.ctime,
                crtime: v.0.crtime,
                kind: FileType::RegularFile,
                perm: (inv.mode & 0o7777).try_into().unwrap(),
                nlink: 1,
                uid: inv.uid,
                gid: inv.gid,
                rdev: 0,
                blksize: 4096,
                flags: 0,
            };
            println!("\t{:?}\n\t{:?}", FileAttr::from(v.0), fa);
            println!("{:o} : {:o}", FileAttr::from(v.0).perm, fa.perm);
            assert_eq_pretty!(FileAttr::from(v.0), fa);
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                ic.insert(v.0.ino, fa);
            }
            #[cfg(feature = "check-data")]
            {
                let fc = &mut fs_data.INV_FILE_CONTENTS;
                fc.insert(v.0.ino, Vec::new());
            }
            #[cfg(feature = "check-xattr")]
            {
                let xc = &mut fs_data.INV_XATTR_CONTENTS;
                xc.insert(v.0.ino, BTreeMap::new());
            }
            #[cfg(feature = "check-dirs")]
            {
                let dc = &mut fs_data.INV_DIR_CONTENTS;
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .insert(inv.name, v.0.ino);
            }
            fs_data.INV_INODE_PATHS.insert(v.0.ino, inv.child_path);
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
