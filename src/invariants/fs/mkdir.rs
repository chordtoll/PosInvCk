use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

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
pub struct MkdirInv {
    uid: u32,
    gid: u32,
    parent: u64,
    name: OsString,
    parent_exists: bool,
    perm: bool,
    toolong: bool,
    child_path: PathBuf,
    mode: u32,
}

pub fn inv_mkdir_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
    mode: u32,
    _umask: u32,
) -> MkdirInv {
    let CPPN {
        child_path,
        parent_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &child_path, Access::Create);

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
pub fn inv_mkdir_after(callid: CallID, inv: MkdirInv, res: &Result<fuser::FileAttr, i32>) {
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
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                ic.insert(v.ino, fa);
            }
            #[cfg(feature = "check-dirs")]
            {
                let mut fc = crate::invariants::DIR_CONTENTS.lock().unwrap();
                fc.insert(v.ino, BTreeMap::new());
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
            #[cfg(feature = "check-meta")]
            {
                let mut dc = crate::invariants::INODE_CONTENTS.lock().unwrap();
                dc.get_mut(&inv.parent)
                    .expect("Parent does not exist")
                    .nlink += 1;
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
