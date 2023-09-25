use std::path::PathBuf;

use fuser::MountOption;
use posix_invariant_checker::fs::InvFS;

fn main() {
    let base = std::env::args_os().nth(1).expect("[base] [mountpoint]");
    let mountpoint = std::env::args_os().nth(2).expect("[base] [mountpoint]");


    fuser::mount2(
        InvFS::new(PathBuf::from(base)),
        mountpoint,
        &[MountOption::AutoUnmount, MountOption::AllowOther],
    )
    .unwrap();

    //store_prev_contents();

    println!("Unmounted")
}
