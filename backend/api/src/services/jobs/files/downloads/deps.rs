use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::db::Db;
use crate::services::files::download_generation::DownloadGeneration;

pub(in crate::services::jobs) struct DownloadJobsDeps<'a> {
    pub(in crate::services::jobs) db: &'a Db,
    pub(in crate::services::jobs) data_root: &'a Path,
    pub(in crate::services::jobs) downloads_dir: &'a Path,
    pub(in crate::services::jobs) download_generation: &'a Arc<DownloadGeneration>,
    pub(in crate::services::jobs) python_bin: &'a str,
    pub(in crate::services::jobs) pipeline_command: &'a str,
}

/// An admitted blocking task owns only the resources needed to read or generate
/// its artifact. Its lifetime never captures the application or job executor.
pub(in crate::services::jobs) struct OwnedDownloadJobsDeps {
    db: Db,
    data_root: PathBuf,
    downloads_dir: PathBuf,
    download_generation: Arc<DownloadGeneration>,
    python_bin: String,
    pipeline_command: String,
}

impl DownloadJobsDeps<'_> {
    pub(in crate::services::jobs) fn owned(&self) -> OwnedDownloadJobsDeps {
        OwnedDownloadJobsDeps {
            db: self.db.clone(),
            data_root: self.data_root.into(),
            downloads_dir: self.downloads_dir.into(),
            download_generation: self.download_generation.clone(),
            python_bin: self.python_bin.into(),
            pipeline_command: self.pipeline_command.into(),
        }
    }
}

impl OwnedDownloadJobsDeps {
    pub(in crate::services::jobs) fn borrowed(&self) -> DownloadJobsDeps<'_> {
        DownloadJobsDeps {
            db: &self.db,
            data_root: &self.data_root,
            downloads_dir: &self.downloads_dir,
            download_generation: &self.download_generation,
            python_bin: &self.python_bin,
            pipeline_command: &self.pipeline_command,
        }
    }
}
