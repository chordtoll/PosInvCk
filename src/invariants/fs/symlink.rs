use std::{ffi::OsString, os::unix::prelude::OsStrExt, path::PathBuf};

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::{FileAttr, FileType},
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
pub struct SymlinkInv {
    uid: u32,
    gid: u32,
    parent: u64,
    name: OsString,
    parent_exists: bool,
    perm: bool,
    toolong: bool,
    child_path: PathBuf,
    link: PathBuf,
}
pub fn inv_symlink_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
    link: &std::path::Path,
) -> SymlinkInv {
    let CPPN {
        child_path,
        parent_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &child_path, Access::Lookup);

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
pub fn inv_symlink_after(callid: CallID, inv: SymlinkInv, res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(inv.perm, "Failed to return EACCES on permission denied");
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
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                ic.insert(v.ino, fa);
            }
            #[cfg(feature = "check-data")]
            {
                let mut fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
                fc.insert(v.ino, inv.link.as_os_str().as_bytes().to_vec());
            }
            #[cfg(feature = "check-xattr")]
            {
                let mut xc = crate::invariants::XATTR_CONTENTS.lock().unwrap();
                xc.insert(v.ino, BTreeMap::new());
            }
            #[cfg(feature = "check-dirs")]
            {
                let mut dc = crate::invariants::DIR_CONTENTS.lock().unwrap();
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .insert(inv.name, v.ino);
            }
            let mut ip = INODE_PATHS.lock().unwrap();
            ip.insert(v.ino, inv.child_path);
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.parent_exists, "Returned ENOENT on extant parent"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
