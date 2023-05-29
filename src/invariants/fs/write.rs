use std::cmp::max;

use crate::{
    invariants::{
        common::{common_pre_ino, CPI},
        perm::{check_perm, Access},
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
#[must_use]
pub struct WriteInv {
    exists: bool,
    perm: bool,
    ino: u64,
    offset: usize,
    data: Vec<u8>,
}

pub fn inv_write_before(
    callid: CallID,
    req: &fuser::Request<'_>,
    ino: u64,
    _fh: u64,
    offset: i64,
    data: &[u8],
    _write_flags: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
) -> WriteInv {
    let CPI { inode_path, exists } = common_pre_ino(callid, ino);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &inode_path, Access::Lookup);

    WriteInv {
        ino,
        offset: offset.try_into().unwrap(),
        data: data.to_vec(),
        exists,
        perm,
    }
}
pub fn inv_write_after(callid: CallID, inv: WriteInv, res: &Result<isize, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(inv.exists, "Failed to return ENOENT on nonexistant file");
            assert_eq!(inv.data.len(), (*v).try_into().unwrap());
            #[cfg(feature = "check-meta")]
            {
                let mut ic = crate::invariants::INODE_CONTENTS.lock().unwrap();
                let mut fa = ic.get_mut(&inv.ino).expect("File missing inode");
                fa.size = max(fa.size, (inv.offset + inv.data.len()).try_into().unwrap());
                //fa.blocks = ((fa.size + (u64::from(fa.blksize) - 1)) / u64::from(fa.blksize)) * (u64::from(fa.blksize) / 512);
            }
            #[cfg(feature = "check-data")]
            {
                let mut fc = crate::invariants::FILE_CONTENTS.lock().unwrap();
                let fd = fc.get_mut(&inv.ino).expect("File missing contents");
                if inv.offset + inv.data.len() > fd.len() {
                    fd.resize(inv.offset + inv.data.len(), 0);
                }
                fd[inv.offset..inv.offset + inv.data.len()].copy_from_slice(&inv.data);
            }
        }
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.exists, "Returned ENOENT on extant file"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
    /*let mut fc = FILE_CONTENTS.lock().unwrap();
    let fce = fc.get_mut(&inv.ino).unwrap();
    if inv.offset + inv.data.len() > fce.len() {
        fce.resize(inv.offset + inv.data.len(), 0);
    }
    fce[inv.offset..inv.offset + inv.data.len()].copy_from_slice(&inv.data);
    let mut ic = INODE_CONTENTS.lock().unwrap();
    let atr = inv.child_path.metadata().unwrap();
    ic.insert(atr.st_ino(), atr.into());

    todo!();*/
}
