use fuser::MountOption;
use posix_invariant_checker::fs::InvFS;

fn main() {
    let base = std::env::args_os().nth(1).unwrap();
    let mountpoint = std::env::args_os().nth(2).unwrap();

    let base = std::env::current_dir().unwrap().join(base);

    fuser::mount2(
        InvFS::new(base.into()),
        mountpoint,
        &[MountOption::AutoUnmount, MountOption::AllowOther],
    )
    .unwrap();
}
