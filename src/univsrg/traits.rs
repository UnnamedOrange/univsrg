use std::{io, path::Path};

use super::types::Package;

pub trait AppendToUnivsrg {
    fn append_to_univsrg(&self, package: &mut Package) -> io::Result<()>;
}

pub trait ToOsu {
    fn to_osu(&self, path: &Path) -> io::Result<()>;
}

pub trait ToMalody {}
