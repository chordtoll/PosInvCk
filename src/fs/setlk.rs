use super::InvFS;

impl InvFS {
    pub fn do_setlk(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        _sleep: bool,
        _reply: fuser::ReplyEmpty,
    ) {
        todo!("SETLK");
    }
}
