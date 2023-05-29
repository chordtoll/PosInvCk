use std::ffi::OsString;

use asserteq_pretty::assert_eq_pretty;

use crate::{
    file_attr::FileAttr,
    invariants::{
        common::{common_pre_parent_name, CPPN},
        perm::{check_perm, Access},
    },
    log_more,
    logging::CallID,
};

#[derive(Debug)]
pub struct LookupArgs {
    parent: u64,
    name: OsString,
}

#[derive(Debug)]
#[must_use]
pub struct LookupInv {
    child_exists: bool,
    perm: bool,
    toolong: bool,
    ino: Option<u64>,
    args: LookupArgs,
}

pub fn inv_lookup_before(
    _callid: CallID,
    req: &fuser::Request<'_>,
    parent: u64,
    name: &std::ffi::OsStr,
) -> LookupInv {
    let CPPN {
        child_path,
        ino,
        child_exists,
        toolong,
        ..
    } = common_pre_parent_name(parent, name);

    let perm = check_perm(req.uid(), req.gid(), req.pid(), &child_path, Access::Lookup);

    LookupInv {
        ino,
        child_exists,
        toolong,
        perm,
        args: LookupArgs {
            parent,
            name: name.to_owned(),
        },
    }
}
pub fn inv_lookup_after(callid: CallID, inv: LookupInv, res: &Result<fuser::FileAttr, i32>) {
    log_more!(callid, "invariant={:?}", inv);
    match res {
        Ok(v) => {
            assert!(
                !inv.toolong,
                "Failed to return ENAMETOOLONG on name too long"
            );
            assert!(inv.perm, "Failed to return EACCES on permission denied");
            assert!(
                inv.child_exists,
                "Failed to return ENOENT on nonexistant child"
            );
            let ino = inv.ino.expect("Failed to get child inode");
            assert_eq!(v.ino, ino, "Returned inode number does not match");
            #[cfg(feature = "check-dirs")]
            assert_eq!(
                crate::invariants::DIR_CONTENTS
                    .lock()
                    .unwrap()
                    .get(&inv.args.parent)
                    .expect("Parent does not exist")
                    .get(&inv.args.name)
                    .expect("Child does not exist"),
                &ino
            );
            #[cfg(feature = "check-meta")]
            assert_eq_pretty!(
                crate::invariants::INODE_CONTENTS
                    .lock()
                    .unwrap()
                    .get(&ino)
                    .map(|x| x.reset_times()),
                Some(FileAttr::from(v).reset_times()),
                "Result did not match expected value"
            );
        }
        Err(libc::ENAMETOOLONG) => assert!(inv.toolong, "Returned ENAMETOOLONG on valid name"),
        Err(libc::EACCES) => assert!(
            !inv.perm,
            "Returned EACCESS on path where we have permission"
        ),
        Err(libc::ENOENT) => assert!(!inv.child_exists, "Returned ENOENT on extant path"),
        Err(e) => panic!("Got unexpected error code {}", e),
    }
}
