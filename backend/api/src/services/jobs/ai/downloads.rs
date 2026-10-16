use crate::error::AppError;
use crate::services::jobs::ai::ai_board::{ai_board_file_download, ai_board_listing, AiBoardListing};
use crate::services::jobs::ai::documents::{document_download, DocumentDownloadKind};
use crate::services::jobs::files::downloads::{FileDownload, JobDownloads};

impl<'a> JobDownloads<'a> {
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
                let job = crate::job_loading::load_supported_job(
                    deps.db,
                    deps.data_root,
                    &job_id,
                )?;
                document_download(&deps, &job, kind)
            })
            .await
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
