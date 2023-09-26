use std::path::PathBuf;

use crate::fs::InvFS;

pub fn create_ifs() -> InvFS {
    let path: PathBuf = std::env::var("PIC_TEST_PATH")
        .expect("Set PIC_TEST_PATH to an empty directory")
        .into();
    let tempdir = path.join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(tempdir.clone()).unwrap();
    InvFS::new(tempdir)
}
