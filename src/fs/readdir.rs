use std::{
    ffi::{OsStr, OsString},
    os::unix::prelude::OsStrExt,
};

use fuser::FileType;

use crate::{
    fs::{restore_ids, set_ids},
    log_call, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_readdir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        let callid = log_call!("READDIR", "ino={},fh={:x},offset={:x}", ino, fh, offset);
        let ids = set_ids(callid, req);
        let dir = self.dir_fhs.get(&fh).unwrap();
        let res = unsafe {
            libc::seekdir(*dir, offset);
            *libc::__errno_location() = 0;
            let res = libc::readdir(*dir);
            if res == std::ptr::null_mut() {
                if *libc::__errno_location() == 0 {
                    Ok(None)
                } else {
                    Err(*libc::__errno_location())
                }
            } else {
                let name = core::ffi::CStr::from_ptr(&(*res).d_name as *const i8);
                let name = OsStr::from_bytes(name.to_bytes());
                let kind = match (*res).d_type {
                    libc::DT_REG => FileType::RegularFile,
                    libc::DT_DIR => FileType::Directory,
                    libc::DT_FIFO => FileType::NamedPipe,
                    libc::DT_BLK => FileType::BlockDevice,
                    libc::DT_CHR => FileType::CharDevice,
                    libc::DT_LNK => FileType::Symlink,
                    libc::DT_SOCK => FileType::Socket,
                    v => todo!("Readdir kind: {:?}", v),
                };
                Ok(Some(((*res).d_ino, (*res).d_off, kind, name)))
            }
        };
        log_res!(callid, "{:?}", res);
        restore_ids(ids);
        match res {
            Ok(Some((ino, offset, kind, name))) => {
                _ = reply.add(ino, offset, kind, OsString::from(name));
                reply.ok()
            }
            Ok(None) => reply.ok(),
            Err(v) => reply.error(v),
        }
    }
}
