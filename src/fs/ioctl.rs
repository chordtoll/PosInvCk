use super::InvFS;

impl InvFS {
    pub fn do_ioctl(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: u32,
        _cmd: u32,
        _in_data: &[u8],
        _out_size: u32,
        _reply: fuser::ReplyIoctl,
    ) {
        todo!("IOCTL");
    }
}
