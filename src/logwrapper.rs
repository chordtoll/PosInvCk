pub trait LogWrapper {
    fn lw(&self) -> String;
}

impl LogWrapper for i32 {
    fn lw(&self) -> String {
        format!("{:x}", self)
    }
}

impl LogWrapper for Vec<u8> {
    fn lw(&self) -> String {
        format!("[{:x}]", self.len())
    }
}

impl<T: LogWrapper, E: LogWrapper> LogWrapper for Result<T, E> {
    fn lw(&self) -> String {
        match self {
            Ok(v) => format!("Ok({})", v.lw()),
            Err(e) => format!("Err({})", e.lw()),
        }
    }
}
