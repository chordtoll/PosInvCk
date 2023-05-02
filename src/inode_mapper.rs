use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use maplit::{btreemap, btreeset};

#[derive(Debug)]
pub struct InodeMapper(BTreeMap<u64, BTreeSet<PathBuf>>);

impl InodeMapper {
    pub fn new() -> Self {
        Self(btreemap! {1 => btreeset![PathBuf::from(".")]})
    }

    pub fn get(&self, ino: u64) -> &Path {
        self.0
            .get(&ino)
            .expect("Accessing an inode we haven't seen before")
            .first()
            .unwrap()
    }

    pub fn insert(&mut self, ino: u64, child: PathBuf) -> u64 {
        if child == PathBuf::from(".") {
            return 1;
        }
        match self.0.entry(ino) {
            Entry::Vacant(v) => {
                v.insert(btreeset![child]);
            }
            Entry::Occupied(mut o) => {
                o.get_mut().insert(child);
            }
        }
        ino
    }

    pub fn remove(&mut self, child: &Path) {
        self.0.retain(|_, v| {
            v.retain(|v| v != child);
            !v.is_empty()
        })
    }

    pub fn rename(&mut self, old: PathBuf, new: PathBuf) {
        self.0.iter_mut().for_each(|(_, v)| {
            if v.remove(&old) {
                v.insert(new.clone());
            }
        })
    }
}
