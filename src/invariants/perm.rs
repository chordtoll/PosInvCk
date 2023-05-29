use std::{collections::BTreeSet, os::linux::fs::MetadataExt, path::Path};

use crate::fs::get_groups;

pub enum Access {
    Lookup,
    Create,
}

pub fn check_perm(uid: u32, gid: u32, pid: u32, mut path: &Path, access: Access) -> bool {
    let sgids = BTreeSet::from_iter(get_groups(pid.try_into().unwrap()).unwrap_or(vec![]));
    loop {
        path = match path.parent() {
            Some(v) => v,
            None => break,
        };
        let meta = path
            .metadata()
            .unwrap_or_else(|_| panic!("Failed to get metadata at {:?}", path));
        if meta.st_uid() == uid {
            if meta.st_mode() & 0o100 == 0 {
                return false;
            }
        } else if meta.st_gid() == gid || sgids.contains(&meta.st_gid().try_into().unwrap()) {
            if meta.st_mode() & 0o010 == 0 {
                return false;
            }
        } else {
            if meta.st_mode() & 0o001 == 0 {
                return false;
            }
        }
    }
    true
}
