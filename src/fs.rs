use std::{
    collections::BTreeMap,
    ffi::CString,
    mem::MaybeUninit,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{inode_mapper::InodeMapper, log_more, logging::CallID};
use fuser::Filesystem;
use libc::c_int;
use procfs::ProcResult;

const TTL: Duration = Duration::new(0, 0);

pub struct InvFS {
    pub(crate) root: PathBuf,
    paths: InodeMapper,
    dir_fhs: BTreeMap<u64, *mut libc::DIR>,
}

impl InvFS {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            paths: InodeMapper::new(),
            dir_fhs: BTreeMap::new(),
        }
    }
}

pub mod access;
pub mod bmap;
pub mod copy_file_range;
pub mod create;
pub mod destroy;
pub mod fallocate;
pub mod flush;
pub mod forget;
pub mod fsync;
pub mod fsyncdir;
pub mod getattr;
pub mod getlk;
pub mod getxattr;
pub mod init;
pub mod ioctl;
pub mod link;
pub mod listxattr;
pub mod lookup;
pub mod lseek;
pub mod mkdir;
pub mod mknod;
pub mod open;
pub mod opendir;
pub mod read;
pub mod readdir;
pub mod readdirplus;
pub mod readlink;
pub mod release;
pub mod releasedir;
pub mod removexattr;
pub mod rename;
pub mod rmdir;
pub mod setattr;
pub mod setlk;
pub mod setxattr;
pub mod statfs;
pub mod symlink;
pub mod unlink;
pub mod write;

impl Filesystem for InvFS {
    fn init(
        &mut self,
        req: &fuser::Request<'_>,
        config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        self.do_init(req, config)
    }

    fn destroy(&mut self) {
        self.do_destroy()
    }

    fn lookup(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        self.do_lookup(req, parent, name, reply)
    }

    fn forget(&mut self, req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        self.do_forget(req, ino, nlookup)
    }

    fn getattr(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        self.do_getattr(req, ino, reply)
    }

    fn setattr(
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
        self.do_setattr(
            req, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime,
            flags, reply,
        )
    }

    fn readlink(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        self.do_readlink(req, ino, reply)
    }

    fn mknod(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        self.do_mknod(req, parent, name, mode, umask, rdev, reply)
    }

    fn mkdir(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        self.do_mkdir(req, parent, name, mode, umask, reply)
    }

    fn unlink(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_unlink(req, parent, name, reply)
    }

    fn rmdir(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_rmdir(req, parent, name, reply)
    }

    fn symlink(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        link: &std::path::Path,
        reply: fuser::ReplyEntry,
    ) {
        self.do_symlink(req, parent, name, link, reply)
    }

    fn rename(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        newparent: u64,
        newname: &std::ffi::OsStr,
        flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_rename(req, parent, name, newparent, newname, flags, reply)
    }

    fn link(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        newparent: u64,
        newname: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        self.do_link(req, ino, newparent, newname, reply)
    }

    fn open(&mut self, req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        self.do_open(req, ino, flags, reply)
    }

    fn read(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        self.do_read(req, ino, fh, offset, size, flags, lock_owner, reply)
    }

    fn write(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        self.do_write(
            req,
            ino,
            fh,
            offset,
            data,
            write_flags,
            flags,
            lock_owner,
            reply,
        )
    }

    fn flush(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_flush(req, ino, fh, lock_owner, reply)
    }

    fn release(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        lock_owner: Option<u64>,
        flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_release(req, ino, fh, flags, lock_owner, flush, reply)
    }

    fn fsync(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_fsync(req, ino, fh, datasync, reply)
    }

    fn opendir(&mut self, req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        self.do_opendir(req, ino, flags, reply)
    }

    fn readdir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: fuser::ReplyDirectory,
    ) {
        self.do_readdir(req, ino, fh, offset, reply)
    }

    fn readdirplus(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        self.do_readdirplus(req, ino, fh, offset, reply)
    }

    fn releasedir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_releasedir(req, ino, fh, flags, reply)
    }

    fn fsyncdir(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_fsyncdir(req, ino, fh, datasync, reply)
    }

    fn statfs(&mut self, req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        self.do_statfs(req, ino, reply)
    }

    fn setxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        value: &[u8],
        flags: i32,
        position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_setxattr(req, ino, name, value, flags, position, reply)
    }

    fn getxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        self.do_getxattr(req, ino, name, size, reply)
    }

    fn listxattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        size: u32,
        reply: fuser::ReplyXattr,
    ) {
        self.do_listxattr(req, ino, size, reply)
    }

    fn removexattr(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_removexattr(req, ino, name, reply)
    }

    fn access(&mut self, req: &fuser::Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        self.do_access(req, ino, mask, reply)
    }

    fn create(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        self.do_create(req, parent, name, mode, umask, flags, reply)
    }

    fn getlk(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        reply: fuser::ReplyLock,
    ) {
        self.do_getlk(req, ino, fh, lock_owner, start, end, typ, pid, reply)
    }

    fn setlk(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        lock_owner: u64,
        start: u64,
        end: u64,
        typ: i32,
        pid: u32,
        sleep: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_setlk(req, ino, fh, lock_owner, start, end, typ, pid, sleep, reply)
    }

    fn bmap(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        blocksize: u32,
        idx: u64,
        reply: fuser::ReplyBmap,
    ) {
        self.do_bmap(req, ino, blocksize, idx, reply)
    }

    fn ioctl(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        flags: u32,
        cmd: u32,
        in_data: &[u8],
        out_size: u32,
        reply: fuser::ReplyIoctl,
    ) {
        self.do_ioctl(req, ino, fh, flags, cmd, in_data, out_size, reply)
    }

    fn fallocate(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        length: i64,
        mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        self.do_fallocate(req, ino, fh, offset, length, mode, reply)
    }

    fn lseek(
        &mut self,
        req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        self.do_lseek(req, ino, fh, offset, whence, reply)
    }

    fn copy_file_range(
        &mut self,
        req: &fuser::Request<'_>,
        ino_in: u64,
        fh_in: u64,
        offset_in: i64,
        ino_out: u64,
        fh_out: u64,
        offset_out: i64,
        len: u64,
        flags: u32,
        reply: fuser::ReplyWrite,
    ) {
        self.do_copy_file_range(
            req, ino_in, fh_in, offset_in, ino_out, fh_out, offset_out, len, flags, reply,
        )
    }
}

#[derive(Debug)]
struct Ids {
    uid: u32,
    gid: u32,
    gids: Vec<u32>,
}

pub fn get_groups(pid: i32) -> ProcResult<Vec<i32>> {
    Ok(procfs::process::Process::new(pid)?.status()?.groups)
}

pub fn chdirin(root: &Path) -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();
    cwd
}

pub fn chdirout(prev: PathBuf) {
    std::env::set_current_dir(prev).unwrap();
}

fn set_ids(callid: CallID, req: &fuser::Request<'_>) -> Ids {
    let gids = get_groups(req.pid().try_into().unwrap()).unwrap_or(vec![]);
    log_more!(
        callid,
        "REQ: uid={},gid={},gids={:?}",
        req.uid(),
        req.gid(),
        gids
    );
    let orig = unsafe {
        let uid = libc::geteuid();
        let gid = libc::getegid();
        let mut gids = [libc::gid_t::MIN; 256];
        let ngroups = libc::getgroups(256, gids.as_mut_ptr() as *mut u32);
        assert_ne!(ngroups, -1, "getgroups failed");
        Ids {
            uid,
            gid,
            gids: Vec::from(&gids[..ngroups.try_into().unwrap()]),
        }
    };
    unsafe {
        let rc = libc::setgroups(
            gids.len(),
            if gids.is_empty() {
                std::ptr::null()
            } else {
                &gids[0] as *const i32 as *const u32
            },
        );
        if rc != 0 {
            panic!("{}", *libc::__errno_location());
        }
        let rc = libc::setegid(req.gid());
        if rc != 0 {
            panic!("{}", *libc::__errno_location());
        }
        let rc = libc::seteuid(req.uid());
        if rc != 0 {
            panic!("{}", *libc::__errno_location());
        }
    }
    orig
}
fn restore_ids(ids: Ids) {
    unsafe {
        assert_eq!(libc::seteuid(ids.uid), 0, "seteuid failed");
        assert_eq!(libc::geteuid(), ids.uid, "failed to restore euid");
        assert_eq!(libc::setegid(ids.gid), 0, "setegid failed");
        assert_eq!(libc::getegid(), ids.gid, "failed to restore egid");
        assert_eq!(
            libc::setgroups(
                ids.gids.len(),
                if ids.gids.is_empty() {
                    std::ptr::null()
                } else {
                    &ids.gids[0] as *const u32
                }
            ),
            0,
            "setgroups failed"
        );
    }
}

unsafe fn stat_path(tgt_path: &Path) -> Result<libc::stat, i32> {
    let tgt = CString::new(tgt_path.as_os_str().as_bytes()).unwrap();
    let mut buf = MaybeUninit::zeroed().assume_init();
    let res = libc::lstat(tgt.as_ptr(), &mut buf);
    if res == 0 {
        Ok(buf)
    } else {
        Err(*libc::__errno_location())
    }
}
