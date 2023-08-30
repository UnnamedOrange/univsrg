use std::{io, path::Path};

use tempfile::{tempdir, TempDir};

use super::super::{traits::ToOsu, types::Package};

impl ToOsu for Package {
    fn to_osu(&self, path: &Path) -> io::Result<()> {
        let temp_dir: TempDir = tempdir()?;
        // TODO: Remap and settle resources.
        // TODO: Compile beatmaps.
        // TODO: Package all files to a bundle.
        Ok(())
    }
}
