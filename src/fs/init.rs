use libc::c_int;

use crate::{
    invariants::fs::init::{inv_init_after, inv_init_before},
    log_call,
    req_rep::{KernelConfig, Request},
};

use super::InvFS;

impl InvFS {
    pub fn do_init(&mut self, req: Request, config: &KernelConfig) -> Result<(), c_int> {
        let callid = log_call!("INIT", "config={:?}", config);
        let inv = inv_init_before(callid, self, req, config);
        self.data
            .lock()
            .unwrap()
            .INODE_PATHS
            .insert(1, self.root.clone());
        let res = Ok(());
        inv_init_after(callid, inv, &res, &mut self.data.lock().unwrap());
        res
    }
}

#[cfg(test)]
mod test {
    use maplit::{btreemap, btreeset};

    use crate::req_rep::{KernelConfig, Request};

    #[test]
    fn test_init() {
        let mut ifs = crate::test::create_ifs();
        ifs.do_init(
            Request {
                uid: 0,
                gid: 0,
                pid: 0,
            },
            &KernelConfig::empty(),
        )
        .unwrap();
        let ips = ifs.data.lock().unwrap().INODE_PATHS.store();
        assert_eq!(ips, btreemap! {1 => btreeset!{ifs.root}})
    }
}
