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

#[cfg(test)]
mod tests {
    use crate::logwrapper::LogWrapper;

    #[test]
    fn test_lw_int() {
        assert_eq!(5i32.lw(), String::from("5"))
    }

    #[test]
    fn test_lw_bytes() {
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8, 5u8].lw(), String::from("[5]"))
    }

    #[test]
    fn test_lw_ok() {
        assert_eq!(Ok::<i32, i32>(3).lw(), String::from("Ok(3)"))
    }

    #[test]
    fn test_lw_err() {
        assert_eq!(Err::<i32, i32>(3).lw(), String::from("Err(3)"))
    }
}
