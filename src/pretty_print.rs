pub trait PPStat {
    fn ppstat(&self) -> String;
}

impl PPStat for libc::stat {
    fn ppstat(&self) -> String {
        format!(
            "
STAT[
    uid={}
    gid={}
    mode={:o}
]",
            self.st_uid, self.st_gid, self.st_mode,
        )
    }
}

impl<S: PPStat, D: std::fmt::Debug> PPStat for Result<S, D> {
    fn ppstat(&self) -> String {
        match self {
            Ok(v) => format!("Ok({})", v.ppstat()),
            Err(e) => format!("Err({:?})", e),
        }
    }
}
