use std::fs::File;
use std::io;

use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{
    super::{traits::AppendToUnivsrg, types::Package},
    types::OszPath,
};

impl AppendToUnivsrg for OszPath {
    fn append_to_univsrg(&self, package: &mut Package) -> io::Result<()> {
        // TODO: Unzip osz file.
        let source_dir: TempDir = tempdir()?;
        let zip_file = File::open(&self.osz_path)?;
        let zip = ZipArchive::new(zip_file)?;

        // TODO: Enumerate osu files and parse.

        Ok(())
    }
}
