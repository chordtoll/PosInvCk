use super::InvFS;

impl InvFS {
    pub fn do_fallocate(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _length: i64,
        _mode: i32,
        _reply: fuser::ReplyEmpty,
    ) {
        todo!("FALLOCATE");
    }
}
