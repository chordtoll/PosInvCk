use super::InvFS;

impl InvFS {
    pub fn do_copy_file_range(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino_in: u64,
        _fh_in: u64,
        _offset_in: i64,
        _ino_out: u64,
        _fh_out: u64,
        _offset_out: i64,
        _len: u64,
        _flags: u32,
        _reply: fuser::ReplyWrite,
    ) {
        todo!("COPY_FILE_RANGE");
    }
}
