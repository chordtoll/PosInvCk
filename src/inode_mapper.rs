use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use maplit::{btreemap, btreeset};

#[derive(Debug, Default)]
pub struct InodeMapper(BTreeMap<u64, BTreeSet<PathBuf>>);

impl InodeMapper {
    pub fn new() -> Self {
        Self(btreemap! {})
    }

    pub fn load(v: BTreeMap<u64, BTreeSet<PathBuf>>) -> Self {
        Self(v)
    }
    pub fn store(&self) -> BTreeMap<u64, BTreeSet<PathBuf>> {
        self.0.clone()
    }

    pub fn get(&self, ino: u64) -> &Path {
        self.0
            .get(&ino)
            .unwrap_or_else(|| panic!("Accessing an inode we haven't seen before: {}", ino))
            .iter()
            .next()
            .unwrap()
    }

    pub fn get_all(&self, ino: u64) -> Option<&BTreeSet<PathBuf>> {
        self.0.get(&ino)
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
        self.remove(&new);
        self.0.iter_mut().for_each(|(k, v)| {
            if v.remove(&old) {
                v.insert(new.clone());
            }
            *v = v
                .iter()
                .map(|x| {
                    if let Ok(v) = x.strip_prefix(old.clone()) {
                        println!("{:?} -R> ({}) {:?}", x, k, new.join(v));
                        new.join(v)
                    } else {
                        x.clone()
                    }
                })
                .collect();
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use maplit::{btreemap, btreeset};

    use super::InodeMapper;

    #[test]
    fn empty() {
        let im = InodeMapper::new();
        assert_eq!(im.store(), btreemap! {});
    }
    #[test]
    fn insert_same_inode() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/foo")), 2);
        assert_eq!(im.insert(2, PathBuf::from("/bar")), 2);
        assert_eq!(
            im.store(),
            btreemap! {2=>btreeset!{PathBuf::from("/foo"),PathBuf::from("/bar")}}
        );
    }
    #[test]
    fn insert_different_inode() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/foo")), 2);
        assert_eq!(im.insert(3, PathBuf::from("/bar")), 3);
        assert_eq!(
            im.store(),
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar")}}
        );
    }
    #[test]
    fn insert_dot() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from(".")), 1);
        assert_eq!(im.store(), btreemap! {});
    }
    #[test]
    fn load() {
        let im = InodeMapper::load(
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")}},
        );
        assert_eq!(
            im.store(),
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")}}
        )
    }
    #[test]
    fn remove() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/foo")), 2);
        assert_eq!(im.insert(2, PathBuf::from("/bar")), 2);
        im.remove(&PathBuf::from("/foo"));
        assert_eq!(im.store(), btreemap! {2=>btreeset!{PathBuf::from("/bar")}});
    }
    #[test]
    fn remove_last() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/foo")), 2);
        im.remove(&PathBuf::from("/foo"));
        assert_eq!(im.store(), btreemap! {});
    }
    #[test]
    fn rename() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/bar")), 2);
        im.rename(PathBuf::from("/bar"), PathBuf::from("/baz"));
        assert_eq!(im.store(), btreemap! {2=>btreeset!{PathBuf::from("/baz")}});
    }
    #[test]
    fn rename_subdirs() {
        let mut im = InodeMapper::new();
        assert_eq!(im.insert(2, PathBuf::from("/bar/foo")), 2);
        im.rename(PathBuf::from("/bar"), PathBuf::from("/baz"));
        assert_eq!(
            im.store(),
            btreemap! {2=>btreeset!{PathBuf::from("/baz/foo")}}
        );
    }
    #[test]
    fn get() {
        let im = InodeMapper::load(
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")}},
        );
        assert_eq!(
            im.get(3),
            PathBuf::from("/bar")
        )
    }
    #[test]
    fn get_all() {
        let im = InodeMapper::load(
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")}},
        );
        assert_eq!(
            im.get_all(3),
            Some(&btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")})
        )
    }
    #[test]
    #[should_panic]
    fn get_nonexistant() {
        let im = InodeMapper::load(
            btreemap! {2=>btreeset!{PathBuf::from("/foo")},3=>btreeset!{PathBuf::from("/bar"),PathBuf::from("/baz")}},
        );
        im.get(4);
    }
}
