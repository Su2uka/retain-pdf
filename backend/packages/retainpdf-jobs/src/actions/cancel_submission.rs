use retainpdf_api_base::error::AppError;
use crate::views::JobSubmissionView;

use crate::actions::cancel::cancel_job as cancel_job_service;
use retainpdf_api_base::job_loading::load_job_or_404;
use crate::support::ensure_cancelable;
use crate::facade::JobsFacade;

impl<'a> JobsFacade<'a> {
    pub async fn cancel_submission(
        &self,
        base_url: &str,
        job_id: &str,
    ) -> Result<JobSubmissionView, AppError> {
        let job = load_job_or_404(self.command.db, job_id)?;
        ensure_cancelable(&job)?;
        let job = cancel_job_service(&self.command.control, job_id).await?;
        // 返回库里的真实状态，不写死 canceled：任务可能在取消前一刻自己跑完了（succeeded）；
        // OCR 任务过了排队阶段时这里只杀进程，终态由 runner 稍后落，此刻还是 running。
        Ok(self.build_submission_view(base_url, &job, job.status.clone(), job.workflow.clone()))
    }
}
