use std::sync::atomic::AtomicU64;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CALL_ID: AtomicU64 = AtomicU64::new(0);
}

pub type CallID = u64;

#[macro_export]
macro_rules! log_call {
    ($call:literal, $($arg:expr),* $(,)?) => {{
        let id = $crate::logging::CALL_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("{}({}): {}",$call,format!($($arg,)*),id);
        id
    }};
}

#[macro_export]
macro_rules! log_more {
    ($callid: ident, $($arg:expr),* $(,)?) => {{
        println!(" {} : {}",$callid,format!($($arg,)*));
    }};
}

#[macro_export]
macro_rules! log_res {
    ($callid: ident, $($arg:expr),* $(,)?) => {{
        println!(" {} => {}",$callid,format!($($arg,)*));
    }};
}

#[macro_export]
macro_rules! log_acces {
    ($callid:ident, $path:expr) => {
        use std::os::linux::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;
        let mut path = $path.clone();
        loop {
            if let Ok(meta) = path.metadata() {
                log_more!(
                    $callid,
                    "ACCES({:?}): {}:{} {:o}",
                    path,
                    meta.st_uid(),
                    meta.st_gid(),
                    meta.permissions().mode()
                );
                path = if let Some(v) = path.parent() {
                    v.to_path_buf()
                } else {
                    break;
                }
            } else {
                log_more!($callid, "ACCES failed");
                break;
            }
        }
    };
}
