use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use osu_file_parser::{
    difficulty::{Difficulty, HPDrainRate, OverallDifficulty},
    general::{AudioFilename, General, Mode},
    metadata::{Artist, ArtistUnicode, Creator, Metadata, Title, TitleUnicode, Version},
    Decimal, OsuFile,
};
use tempfile::{tempdir, TempDir};

use super::super::{resource::ResourceOut, traits::ToOsu, types::Beatmap, types::Package};

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
    // audio_lead_in, audio_hash, preview_time
    // are not supported.

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
