use std::io;

use super::{
    super::{traits::AppendToUnivsrg, types::Package},
    types::OszPath,
};

impl AppendToUnivsrg for OszPath {
    fn append_to_univsrg(&self, package: &mut Package) -> io::Result<()> {
        // TODO: Unzip osz file.

        // TODO: Enumerate osu files and parse.

        Ok(())
    }
}
