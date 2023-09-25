use std::{
    ffi::CString,
    mem::MaybeUninit,
    os::unix::prelude::OsStrExt,
    time::{SystemTime, UNIX_EPOCH},
};

use fuser::TimeOrNow;
use libc::timeval;

use crate::{
    fs::{chdirin, chdirout, restore_ids, set_ids, stat_path, TTL},
    fs_to_fuse::FsToFuseAttr,
    invariants::fs::setattr::{inv_setattr_after, inv_setattr_before},
    log_call, log_more, log_res,
};

use super::InvFS;

impl InvFS {
    pub fn do_setattr(
        &mut self,
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
        reply: fuser::ReplyAttr,
    ) {
        let callid = log_call!("SETATTR", "ino={}", ino);
        let cwd = chdirin(&self.root);
        let mut dl = self.data.lock().unwrap();
        let inv = inv_setattr_before(
            callid, req, &self.root, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime,
            chgtime, bkuptime, flags, &mut dl,
        );
        let ids = set_ids(callid, req, None);
        let ip = &dl.INODE_PATHS;
        let path = ip.get(ino);
        log_more!(callid, "path={:?}", path);
        let res = (|| unsafe {
            let tgt = CString::new(path.as_os_str().as_bytes()).unwrap();
            if let Some(v) = mode {
                log_more!(callid, "mode={:o} ({})", v, v);
                if libc::chmod(tgt.as_ptr(), v) != 0 {
                    return Err(*libc::__errno_location());
                }
            }
            if let Some(v) = uid {
                log_more!(callid, "uid={}", v);
                if libc::lchown(tgt.as_ptr(), v, u32::MAX) != 0 {
                    return Err(*libc::__errno_location());
                }
            }
            if let Some(v) = gid {
                log_more!(callid, "gid={}", v);
                if libc::lchown(tgt.as_ptr(), u32::MAX, v) != 0 {
                    return Err(*libc::__errno_location());
                }
            }
            if let Some(v) = size {
                log_more!(callid, "size={}", v);
                if libc::truncate(tgt.as_ptr(), v.try_into().unwrap()) != 0 {
                    return Err(*libc::__errno_location());
                }
            }
            match (atime, mtime) {
                (Some(a), Some(m)) => {
                    log_more!(callid, "atime={:?},mtime={:?}", a, m);
                    let a = if let TimeOrNow::SpecificTime(t) = a {
                        t
                    } else {
                        SystemTime::now()
                    };
                    let m = if let TimeOrNow::SpecificTime(t) = m {
                        t
                    } else {
                        SystemTime::now()
                    };
                    let a_s = a.duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let a_u = a.duration_since(UNIX_EPOCH).unwrap().as_micros() % 1_000_000;
                    let m_s = m.duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let m_u = m.duration_since(UNIX_EPOCH).unwrap().as_micros() % 1_000_000;
                    let times = [
                        timeval {
                            tv_sec: a_s.try_into().unwrap(),
                            tv_usec: a_u.try_into().unwrap(),
                        },
                        timeval {
                            tv_sec: m_s.try_into().unwrap(),
                            tv_usec: m_u.try_into().unwrap(),
                        },
                    ];
                    if libc::utimes(tgt.as_ptr(), times.as_ptr()) != 0 {
                        return Err(*libc::__errno_location());
                    }
                }
                (Some(a), None) => {
                    let a = if let TimeOrNow::SpecificTime(t) = a {
                        t
                    } else {
                        SystemTime::now()
                    };
                    let a_s = a.duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let a_u = a.duration_since(UNIX_EPOCH).unwrap().as_micros() % 1_000_000;

                    let mut buf = MaybeUninit::zeroed().assume_init();
                    libc::stat(tgt.as_ptr(), &mut buf);
                    let m_s = buf.st_mtime;
                    let m_u = buf.st_mtime_nsec / 1000;

                    let times = [
                        timeval {
                            tv_sec: a_s.try_into().unwrap(),
                            tv_usec: a_u.try_into().unwrap(),
                        },
                        timeval {
                            tv_sec: m_s,
                            tv_usec: m_u,
                        },
                    ];
                    if libc::utimes(tgt.as_ptr(), times.as_ptr()) != 0 {
                        return Err(*libc::__errno_location());
                    }
                }
                (None, Some(m)) => {
                    let mut buf = MaybeUninit::zeroed().assume_init();
                    libc::stat(tgt.as_ptr(), &mut buf);
                    let a_s = buf.st_atime;
                    let a_u = buf.st_atime_nsec / 1000;

                    let m = if let TimeOrNow::SpecificTime(t) = m {
                        t
                    } else {
                        SystemTime::now()
                    };
                    let m_s = m.duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let m_u = m.duration_since(UNIX_EPOCH).unwrap().as_micros() % 1_000_000;

                    let times = [
                        timeval {
                            tv_sec: a_s,
                            tv_usec: a_u,
                        },
                        timeval {
                            tv_sec: m_s.try_into().unwrap(),
                            tv_usec: m_u.try_into().unwrap(),
                        },
                    ];
                    if libc::utimes(tgt.as_ptr(), times.as_ptr()) != 0 {
                        return Err(*libc::__errno_location());
                    }
                }
                (None, None) => {}
            }
            if let Some(v) = ctime {
                log_more!(callid, "ctime={:?}", v);
                todo!("SETATTR ctime");
            }
            if let Some(v) = fh {
                log_more!(callid, "fh={}", v);
            }
            if let Some(v) = crtime {
                log_more!(callid, "crtime={:?}", v);
                todo!("SETATTR crtime");
            }
            if let Some(v) = chgtime {
                log_more!(callid, "chgtime={:?}", v);
                todo!("SETATTR chgtime");
            }
            if let Some(v) = bkuptime {
                log_more!(callid, "bkuptime={:?}", v);
                todo!("SETATTR bkuptime");
            }
            if let Some(v) = flags {
                log_more!(callid, "flags={}", v);
                todo!("SETATTR flags");
            }
            stat_path(path).map(|x| x.to_fuse_attr(ino))
        })();

        log_res!(callid, "{:?}", res);

        restore_ids(ids);
        inv_setattr_after(callid, inv, &res, &mut dl);
        chdirout(cwd);
        match res {
            Ok(v) => reply.attr(&TTL, &v),
            Err(v) => reply.error(v),
        }
    }
}
