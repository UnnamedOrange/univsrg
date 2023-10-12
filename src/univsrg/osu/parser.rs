use std::{
    fs::{create_dir_all, File},
    io,
    path::Path,
};

use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{
    super::{traits::AppendToUnivsrg, types::Package},
    types::OszPath,
};

impl AppendToUnivsrg for OszPath {
    fn append_to_univsrg(&self, package: &mut Package) -> io::Result<()> {
        // Unzip osz file.
        let source_dir: TempDir = tempdir()?;
        let zip_file = File::open(&self.osz_path)?;
        let mut zip = ZipArchive::new(zip_file)?;

        // https://blog.csdn.net/m0_47202518/article/details/120421870
        for idx in 0..zip.len() {
            let mut file = zip.by_index(idx)?;
            let out_path = if let Some(inner_path) = file.enclosed_name() {
                Path::join(source_dir.path(), inner_path)
            } else {
                continue;
            };

            if !file.name().ends_with('/') {
                if let Some(parent) = out_path.parent() {
                    if !parent.exists() {
                        create_dir_all(parent)?;
                    }
                    let mut out_file = File::create(&out_path)?;
                    io::copy(&mut file, &mut out_file)?;
                }
            }
        }

        // TODO: Enumerate osu files and parse.

        Ok(())
    }
}
