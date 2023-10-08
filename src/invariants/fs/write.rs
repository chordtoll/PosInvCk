use std::{cmp::max, path::Path, sync::MutexGuard};

use crate::{
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
        FSData,
    },
    log_more,
    logging::CallID,
    req_rep::Request,
};

#[derive(Debug)]
#[must_use]
pub struct WriteInv {
    exists: bool,
    perm: Option<i32>,
    ino: u64,
    offset: usize,
    data: Vec<u8>,
}

pub fn inv_write_before(
    callid: CallID,
    req: &Request,
    base: &Path,
    ino: u64,
    _fh: u64,
    offset: i64,
    data: &[u8],
    _write_flags: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
    fs_data: &mut MutexGuard<'_, FSData>,
) -> WriteInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino, fs_data);

    let perm = check_perm(
        req.uid(),
        req.gid(),
        req.pid(),
        &inode_path,
        base,
        Access::Lookup,
    );

    WriteInv {
        ino,
        offset: offset.try_into().unwrap(),
        data: data.to_vec(),
        exists,
        perm,
    }
}
pub fn inv_write_after(
    callid: CallID,
    inv: WriteInv,
    res: &Result<isize, i32>,
    fs_data: &mut MutexGuard<'_, FSData>,
) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                inv.perm.is_none(),
                "Failed to return error on permission denied"
            );
            assert!(inv.exists, "Failed to return ENOENT on nonexistant file");
            assert_eq!(inv.data.len(), usize::try_from(*v).unwrap());
            #[cfg(feature = "check-meta")]
            {
                let ic = &mut fs_data.INV_INODE_CONTENTS;
                let fa = ic.get_mut(&inv.ino).expect("File missing inode");
                fa.size = max(fa.size, (inv.offset + inv.data.len()).try_into().unwrap());
                //fa.blocks = ((fa.size + (u64::from(fa.blksize) - 1)) / u64::from(fa.blksize)) * (u64::from(fa.blksize) / 512);
            }
            #[cfg(feature = "check-data")]
            {
                let fc = &mut fs_data.INV_FILE_CONTENTS;
                let fd = fc.get_mut(&inv.ino).expect("File missing contents");
                if inv.offset + inv.data.len() > fd.len() {
                    fd.resize(inv.offset + inv.data.len(), 0);
                }
                fd[inv.offset..inv.offset + inv.data.len()].copy_from_slice(&inv.data);
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
        Err(libc::ENOENT) => assert!(!inv.exists, "Returned ENOENT on extant file"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
    /*let mut fc = INV_FILE_CONTENTS.lock().unwrap();
    let fce = fc.get_mut(&inv.ino).unwrap();
    if inv.offset + inv.data.len() > fce.len() {
        fce.resize(inv.offset + inv.data.len(), 0);
    }
    fce[inv.offset..inv.offset + inv.data.len()].copy_from_slice(&inv.data);
    let mut ic = INV_INODE_CONTENTS.lock().unwrap();
    let atr = inv.child_path.metadata().unwrap();
    ic.insert(atr.st_ino(), atr.into());

    todo!();*/
}
