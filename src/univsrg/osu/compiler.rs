use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use osu_file_parser::{
    metadata::{Metadata, Title},
    OsuFile,
};
use tempfile::{tempdir, TempDir};

use super::super::{resource::ResourceOut, traits::ToOsu, types::Beatmap, types::Package};

fn compile_beatmap(beatmap: &Beatmap, root: &Path, resource: &ResourceOut) -> io::Result<()> {
    let basename = PathBuf::from(beatmap.make_basename());
    let filename: PathBuf = [root, &basename].iter().collect();

    let mut osu_file = OsuFile::new(14);
    let mut metadata = Metadata::new();
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
