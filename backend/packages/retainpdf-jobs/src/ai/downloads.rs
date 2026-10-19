use retainpdf_api_base::error::AppError;
use crate::ai::ai_board::{ai_board_file_download, ai_board_listing, AiBoardListing};
use crate::files::downloads::{FileDownload, JobDownloads};

impl<'a> JobDownloads<'a> {
    /// 画板里有什么。读目录、接力都是同步磁盘活（接力最多拷 500 个文件 / 64 MB），
    /// 前端还每几秒轮询一次，所以放到阻塞线程里做，不占 tokio 工作线程。
    pub async fn ai_board_listing(&self, job_id: &str) -> Result<AiBoardListing, AppError> {
        let deps = self.deps.owned();
        let job_id = job_id.to_owned();
        tokio::task::spawn_blocking(move || ai_board_listing(&deps.borrowed(), &job_id))
            .await
            .map_err(|_| AppError::internal("ai board listing task failed"))?
    }

    /// 取画板里的一个文件。和译文 PDF 等下载一样走 `download_generation`：
    /// 阻塞线程里做，同一个文件的并发请求合并成一次。
    pub async fn ai_board_file_download(
        &self,
        job_id: &str,
        name: &str,
    ) -> Result<FileDownload, AppError> {
        let deps = self.deps.owned();
        let job_id = job_id.to_owned();
        let name = name.to_owned();
        self.deps
            .download_generation
            .run(format!("{job_id}:ai:board:{name}"), move || {
                ai_board_file_download(&deps.borrowed(), &job_id, &name)
            })
            .await
    }
}
