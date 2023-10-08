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

type ReplyAttrOK = (Duration, FileAttr);

pub struct ReplyAttr(OnceCell<Result<ReplyAttrOK, i32>>);

impl ReplyAttr {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn attr(&self, ttl: &Duration, attr: &FileAttr) {
        self.0.set(Ok((*ttl, *attr))).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<(Duration, FileAttr), i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyAttr) {
        match self.0.get().unwrap() {
            Ok((ttl, attr)) => rep.attr(ttl, attr),
            Err(e) => rep.error(*e),
        }
    }
}

type ReplyWriteOK = u32;

pub struct ReplyWrite(OnceCell<Result<ReplyWriteOK, i32>>);

impl ReplyWrite {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn written(&self, written: u32) {
        self.0.set(Ok(written)).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<ReplyWriteOK, i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyWrite) {
        match self.0.get().unwrap() {
            Ok(written) => rep.written(*written),
            Err(e) => rep.error(*e),
        }
    }
}

type ReplyOpenOK = (u64, u32);

pub struct ReplyOpen(OnceCell<Result<ReplyOpenOK, i32>>);

impl ReplyOpen {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn opened(&self, fh: u64, flags: u32) {
        self.0.set(Ok((fh, flags))).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<ReplyOpenOK, i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyOpen) {
        match self.0.get().unwrap() {
            Ok((fh, flags)) => rep.opened(*fh, *flags),
            Err(e) => rep.error(*e),
        }
    }
}

type ReplyDataOK = Vec<u8>;

pub struct ReplyData(OnceCell<Result<ReplyDataOK, i32>>);

impl ReplyData {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn data(&self, data: Vec<u8>) {
        self.0.set(Ok(data)).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<ReplyDataOK, i32> {
        self.0.get().unwrap().clone()
    }
    pub fn reply(&self, rep: fuser::ReplyData) {
        match self.0.get().unwrap() {
            Ok(data) => rep.data(data),
            Err(e) => rep.error(*e),
        }
    }
}

type ReplyEmptyOK = ();

pub struct ReplyEmpty(OnceCell<Result<ReplyEmptyOK, i32>>);

impl ReplyEmpty {
    pub fn new() -> Self {
        Self(OnceCell::new())
    }
    pub fn ok(&self) {
        self.0.set(Ok(())).unwrap();
    }
    pub fn error(&self, e: i32) {
        self.0.set(Err(e)).unwrap()
    }
    pub fn get(&self) -> Result<ReplyEmptyOK, i32> {
        *self.0.get().unwrap()
    }
    pub fn reply(&self, rep: fuser::ReplyEmpty) {
        match self.0.get().unwrap() {
            Ok(()) => rep.ok(),
            Err(e) => rep.error(*e),
        }
    }
}
