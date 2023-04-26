use super::InvFS;

impl InvFS {
    pub fn do_lseek(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _whence: i32,
        _reply: fuser::ReplyLseek,
    ) {
        todo!("LSEEK should not be needed? Read has an offset?");
    }
}
