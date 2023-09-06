use std::{io, path::Path};

use tempfile::{tempdir, TempDir};

use super::super::{resource::ResourceOut, traits::ToOsu, types::Package};

impl ToOsu for Package {
    fn to_osu(&self, path: &Path) -> io::Result<()> {
        let temp_dir: TempDir = tempdir()?;

        // Remap and settle resources.
        let mut resource_out = ResourceOut::new();
        resource_out.inflate(temp_dir.path().to_owned(), &self.resource_pool)?;

        // TODO: Compile beatmaps.

        // TODO: Package all files to a bundle.

        Ok(())
    }
}
