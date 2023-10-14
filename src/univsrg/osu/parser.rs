use std::{
    fs::{create_dir_all, read_dir, File},
    io::{self, Read},
    path::Path,
};

use osu_file_parser::OsuFile;
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{
    super::{traits::AppendToUnivsrg, types::Package},
    types::OszPath,
};

fn parse_osu_file(path: &Path, package: &mut Package) -> io::Result<()> {
    let mut file = File::open(&path)?;
    let mut osu_file_string = String::new();
    file.read_to_string(&mut osu_file_string)?;
    let osu_file = osu_file_string.parse::<OsuFile>().unwrap();

    // TODO: Parse osu file.

    Ok(())
}

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

        // Enumerate osu files and parse.
        for entry in read_dir(&source_dir)? {
            let path = entry?.path();
            if !path.ends_with(".osu") {
                continue;
            }
            let _ = parse_osu_file(&path, package);
        }

        Ok(())
    }
}
