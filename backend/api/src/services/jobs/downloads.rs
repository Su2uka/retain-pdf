use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::db::Db;
use crate::services::download_generation::DownloadGeneration;

use self::deps::DownloadJobsDeps;

// Keep this file as the small public facade for job downloads. Concrete
// handlers live in submodules so PDF, markdown, preview, and artifact behavior
// can evolve without turning one route helper into another large grab bag.
mod artifact_deps;
mod catalog;
mod deps;
mod ai_board;
mod ai_carryover;
mod documents;
mod files;
mod markdown;
mod paths;
mod pdf;
mod service;

#[cfg(all(test, unix))]
mod ai_carryover_tests;
#[cfg(all(test, unix))]
mod tests;

use documents::document_download;
pub(crate) use documents::DocumentDownloadKind;
pub(crate) use catalog::{job_files_view, FileGroup, JobFilesView};
pub(crate) use files::{Disposition, JobFile, JobScope};
use markdown::{markdown_document_view, markdown_download};

/// Download capabilities contain no launcher, upload, control, or replay state.
pub struct JobDownloads<'a> {
    deps: DownloadJobsDeps<'a>,
}

impl<'a> JobDownloads<'a> {
    pub(crate) fn new(
        db: &'a Db,
        data_root: &'a Path,
        downloads_dir: &'a Path,
        download_generation: &'a Arc<DownloadGeneration>,
        python_bin: &'a str,
        pipeline_command: &'a str,
    ) -> Self {
        Self {
            deps: DownloadJobsDeps {
                db,
                data_root,
                downloads_dir,
                download_generation,
                python_bin,
                pipeline_command,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileDownload {
    pub path: PathBuf,
    pub content_type: String,
    pub download_name: Option<String>,
    /// 不强制下载（照常在浏览器里打开）但给个文件名：`inline; filename=…`。
    /// 译文 PDF、Markdown 这类「查看」和「下载」共用一个端点的，用这个。
    pub inline_name: Option<String>,
    pub job_id_header: Option<String>,
    /// `Cache-Control`。封面、缩略图这种按书不变的图片让浏览器缓存。
    pub cache_control: Option<&'static str>,
}

impl FileDownload {
    pub fn new(
        path: PathBuf,
        content_type: impl Into<String>,
        download_name: Option<String>,
    ) -> Self {
        Self {
            path,
            content_type: content_type.into(),
            download_name,
            inline_name: None,
            job_id_header: None,
            cache_control: None,
        }
    }

    pub fn with_inline_name(mut self, name: impl Into<String>) -> Self {
        self.inline_name = Some(name.into());
        self
    }

    pub fn with_job_id_header(mut self, job_id: impl Into<String>) -> Self {
        self.job_id_header = Some(job_id.into());
        self
    }
}

#[derive(Debug)]
pub struct MarkdownDownload {
    pub job_id: String,
    pub content: String,
}
