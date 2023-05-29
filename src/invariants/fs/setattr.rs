use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::FileAttr,
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
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
    perm: bool,
    args: SetattrArgs,
}

pub fn inv_setattr_before(
    callid: CallID,
    req: &fuser::Request<'_>,
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
) -> SetattrInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &inode_path, Access::Lookup);

    SetattrInv {
        exists,
        perm,
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
pub fn inv_setattr_after(callid: CallID, inv: SetattrInv, res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(inv.exists, "Failed to return ENOENT on nonexistant parent");
            #[cfg(feature = "check-meta")]
            {
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                let mut fa = ic.get(&inv.args.ino).expect("Inode does not exist").clone();
                fa.ctime = v.ctime;
                if let Some(v) = inv.args.mode {
                    fa.perm = (v & 0o7777).try_into().unwrap();
                }
                if let Some(v) = inv.args.size {
                    fa.size = v;
                    #[cfg(feature = "check-data")]
                    {
                        let mut fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
                        let fd = fc.get_mut(&inv.args.ino).expect("Contents do not exist");
                        fd.resize(v.try_into().unwrap(), 0);
                    }
                }
                assert_eq_pretty!(FileAttr::from(v).reset_times(), fa.reset_times());
                ic.insert(inv.args.ino, fa);
            }
        }
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
