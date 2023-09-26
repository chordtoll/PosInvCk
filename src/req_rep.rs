use std::{fmt::Debug, sync::Mutex};

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
