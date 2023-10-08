use std::{os::linux::fs::MetadataExt, path::Path, sync::MutexGuard};

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::FileAttr,
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, sgids, Access},
        FSData,
    },
    log_more,
    logging::CallID, req_rep::Request,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct SetattrArgs {
    ino: u64,
    mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
    size: Option<u64>,
    atime: Option<fuser::TimeOrNow>,
    mtime: Option<fuser::TimeOrNow>,
    ctime: Option<std::time::SystemTime>,
    fh: Option<u64>,
    crtime: Option<std::time::SystemTime>,
    chgtime: Option<std::time::SystemTime>,
    bkuptime: Option<std::time::SystemTime>,
    flags: Option<u32>,
}

#[derive(Debug)]
#[must_use]
pub struct SetattrInv {
    exists: bool,
    perm: Option<i32>,
    args: SetattrArgs,
    clear_setgid: bool,
}

pub fn inv_setattr_before(
    callid: CallID,
    req: &Request,
    base: &Path,
    ino: u64,
    mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
    size: Option<u64>,
    atime: Option<fuser::TimeOrNow>,
    mtime: Option<fuser::TimeOrNow>,
    ctime: Option<std::time::SystemTime>,
    fh: Option<u64>,
    crtime: Option<std::time::SystemTime>,
    chgtime: Option<std::time::SystemTime>,
    bkuptime: Option<std::time::SystemTime>,
    flags: Option<u32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> SetattrInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino, fs_data);

    let mut perm = None;
    if let Some(uid) = uid {
        if perm.is_none() {
            perm = check_perm(
                req.uid(),
                req.gid(),
                req.pid(),
                &inode_path,
                base,
                Access::Chown(uid),
            );
        }
    }
    if let Some(gid) = gid {
        if perm.is_none() {
            perm = check_perm(
                req.uid(),
                req.gid(),
                req.pid(),
                &inode_path,
                base,
                Access::Chgrp(gid),
            );
        }
    }
    if (mode.is_some()) && perm.is_none() {
        perm = check_perm(
            req.uid(),
            req.gid(),
            req.pid(),
            &inode_path,
            base,
            Access::Chmod,
        );
    }
    if (size.is_some()) && perm.is_none() {
        perm = check_perm(
            req.uid(),
            req.gid(),
            req.pid(),
            &inode_path,
            base,
            Access::Write,
        );
    }

    let sgids = sgids(req.pid());
    let mut clear_setgid = true;
    if req.uid() == 0 {
        clear_setgid = false;
    }
    if req.gid() == inode_path.symlink_metadata().unwrap().st_gid() {
        clear_setgid = false;
    }
    if sgids.contains(&inode_path.symlink_metadata().unwrap().st_gid()) {
        clear_setgid = false;
    }

    SetattrInv {
        exists,
        perm,
        clear_setgid,
        args: SetattrArgs {
            ino,
            mode,
            uid,
            gid,
            size,
            atime,
            mtime,
            ctime,
            fh,
            crtime,
            chgtime,
            bkuptime,
            flags,
        },
    }
}
pub fn inv_setattr_after(
    callid: CallID,
    inv: SetattrInv,
    res: &Result<fuser::FileAttr, i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(inv.exists, "Failed to return ENOENT on nonexistant parent");
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                let mut fa = ic.get(&inv.args.ino).expect("Inode does not exist").clone();
                fa.ctime = v.ctime;
                if let Some(v) = inv.args.mode {
                    fa.perm = (v & 0o7777).try_into().unwrap();
                    if inv.clear_setgid {
                        fa.perm &= !0o2000;
                    }
                }
                if let Some(v) = inv.args.size {
                    fa.size = v;
                    #[cfg(feature = "check-data")]
                    {
                        let fc = &mut fs_data.INV_FILE_CONTENTS;
                        let fd = fc.get_mut(&inv.args.ino).expect("Contents do not exist");
                        fd.resize(v.try_into().unwrap(), 0);
                    }
                }
                if let Some(v) = inv.args.uid {
                    fa.uid = v;
                }
                if let Some(v) = inv.args.gid {
                    fa.gid = v;
                }
                println!("{:o} : {:o}", FileAttr::from(v).perm, fa.perm);
                assert_eq_pretty!(FileAttr::from(v).reset_times(), fa.reset_times());
                fs_data.INV_INODE_CONTENTS.insert(inv.args.ino, fa);
            }
        }
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
        Err(libc::EFBIG) => println!("FBIG"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
