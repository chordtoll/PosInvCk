use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Mutex,
};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref INODE_PATHS: Mutex<BTreeMap<u64, BTreeSet<PathBuf>>> =
        Mutex::new(BTreeMap::new());
}
pub mod init;
