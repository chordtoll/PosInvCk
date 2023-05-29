use std::{os::unix::prelude::MetadataExt, path::PathBuf};

use crate::{
    file_attr::FileAttr, fs::InvFS, invariants::INODE_PATHS, load_prev_contents, logging::CallID,
};

pub struct InitInv {
    root: PathBuf,
}

pub fn inv_init_before(
    _callid: CallID,
    fs: &InvFS,
    _req: &fuser::Request<'_>,
    _config: &fuser::KernelConfig,
) -> InitInv {
    InitInv {
        root: fs.root.clone(),
    }
}
pub fn inv_init_after(_callid: CallID, inv: InitInv, _res: &Result<(), i32>) {
    #[cfg(any(
        feature = "check-meta",
        feature = "check-dirs",
        feature = "check-data",
        feature = "check-xattr"
    ))]
    if load_prev_contents() {
        println!("Loaded previous filesystem contents");
    } else {
        println!("No previous filesystem contents, scanning...");
        let mut ip = INODE_PATHS.lock().unwrap();
        ip.insert(1, inv.root.clone());
        for e in walkdir::WalkDir::new(inv.root.clone()) {
            let e = e.expect("Encountered error while scanning filesystem");
            let m = e
                .metadata()
                .unwrap_or_else(|_| panic!("Failed to get metadata for {:?}", e.path()));
            let ino = if e.path() == inv.root { 1 } else { m.ino() };
            ip.insert(ino, e.path().to_path_buf());
            #[cfg(feature = "check-dirs")]
            if m.is_dir() {
                let dc = e
                    .path()
                    .read_dir()
                    .unwrap()
                    .map(|x| x.unwrap())
                    .map(|x| (x.file_name(), x.metadata().unwrap().ino()))
                    .collect();
                match crate::invariants::DIR_CONTENTS.lock().unwrap().entry(ino) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(dc);
                    }
                    std::collections::btree_map::Entry::Occupied(o) => {
                        assert!(
                            o.get() == &dc,
                            "Same directory returned different contents at different paths"
                        );
                    }
                }
            }
            #[cfg(feature = "check-meta")]
            {
                match crate::invariants::INODE_CONTENTS.lock().unwrap().entry(ino) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(FileAttr::from(&m).set_ino(ino));
                    }
                    std::collections::btree_map::Entry::Occupied(o) => {
                        assert!(
                            o.get() == &FileAttr::from(&m).set_ino(ino),
                            "Same inode returned different metadata at different paths"
                        );
                    }
                }
            }
            #[cfg(feature = "check-data")]
            if m.is_file() {
                let data = std::fs::read(e.path()).expect("Failed to read file");
                match crate::invariants::FILE_CONTENTS.lock().unwrap().entry(ino) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(data);
                    }
                    std::collections::btree_map::Entry::Occupied(o) => {
                        assert!(
                            o.get() == &data,
                            "Same inode returned different content at different paths"
                        );
                    }
                }
            }
            #[cfg(feature = "check-xattr")]
            {
                let xa = xattr::list(e.path()).unwrap();
                let xa = xa
                    .map(|x| (x.clone(), xattr::get(e.path(), x).unwrap().unwrap()))
                    .collect();
                match crate::invariants::XATTR_CONTENTS.lock().unwrap().entry(ino) {
                    std::collections::btree_map::Entry::Vacant(v) => {
                        v.insert(xa);
                    }
                    std::collections::btree_map::Entry::Occupied(o) => {
                        assert!(
                            o.get() == &xa,
                            "Same inode returned different xattrs at different paths"
                        );
                    }
                }
            }
        }
    }
}
