use std::{
    fs::{create_dir_all, read_dir, File},
    io::{self, Read},
    path::Path,
};

use osu_file_parser::{OsuFile, VersionedToString};
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{
    super::{
        traits::AppendToUnivsrg,
        types::{Beatmap, Package},
    },
    types::OszPath,
};

fn parse_osu_file(path: &Path, package: &mut Package) -> io::Result<()> {
    let mut file = File::open(&path)?;
    let mut osu_file_string = String::new();
    file.read_to_string(&mut osu_file_string)?;
    let osu_file = osu_file_string.parse::<OsuFile>().unwrap();
    let osu_file_version: u8 = osu_file.version;

    // TODO: Parse osu file.
    let resource_pool = &mut package.resource_pool;
    let mut beatmap = Beatmap::new();

    let metadata = osu_file.metadata.as_ref();
    metadata.map(|m| {
        m.title
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .map(|v| beatmap.title.latin = Some(v));
        m.title
            .as_ref()
            .and_then(|v| v.to_string((osu_file_version)))
            .map(|v| beatmap.title.unicode = Some(v));
        m.artist
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .map(|v| beatmap.artist.latin = Some(v));
        m.artist
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .map(|v| beatmap.artist.unicode = Some(v));
        m.creator
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .map(|v| beatmap.creator = Some(v));
        m.version
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .map(|v| beatmap.version = Some(v));
    });

    let difficulty = osu_file.difficulty.as_ref();
    difficulty.map(|d| {
        d.circle_size
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<u32>().ok())
            .map(|v| beatmap.column_count = Some(v));
        d.hp_drain_rate
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| beatmap.hp_difficulty = Some(v));
        d.overall_difficulty
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| beatmap.acc_difficulty = Some(v));
    });

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
