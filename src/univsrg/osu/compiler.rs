use std::{
    fs::File,
    io::{self, Error, ErrorKind, Write},
    path::{Path, PathBuf},
};

use osu_file_parser::{
    difficulty::{CircleSize, Difficulty, HPDrainRate, OverallDifficulty},
    events::{Background, Event},
    general::{AudioFilename, AudioLeadIn, Countdown, General, Mode, PreviewTime},
    hitobjects::{HitObject, HitObjectParams::OsuManiaHold, HitSample},
    metadata::{Artist, ArtistUnicode, Creator, Metadata, Title, TitleUnicode, Version},
    timingpoints::{Effects, SampleIndex, SampleSet, TimingPoint, Volume},
    Decimal, Events, FilePath, HitObjects, OsuFile, TimingPoints, VersionedDefault,
};
use tempfile::{tempdir, TempDir};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter};

use super::super::{
    resource::ResourceOut,
    traits::ToOsu,
    types::{
        Beatmap,
        Object::{LongNote, Note},
        Package,
    },
};

fn compile_beatmap(beatmap: &Beatmap, root: &Path, resource: &ResourceOut) -> io::Result<()> {
    // Refuse to compile if column count or audio is None.
    if beatmap.column_count.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "[beatmap.column_count] is None",
        ));
    }
    if beatmap.audio.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "[beatmap.audio] is None",
        ));
    }

    let basename = beatmap.make_basename();
    let filename = PathBuf::from(basename + ".osu");
    let out_file_path: PathBuf = [root, &filename].iter().collect();

    let mut osu_file = OsuFile::new(14);

    let mut metadata = Metadata::new();
    metadata.title = beatmap //
        .title
        .latin
        .as_ref()
        .map(|v| Title::from(v.clone()));
    metadata.title_unicode = beatmap //
        .title
        .unicode
        .as_ref()
        .map(|v| TitleUnicode::from(v.clone()));
    metadata.artist = beatmap //
        .artist
        .latin
        .as_ref()
        .map(|v| Artist::from(v.clone()));
    metadata.artist_unicode = beatmap //
        .artist
        .unicode
        .as_ref()
        .map(|v| ArtistUnicode::from(v.clone()));
    metadata.creator = beatmap //
        .creator
        .as_ref()
        .map(|v| Creator::from(v.clone()));
    metadata.version = beatmap //
        .version
        .as_ref()
        .map(|v| Version::from(v.clone()));
    // source, tags, beatmap_id, beatmap_set_id
    // are not supported.
    osu_file.metadata = Some(metadata);

    let mut difficulty = Difficulty::new();
    // Column count is circle size.
    difficulty.circle_size = Some(CircleSize::from(Decimal::from(
        beatmap.column_count.unwrap() as i32,
    )));
    difficulty.hp_drain_rate = beatmap
        .hp_difficulty
        .as_ref()
        .map(|v| HPDrainRate::from(Decimal::new_from_str(&format!("{:.1}", v))));
    difficulty.overall_difficulty = beatmap
        .acc_difficulty
        .as_ref()
        .map(|v| OverallDifficulty::from(Decimal::new_from_str(&format!("{:.1}", v))));
    osu_file.difficulty = Some(difficulty);

    let mut general = General::new();
    general.mode = Some(Mode::Mania);
    general.audio_filename = resource
        .get_path_from_entry(beatmap.audio.as_ref().unwrap())
        .map(|v| AudioFilename::from(v.clone()));
    general.audio_lead_in = beatmap.audio_lead_in.map(|v| AudioLeadIn::from(v));
    general.preview_time = beatmap.preview_time.map(|v| PreviewTime::from(v));
    // audio_hash
    // are not supported.
    // Count down is not "No Count Down" by default, so we turn it off manually.
    general.countdown = Some(Countdown::NoCountdown);
    osu_file.general = Some(general);

    let mut timing_points = Vec::<TimingPoint>::new();
    let mut idx_red = 0;
    let mut idx_green = 0;
    while idx_red < beatmap.bpm_time_points.len() || idx_green < beatmap.effect_time_points.len() {
        if idx_red >= beatmap.bpm_time_points.len()
            || beatmap.effect_time_points[idx_green].offset
                < beatmap.bpm_time_points[idx_red].offset
        {
            let etp = &beatmap.effect_time_points[idx_green];
            let tp = TimingPoint::new_inherited(
                etp.offset,
                rust_decimal::Decimal::try_from(etp.velocity_multiplier).unwrap(),
                0, // Ignored by inherited timing points.
                SampleSet::BeatmapDefault,
                SampleIndex::OsuDefaultHitsounds,
                Volume::new(100, 14).unwrap(),
                Effects::new(false, false),
            );
            timing_points.push(tp);
            idx_green += 1;
        } else {
            let btp = &beatmap.bpm_time_points[idx_red];
            let beat_duration_ms = 60000f32 / btp.bpm;
            let tp = TimingPoint::new_uninherited(
                btp.offset,
                Decimal::new_from_str(&format!("{:.3}", beat_duration_ms)),
                btp.beats_per_bar as i32,
                SampleSet::BeatmapDefault,
                SampleIndex::OsuDefaultHitsounds,
                Volume::new(100, 14).unwrap(),
                Effects::new(false, false),
            );
            timing_points.push(tp);
            idx_red += 1;
        }
    }
    osu_file.timing_points = Some(TimingPoints(timing_points));

    let mut hit_objects = Vec::<HitObject>::new();
    for object in &beatmap.objects {
        // https://osu.ppy.sh/wiki/en/Client/File_formats/osu_%28file_format%29#holds-(osu!mania-only)
        fn _position_to_column(x: u32, column_count: u32) -> u32 {
            x * column_count / 512
        }
        fn column_to_position(column: u32, column_count: u32) -> u32 {
            (2 * column + 1) * 512 / 2 / column_count
        }
        let mut ho;
        match object {
            // Note: 要将 enum 的类型单独匹配为一个对象，只能写成 new type。
            Note { column, offset } => {
                ho = HitObject::hitcircle_default();
                ho.position.x = Decimal::from(column_to_position(
                    *column,
                    beatmap.column_count.unwrap(),
                ) as i32);
                ho.time = Decimal::from(*offset as i32);
            }
            LongNote {
                column,
                offset,
                end_offset,
            } => {
                ho = HitObject::osu_mania_hold_default();
                ho.position.x = Decimal::from(column_to_position(
                    *column,
                    beatmap.column_count.unwrap(),
                ) as i32);
                ho.time = Decimal::from(*offset as i32);
                ho.obj_params = OsuManiaHold {
                    end_time: Decimal::from(*end_offset),
                };
                // For holds, a default hit sample must be given.
                ho.hitsample = HitSample::default(14);
            }
        }
        hit_objects.push(ho);
    }
    osu_file.hitobjects = Some(HitObjects(hit_objects));

    let mut events = Vec::<Event>::new();
    beatmap
        .background
        .as_ref()
        .and_then(|b| resource.get_path_from_entry(b))
        .map(|v| Background {
            start_time: 0,
            file_name: FilePath::from(v),
            position: None,
            commands: vec![],
        })
        .map(|v| {
            events.push(Event::Background(v));
        });
    osu_file.events = Some(Events(events));

    File::create(out_file_path)?.write_all(osu_file.to_string().as_bytes())?;

    Ok(())
}

fn zip_folder<P: AsRef<Path>>(folder_path: P, zip_path: P) -> io::Result<()> {
    let folder_path = folder_path.as_ref();
    let zip_path = zip_path.as_ref();

    let zip_file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options =
        zip::write::FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let mut file = File::open(path)?;
            let relative_path = path
                .strip_prefix(folder_path)
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            zip.start_file(relative_path.to_string_lossy(), options)?;
            io::copy(&mut file, &mut zip)?;
        }
    }

    zip.finish()?;

    Ok(())
}

impl ToOsu for Package {
    fn to_osu(&self, path: &Path) -> io::Result<()> {
        let temp_dir: TempDir = tempdir()?;

        // Remap and settle resources.
        let mut resource_out = ResourceOut::new();
        resource_out.inflate(temp_dir.path().to_owned(), &self.resource_pool)?;

        // Compile beatmaps.
        for beatmap in &self.beatmaps {
            let result = compile_beatmap(beatmap, temp_dir.path(), &resource_out);
            if result.is_err() {
                continue;
            }
        }

        // Package all files to a bundle.
        zip_folder(temp_dir.as_ref(), &path)?;

        Ok(())
    }
}
