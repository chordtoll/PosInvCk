use std::{fmt::Debug, sync::Mutex, time::Duration};

use fuser::FileAttr;
use once_cell::sync::OnceCell;

pub struct Request {
    pub uid: u32,
    pub gid: u32,
    pub pid: u32,
}

impl Request {
    pub fn uid(&self) -> u32 {
        self.uid
    }
    pub fn gid(&self) -> u32 {
        self.gid
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
}

impl<'a> From<&fuser::Request<'a>> for Request {
    fn from(value: &fuser::Request<'a>) -> Self {
        Self {
            uid: value.uid(),
            gid: value.gid(),
            pid: value.pid(),
        }
    }
}

pub struct KernelConfig<'a>(Option<Mutex<&'a mut fuser::KernelConfig>>);

impl<'a> KernelConfig<'a> {
    pub fn new(config: &'a mut fuser::KernelConfig) -> Self {
        Self(Some(Mutex::new(config)))
    }
    pub fn empty() -> Self {
        Self(None)
    }
}

impl<'a> Debug for KernelConfig<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.0 {
            f.debug_tuple("KernelConfig")
                .field(&v.lock().unwrap())
                .finish()
        } else {
            f.debug_tuple("KernelConfig").field(&None::<()>).finish()
        }
    }
}

type ReplyCreateOK = (Duration, FileAttr, u64, u64, u32);

pub struct ReplyCreate(OnceCell<Result<ReplyCreateOK, i32>>);

impl ReplyCreate {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn created(&self, ttl: &Duration, attr: &FileAttr, generation: u64, fh: u64, flags: u32) {
        self.0
            .set(Ok((*ttl, *attr, generation, fh, flags)))
            .unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<(Duration, FileAttr, u64, u64, u32), i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyCreate) {
        match self.0.get().unwrap() {
            Ok((ttl, attr, generation, fh, flags)) => {
                rep.created(ttl, attr, *generation, *fh, *flags)
            }
            Err(e) => rep.error(*e),
        }
    }
}

type ReplyEntryOK = (Duration, FileAttr, u64);

pub struct ReplyEntry(OnceCell<Result<ReplyEntryOK, i32>>);

impl ReplyEntry {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn entry(&self, ttl: &Duration, attr: &FileAttr, generation: u64) {
        self.0.set(Ok((*ttl, *attr, generation))).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<(Duration, FileAttr, u64), i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyEntry) {
        match self.0.get().unwrap() {
            Ok((ttl, attr, generation)) => rep.entry(ttl, attr, *generation),
            Err(e) => rep.error(*e),
        }
    }
}
