use std::{collections::BTreeSet, fs::Metadata, os::linux::fs::MetadataExt, path::Path};

use crate::fs::get_groups;

pub fn sgids(pid: u32) -> BTreeSet<u32> {
    BTreeSet::from_iter(get_groups(pid.try_into().unwrap()).unwrap_or(vec![]))
}

#[derive(Debug)]
pub enum Access {
    Lookup,
    Create,
    Setattr,
    Chmod,
    Chown(u32),
    Chgrp(u32),
    Write,
    Delete,
}

pub fn check_perm(
    uid: u32,
    gid: u32,
    pid: u32,
    path: &Path,
    base: &Path,
    access: Access,
) -> Option<i32> {
    // Get supplementary groups
    let sgids = sgids(pid);
    println!(
        "\tPERM: u{} g{}+{:?} p{} {:?} {:?}",
        uid, gid, sgids, pid, path, access
    );
    let mut p = path;
    // Check for traversal permissions
    loop {
        p = match p.parent() {
            Some(v) if v == base => break,
            Some(v) => v,
            None => break,
        };
        if let Ok(meta) = p.metadata() {
            println!("\t parent perm {:?}", p);
            match perm(meta, uid, gid, &sgids, 1, libc::EACCES) {
                None => {}
                Some(libc::EACCES) => {
                    println!("EA");
                    return Some(libc::EACCES);
                }
                Some(_) => todo!(),
            }
        } else {
            println!("ENOE");
            return Some(libc::ENOENT);
        }
    }
    println!("\t self perm {:?}:{:?}", access, path);
    match (
        access,
        path.symlink_metadata(),
        path.parent().map(|x| x.metadata()),
    ) {
        (Access::Lookup, _, _) => None,
        (Access::Create, Ok(m), Some(Ok(m_p))) => perm_overwrite(m, m_p, uid, gid, &sgids),
        (Access::Create, Err(_), Some(Ok(m))) => perm(m, uid, gid, &sgids, 2, libc::EACCES),
        (Access::Delete, Ok(m), Some(Ok(m_p))) => perm_delete(m, m_p, uid, gid, &sgids),
        (Access::Delete, Err(_), _) => todo!("ENOE"),
        (Access::Chmod, Ok(m), _) => perm_chmod(m, uid),
        (Access::Chown(new_uid), Ok(m), _) => perm_chown(m, uid, new_uid),
        (Access::Chgrp(new_gid), Ok(m), _) => perm_chgrp(m, uid, gid, &sgids, new_gid),
        (Access::Write, Ok(m), _) => perm(m, uid, gid, &sgids, 2, libc::EACCES),
        (_, Err(e), _) if e.kind() == std::io::ErrorKind::NotFound => Some(libc::ENOENT),
        (a, b, c) => todo!("\t  {:?} {:?} {:?}", a, b, c),
    }

    /*if let Ok(meta) = path.metadata() {
        match access {
            Access::Lookup => {println!("\t  lookup OK"); None},
            Access::Create => todo!(),
            Access::Write => perm(meta, uid, gid, &sgids, 2),
        }
    } else {
        println!("\t  noent");
        Some(libc::ENOENT)
    }*/
}

fn perm(
    meta: Metadata,
    uid: u32,
    gid: u32,
    sgids: &BTreeSet<u32>,
    mode: u32,
    code: i32,
) -> Option<i32> {
    println!(
        "\t  perm on file: u{} g{} m{:o}?{}",
        meta.st_uid(),
        meta.st_gid(),
        meta.st_mode(),
        mode
    );
    assert!(mode < 8, "Mode must be a single octal digit");
    // Root always has perms
    if uid == 0 {
        return None;
    }
    if meta.st_uid() == uid {
        if meta.st_mode() & (mode << 6) == 0 {
            println!("\t   Eu");
            return Some(code);
        }
    } else if meta.st_gid() == gid || sgids.contains(&meta.st_gid()) {
        if meta.st_mode() & (mode << 3) == 0 {
            println!("\t   Eg");
            return Some(code);
        }
    } else if meta.st_mode() & (mode) == 0 {
        println!("\t   Eo");
        return Some(code);
    }
    None
}

fn perm_chmod(meta: Metadata, uid: u32) -> Option<i32> {
    // You need to be owner or root to chmod
    if uid == meta.st_uid() || uid == 0 {
        None
    } else {
        Some(libc::EPERM)
    }
}

fn perm_chown(meta: Metadata, uid: u32, new_uid: u32) -> Option<i32> {
    // You must be root to chown
    if uid == 0 {
        return None;
    }
    // We're allowed to do a no-op
    if meta.st_uid() == new_uid {
        return None;
    }
    Some(libc::EPERM)
}

fn perm_chgrp(
    meta: Metadata,
    uid: u32,
    gid: u32,
    sgids: &BTreeSet<u32>,
    new_gid: u32,
) -> Option<i32> {
    // You must be root to chgrp, unless you are changing the group to one you're in
    if uid == 0 {
        return None;
    }
    // If we're the owner, we're allowed to change it to a group we're in
    if uid == meta.st_uid() && (new_gid == gid || sgids.contains(&new_gid)) {
        return None;
    }
    Some(libc::EPERM)
}

fn perm_delete(
    meta: Metadata,
    meta_parent: Metadata,
    uid: u32,
    gid: u32,
    sgids: &BTreeSet<u32>,
) -> Option<i32> {
    if uid == 0 {
        return None;
    }
    if (meta_parent.st_mode() & libc::S_ISVTX) != 0 {
        println!("Sticky");
        if meta.st_uid() != uid && meta_parent.st_uid() != uid {
            return Some(libc::EPERM);
        }
    }

    perm(meta_parent, uid, gid, sgids, 2, libc::EACCES)
}

fn perm_overwrite(
    meta: Metadata,
    meta_parent: Metadata,
    uid: u32,
    gid: u32,
    sgids: &BTreeSet<u32>,
) -> Option<i32> {
    if uid == 0 {
        return None;
    }
    if (meta_parent.st_mode() & libc::S_ISVTX) != 0 {
        println!("Sticky");
        if meta.st_uid() != uid && meta_parent.st_uid() != uid {
            return Some(libc::EPERM);
        }
    }

    perm(meta_parent, uid, gid, sgids, 2, libc::EACCES)
}
