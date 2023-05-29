#![allow(clippy::too_many_arguments)] // We have no control over the signatures of fuse calls
#![allow(clippy::new_without_default)]

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io::{Seek, Write},
    os::unix::prelude::{FileExt, OsStrExt, OsStringExt},
};

use inode_mapper::InodeMapper;
use stfu8::{decode_u8, encode_u8};

pub mod file_attr;
pub mod fs;
pub mod fs_to_fuse;
pub mod inode_mapper;
pub mod invariants;
pub mod logging;
pub mod logwrapper;

pub fn load_prev_contents() -> bool {
    let rfr = std::fs::remove_file("fs.contents");
    if rfr.is_ok() {
        
        println!("path");
        *invariants::INODE_PATHS.lock().unwrap() = InodeMapper::load(
            ron::from_str(&std::fs::read_to_string("fs.path").unwrap()).unwrap(),
        );
        #[cfg(feature = "check-meta")]
        {
            println!("meta");
            *invariants::INODE_CONTENTS.lock().unwrap() =
                ron::from_str(&std::fs::read_to_string("fs.meta").unwrap()).unwrap();
        }
        #[cfg(feature = "check-dirs")]
        {
            println!("dirs");
            let dc: BTreeMap<u64, BTreeMap<String, u64>> =
                ron::from_str(&std::fs::read_to_string("fs.dirs").unwrap()).unwrap();
            *invariants::DIR_CONTENTS.lock().unwrap() = dc
                .iter()
                .map(|(k, v)| {
                    (
                        *k,
                        v.iter()
                            .map(|(k, v)| (OsString::from_vec(decode_u8(k).unwrap()), *v))
                            .collect(),
                    )
                })
                .collect();
        }
        #[cfg(feature = "check-data")]
        {
            println!("data");
            let data = std::fs::File::open("fs.data").unwrap();
            let fc: BTreeMap<u64, (u64, usize)> =
                ron::from_str(&std::fs::read_to_string("fs.data.index").unwrap()).unwrap();
            *invariants::FILE_CONTENTS.lock().unwrap() = fc
                .iter()
                .map(|(k, v)| {
                    let mut buf = vec![0; v.1];
                    data.read_exact_at(&mut buf, v.0).unwrap();
                    (*k, buf)
                })
                .collect();
        }
        #[cfg(feature = "check-xattr")]
        {
            println!("xattr");
            *invariants::XATTR_CONTENTS.lock().unwrap() =
                ron::from_str(&std::fs::read_to_string("fs.xattr").unwrap()).unwrap();
        }
        true
    } else {
        println!("Failed to load previous contents: {:?}", rfr);
        false
    }
}

pub fn store_prev_contents() {
    let pc = ron::ser::PrettyConfig::default();

    println!("path");
    ron::ser::to_writer_pretty(
        File::create("fs.path").unwrap(),
        &invariants::INODE_PATHS.lock().unwrap().store(),
        pc.clone(),
    )
    .unwrap();

    #[cfg(feature = "check-meta")]
    {
        println!("meta");
        ron::ser::to_writer_pretty(
            File::create("fs.meta").unwrap(),
            &*invariants::INODE_CONTENTS.lock().unwrap(),
            pc.clone(),
        )
        .unwrap();
    }
    #[cfg(feature = "check-dirs")]
    {
        println!("dirs");
        let dr: BTreeMap<u64, BTreeMap<String, u64>> = invariants::DIR_CONTENTS
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    v.iter()
                        .map(|(k, v)| (encode_u8(k.as_bytes()), *v))
                        .collect(),
                )
            })
            .collect();
        ron::ser::to_writer_pretty(File::create("fs.dirs").unwrap(), &dr, pc.clone()).unwrap();
    }
    #[cfg(feature = "check-data")]
    {
        println!("data");
        let mut data = std::fs::File::create("fs.data").unwrap();
        let fr: BTreeMap<u64, (u64, usize)> = invariants::FILE_CONTENTS
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                let idx = data.stream_position().unwrap();
                data.write_all(v).unwrap();
                (*k, (idx, v.len()))
            })
            .collect();
        ron::ser::to_writer_pretty(File::create("fs.data.index").unwrap(), &fr, pc).unwrap();
    }
    #[cfg(feature = "check-xattr")]
    {
        println!("xattr");
        ron::ser::to_writer_pretty(
            File::create("fs.xattr").unwrap(),
            &*invariants::XATTR_CONTENTS.lock().unwrap(),
            pc,
        )
        .unwrap();
    }
    std::fs::write("fs.contents", "").unwrap();
}
