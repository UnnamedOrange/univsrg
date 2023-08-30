use std::{io, path::Path};

pub trait ToUnivsrg {}

pub trait ToOsu {
    fn to_osu(&self, path: &Path) -> io::Result<()>;
}

pub trait ToMalody {}
