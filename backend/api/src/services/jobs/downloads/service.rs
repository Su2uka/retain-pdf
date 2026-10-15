use crate::error::AppError;
use crate::models::api::MarkdownDocumentView;
use crate::services::jobs::downloads::{
    document_download, markdown_document_view, markdown_download, DocumentDownloadKind,
    FileDownload, MarkdownDownload,
};

use super::ai_board::{ai_board_file_download, ai_board_listing, AiBoardListing};
use super::files::{generation_key, open_job_file, Disposition, JobFile, JobScope};
use super::JobDownloads;

impl<'a> JobDownloads<'a> {
    /// 任务文件的唯一下载入口。读文件、按需生成都放进 `download_generation`：
    /// 同一份产物同时被请求时只生成一次，也不占着异步运行时。
    pub(crate) async fn open(
        &self,
        job_id: &str,
        scope: JobScope,
        file: JobFile,
        disposition: Disposition,
    ) -> Result<FileDownload, AppError> {
        let deps = self.deps.owned();
        let key = generation_key(job_id, scope, &file);
        let job_id = job_id.to_owned();
        self.deps
            .download_generation
            .run(key, move || {
                open_job_file(&deps.borrowed(), &job_id, scope, &file, disposition)
            })
            .await
    }

    /// AI 助手写在 `<job>/ai/` 里的阅读路径、概念图。
    pub(crate) async fn ai_document(
        &self,
        job_id: &str,
        kind: DocumentDownloadKind,
    ) -> Result<FileDownload, AppError> {
        let deps = self.deps.owned();
        let job_id = job_id.to_owned();
        self.deps
            .download_generation
            .run(format!("{job_id}:ai:{}", kind.label()), move || {
                let deps = deps.borrowed();
                let job = crate::services::jobs::query::load_supported_job(
                    deps.db,
                    deps.data_root,
                    &job_id,
                )?;
                document_download(&deps, &job, kind)
            })
            .await
    }

    pub async fn markdown_document(&self, job_id: String) -> Result<MarkdownDownload, AppError> {
        markdown_download(&self.deps, job_id).await
    }

    pub async fn markdown_document_view(
        &self,
        job_id: &str,
        base_url: &str,
    ) -> Result<MarkdownDocumentView, AppError> {
        markdown_document_view(&self.deps, job_id, base_url).await
    }

    pub fn ai_board_listing(&self, job_id: &str) -> Result<AiBoardListing, AppError> {
        ai_board_listing(&self.deps, job_id)
    }

    pub fn ai_board_file_download(
        &self,
        job_id: &str,
        name: &str,
    ) -> Result<FileDownload, AppError> {
        ai_board_file_download(&self.deps, job_id, name)
    }
}
