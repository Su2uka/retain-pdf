use retainpdf_api_base::error::AppError;
use crate::views::JobSubmissionView;
use retainpdf_core::models::domain::JobStatusKind;

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
        Ok(self.build_submission_view(base_url, &job, JobStatusKind::Canceled, job.workflow.clone()))
    }
}
