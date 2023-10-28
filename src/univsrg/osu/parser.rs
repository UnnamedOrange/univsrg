use std::{
    fs::{create_dir_all, read_dir, File},
    io::{self, ErrorKind, Read},
    path::{Path, PathBuf},
};

use osu_file_parser::{events::Event, hitobjects::HitObjectParams, OsuFile, VersionedToString};
use rust_decimal::prelude::ToPrimitive;
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{
    super::{
        resource::ResourceEntry,
        traits::AppendToUnivsrg,
        types::{Beatmap, BpmTimePoint, EffectTimePoint, Object, Package},
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

    let resource_pool = &mut package.resource_pool;
    let mut beatmap = Beatmap::new();

    let metadata = osu_file.metadata.as_ref();
    metadata.map(|m| {
        beatmap.title.latin = m.title.as_ref().and_then(|v| v.to_string(osu_file_version));
        beatmap.title.unicode = m
            .title_unicode
            .as_ref()
            .and_then(|v| v.to_string(osu_file_version));
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
    beatmap.column_count.ok_or(io::Error::new(
        ErrorKind::Other,
        "Column count is necessary.",
    ))?;

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
            })
            .map(|v| {
                resource_pool.insert(v.clone());
                v
            });
    });

    let timing_points = osu_file.timing_points.as_ref();
    timing_points.map(|t| &t.0).map(|t| {
        let mut btps = Vec::<BpmTimePoint>::new();
        let mut etps = Vec::<EffectTimePoint>::new();
        for tp in t {
            let offset = tp.time().to_string().parse::<u32>().ok();
            if tp.uninherited() {
                let bpm = tp.calc_bpm().and_then(|v| v.to_f32());
                let beats_per_bar = tp.meter() as u32;
                if let (Some(offset), Some(bpm)) = (offset, bpm) {
                    btps.push(BpmTimePoint {
                        offset,
                        bpm,
                        beats_per_bar,
                    });
                }
            } else {
                let velocity_multiplier = tp
                    .calc_slider_velocity_multiplier()
                    .and_then(|v| v.to_f32());
                if let (Some(offset), Some(velocity_multiplier)) = (offset, velocity_multiplier) {
                    etps.push(EffectTimePoint {
                        offset,
                        velocity_multiplier,
                    })
                };
            }
        }
        beatmap.bpm_time_points = btps;
        beatmap.effect_time_points = etps;
    });

    let hit_objects = osu_file.hitobjects.as_ref();
    hit_objects.map(|h| &h.0).map(|h| {
        let mut objects = Vec::<Object>::new();
        // https://osu.ppy.sh/wiki/en/Client/File_formats/osu_%28file_format%29#holds-(osu!mania-only)
        fn position_to_column(x: u32, column_count: u32) -> u32 {
            x * column_count / 512
        }
        for ho in h {
            let x = ho.position.x.to_string().parse::<u32>().ok();
            let column = x.map(|v| position_to_column(v, beatmap.column_count.unwrap()));
            let offset = ho.time.to_string().parse::<u32>().ok();
            if let (Some(column), Some(offset)) = (column, offset) {
                match &ho.obj_params {
                    HitObjectParams::HitCircle => objects.push(Object::Note { column, offset }),
                    HitObjectParams::OsuManiaHold { end_time } => {
                        end_time.to_string().parse::<i32>().ok().map(|v| {
                            objects.push(Object::LongNote {
                                column,
                                offset,
                                end_offset: v,
                            })
                        });
                    }
                    _ => {}
                }
            }
        }
        beatmap.objects = objects;
    });

    let events = osu_file.events.as_ref();
    events.map(|e| &e.0).map(|e| {
        for event in e {
            if let Event::Background(bg) = event {
                ResourceEntry::new_from_file_in_bundle(bundle_base, bg.file_name.get().to_owned())
                    .ok()
                    .map(|v| {
                        resource_pool.insert(v.clone());
                        beatmap.background = Some(v);
                    });
                break;
            }
        }
    });

    package.beatmaps.push(beatmap);

    Ok(())
}

impl AppendToUnivsrg for OszPath {
    fn append_to_univsrg(&self, package: &mut Package) -> io::Result<()> {
        // Unzip osz file.
        let source_dir: TempDir = tempdir()?;
        let zip_file = File::open(&self.0)?;
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
            if !path
                .extension()
                .and_then(|v| v.to_str())
                .is_some_and(|v| v == "osu")
            {
                continue;
            }
            let _ = parse_osu_file(&path, source_dir.path(), package);
        }

        Ok(())
    }
}
