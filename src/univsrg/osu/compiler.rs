use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use osu_file_parser::{
    difficulty::{CircleSize, Difficulty, HPDrainRate, OverallDifficulty},
    general::{AudioFilename, AudioLeadIn, General, Mode, PreviewTime},
    hitobjects::{HitObject, HitObjectParams::OsuManiaHold},
    metadata::{Artist, ArtistUnicode, Creator, Metadata, Title, TitleUnicode, Version},
    timingpoints::{Effects, SampleIndex, SampleSet, TimingPoint, Volume},
    Decimal, HitObjects, Integer, OsuFile, TimingPoints,
};
use tempfile::{tempdir, TempDir};

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
    let basename = PathBuf::from(beatmap.make_basename());
    let filename: PathBuf = [root, &basename].iter().collect();

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
    difficulty.circle_size = Some(CircleSize::from(Decimal::from(beatmap.column_count as i32)));
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
        .get_path_from_entry(&beatmap.audio)
        .map(|v| AudioFilename::from(v.clone()));
    general.audio_lead_in = beatmap
        .audio_lead_in
        .map(|v| AudioLeadIn::from(v as Integer));
    general.preview_time = beatmap
        .preview_time
        .map(|v| PreviewTime::from(v as Integer));
    // audio_hash
    // are not supported.
    osu_file.general = Some(general);

    let mut timing_points = Vec::<TimingPoint>::new();
    let mut idx_red = 0;
    let mut idx_green = 0;
    while idx_red < beatmap.bpm_time_points.len() && idx_green < beatmap.effect_time_points.len() {
        if beatmap.effect_time_points[idx_green].offset < beatmap.bpm_time_points[idx_red].offset {
            let etp = &beatmap.effect_time_points[idx_green];
            let tp = TimingPoint::new_inherited(
                etp.offset as i32, // u32 bug?
                rust_decimal::Decimal::try_from(etp.velocity_multiplier).unwrap(),
                0,
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
                btp.offset as i32,
                Decimal::new_from_str(&format!("{:.3}", beat_duration_ms)),
                0,
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
        fn position_to_column(x: u32, column_count: u32) -> u32 {
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
                ho.position.x =
                    Decimal::from(column_to_position(*column, beatmap.column_count) as i32);
                ho.time = Decimal::from(*offset as i32);
            }
            LongNote {
                column,
                offset,
                end_offset,
            } => {
                ho = HitObject::osu_mania_hold_default();
                ho.position.x =
                    Decimal::from(column_to_position(*column, beatmap.column_count) as i32);
                ho.time = Decimal::from(*offset as i32);
                ho.obj_params = OsuManiaHold {
                    end_time: Decimal::from(*end_offset),
                }
            }
        }
        hit_objects.push(ho);
    }
    osu_file.hitobjects = Some(HitObjects(hit_objects));

    // TODO: Generate the file.

    File::create(filename)?.write_all(osu_file.to_string().as_bytes())?;

    Ok(())
}

impl ToOsu for Package {
    fn to_osu(&self, path: &Path) -> io::Result<()> {
        let temp_dir: TempDir = tempdir()?;

        // Remap and settle resources.
        let mut resource_out = ResourceOut::new();
        resource_out.inflate(temp_dir.path().to_owned(), &self.resource_pool)?;

        for beatmap in &self.beatmaps {
            // TODO: Handle error.
            compile_beatmap(beatmap, temp_dir.path(), &resource_out)?;
        }

        // TODO: Package all files to a bundle.

        Ok(())
    }
}
