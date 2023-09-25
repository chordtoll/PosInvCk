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
};

#[derive(Debug)]
#[must_use]
pub struct RenameInv {
    new_parent: u64,
    new_name: OsString,
    new_parent_exists: bool,
    new_child_exists: bool,
    new_perm: Option<i32>,
    new_toolong: bool,
    new_child_path: PathBuf,
    new_ino: Option<u64>,
    new_notempty: bool,
    old_parent: u64,
    old_name: OsString,
    old_child_exists: bool,
    old_perm: Option<i32>,
    old_toolong: bool,
    old_child_path: PathBuf,
    old_ino: Option<u64>,
}

pub fn inv_rename_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    base: &Path,
    parent: u64,
    name: &std::ffi::OsStr,
    newparent: u64,
    newname: &std::ffi::OsStr,
    _flags: u32,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> RenameInv {
    let CPPN {
        child_path: old_child_path,
        child_exists: old_child_exists,
        toolong: old_toolong,
        ino: old_ino,
        ..
    } = common_pre_parent_name(parent, name, fs_data);

    let CPPN {
        child_path: new_child_path,
        parent_exists: new_parent_exists,
        child_exists: new_child_exists,
        toolong: new_toolong,
        ino: new_ino,
        ..
    } = common_pre_parent_name(newparent, newname, fs_data);

    let old_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &old_child_path,
        base,
        Access::Delete,
    );
    let new_perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &new_child_path,
        base,
        Access::Create,
    );

    let new_notempty = new_child_exists
        && new_child_path.symlink_metadata().unwrap().is_dir()
        && new_child_path.read_dir().unwrap().count() != 0;

    RenameInv {
        new_parent: newparent,
        new_name: newname.to_os_string(),
        new_parent_exists,
        new_child_exists,
        new_perm,
        new_toolong,
        new_child_path,
        new_ino,
        new_notempty,
        old_parent: parent,
        old_name: name.to_os_string(),
        old_child_exists,
        old_perm,
        old_toolong,
        old_child_path,
        old_ino,
    }
}
pub fn inv_rename_after(
    callid: CallID,
    inv: RenameInv,
    res: &Result<(), i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(()) => {
            assert!(
                !inv.old_toolong && !inv.new_toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(
                inv.old_perm.is_none() && inv.new_perm.is_none(),
                "Returned OK when error expected ({:?}) -> ({:?})",
                inv.old_perm,
                inv.new_perm
            );
            assert!(
                inv.old_child_exists,
                "Failed to return ENOENT on nonexistant child"
            );
            assert!(
                inv.new_parent_exists,
                "Failed to return ENOENT on nonexistant parent"
            );
            assert!(
                !inv.new_notempty,
                "Failed to return ENOTEMPTY on nonempty new dir"
            );
            #[cfg(feature = "check-dirs")]
            {
                let dc = &mut fs_data.INV_DIR_CONTENTS;
                let ino = dc
                    .get_mut(&inv.old_parent)
                    .expect("Parent does not exist")
                    .remove(&inv.old_name)
                    .expect("No old dir entry to remove");
                dc.get_mut(&inv.new_parent)
                    .expect("Parent does not exist")
                    .insert(inv.new_name, ino);
            }
            let ic = &mut fs_data.INV_INODE_CONTENTS;
            let ino = ic
                .get(&inv.old_ino.unwrap())
                .expect("Overwriting dest, but no file to delete");
            let ik = ino.kind;
            if inv.new_child_exists {
                #[cfg(feature = "check-meta")]
                {
                    let ic = &mut fs_data.INV_INODE_CONTENTS;
                    let ino = ic
                        .get_mut(&inv.new_ino.unwrap())
                        .expect("Overwriting dest, but no file to delete");
                    println!("DEC N");
                    ino.nlink -= 1;
                    if ino.nlink == 0 {
                        ic.remove(&inv.new_ino.unwrap());
                        #[cfg(feature = "check-data")]
                        {
                            let fc = &mut fs_data.INV_FILE_CONTENTS;
                            fc.remove(&inv.new_ino.unwrap());
                        }
                        #[cfg(feature = "check-xattr")]
                        {
                            let xc = &mut fs_data.INV_XATTR_CONTENTS;
                            xc.remove(&inv.new_ino.unwrap());
                        }
                    }
                }
            }
            let ic = &mut fs_data.INV_INODE_CONTENTS;
            if ik == FileType::Directory {
                let old_parent_ino = ic
                    .get_mut(&inv.old_parent)
                    .expect("Can't get parent to decrement refcount");
                println!("DEC OP");
                old_parent_ino.nlink -= 1;
                if !inv.new_child_exists {
                    let new_parent_ino = ic
                        .get_mut(&inv.new_parent)
                        .expect("Can't get parent to increment refcount");
                    println!("INC NP");
                    new_parent_ino.nlink += 1;
                }
            }
            fs_data
                .INV_INODE_PATHS
                .rename(inv.old_child_path, inv.new_child_path);

            let opk = FileAttr::from(
                std::fs::symlink_metadata(fs_data.INODE_PATHS.get(inv.old_parent)).unwrap(),
            );
            let ope = fs_data.INV_INODE_CONTENTS.get(&inv.old_parent).unwrap();
            if ope.ino != 1 {
                assert_eq_pretty!(opk.reset_times(), ope.reset_times());
            }

            let npk = FileAttr::from(
                std::fs::symlink_metadata(fs_data.INODE_PATHS.get(inv.new_parent)).unwrap(),
            );
            let npe = fs_data.INV_INODE_CONTENTS.get(&inv.new_parent).unwrap();
            if npe.ino != 1 {
                assert_eq_pretty!(npk.reset_times(), npe.reset_times());
            }

            //let nk = FileAttr::from(std::fs::metadata(fs_data.INODE_PATHS.get(inv.new_ino.unwrap())).unwrap());
            //let ne = fs_data.INV_INODE_CONTENTS.get(&inv.new_ino.unwrap()).unwrap();
            //assert_eq_pretty!(nk.reset_times(),ne.reset_times());
        }
        Err(libc::ENOTEMPTY) => assert!(
            inv.new_notempty,
            "Returned ENOTEMPTY on nonexistant/nondir/empty new"
        ),
        Err(libc::ENAMETOOLONG) => assert!(
            inv.old_toolong || inv.new_toolong,
            "Returned ENAMETOOLONG on valid name"
        ),
        Err(libc::ENOENT) => assert!(
            !inv.old_child_exists || !inv.new_parent_exists,
            "Returned ENOENT on extant item"
        ),
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
