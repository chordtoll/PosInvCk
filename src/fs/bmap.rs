use super::InvFS;

impl InvFS {
    pub fn do_bmap(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _blocksize: u32,
        _idx: u64,
        _reply: fuser::ReplyBmap,
    ) {
        todo!("BMAP");
    }
}
