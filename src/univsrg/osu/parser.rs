use std::{
    fs::{create_dir_all, read_dir, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use super::super::resource::ResourceEntry;
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

fn parse_osu_file(
    osu_file_path: &Path,
    bundle_base: &Path,
    package: &mut Package,
) -> io::Result<()> {
    let mut file = File::open(&osu_file_path)?;
    let mut osu_file_string = String::new();
    file.read_to_string(&mut osu_file_string)?;
    let osu_file = osu_file_string.parse::<OsuFile>().unwrap();
    let osu_file_version: u8 = osu_file.version;

    // TODO: Parse osu file.
    let resource_pool = &mut package.resource_pool;
    let mut beatmap = Beatmap::new();

    let metadata = osu_file.metadata.as_ref();
    metadata.map(|m| {
        beatmap.title.latin = m.title.as_ref().and_then(|v| v.to_string(osu_file_version));
        beatmap.title.unicode = m
            .title_unicode
            .as_ref()
            .and_then(|v| v.to_string((osu_file_version)));
        beatmap.artist.latin = m
            .artist
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version));
        beatmap.artist.unicode = m
            .artist_unicode
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version));
        beatmap.creator = m
            .creator
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version));
        beatmap.version = m
            .version
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version));
    });

    let difficulty = osu_file.difficulty.as_ref();
    difficulty.map(|d| {
        beatmap.column_count = d
            .circle_size
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<u32>().ok());
        beatmap.hp_difficulty = d
            .hp_drain_rate
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<f32>().ok());
        beatmap.acc_difficulty = d
            .overall_difficulty
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse::<f32>().ok());
    });

    let general = osu_file.general.as_ref();
    general.map(|g| {
        beatmap.preview_time = g
            .preview_time
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse().ok());
        beatmap.audio_lead_in = g
            .audio_lead_in
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| v.parse().ok());
        beatmap.audio = g
            .audio_filename
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version))
            .and_then(|v| {
                ResourceEntry::new_from_file_in_bundle(bundle_base, PathBuf::from(v)).ok()
            });
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
            let _ = parse_osu_file(&path, source_dir.path(), package);
        }

        Ok(())
    }
}
