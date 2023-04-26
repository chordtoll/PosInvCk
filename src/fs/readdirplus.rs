use super::InvFS;

impl InvFS {
    pub fn do_readdirplus(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _reply: fuser::ReplyDirectoryPlus,
    ) {
        todo!("READDIRPLUS");
    }
}
