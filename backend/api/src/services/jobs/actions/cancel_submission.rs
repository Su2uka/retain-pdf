use crate::error::AppError;
use crate::models::api::JobSubmissionView;
use crate::models::domain::JobStatusKind;

use crate::services::jobs::actions::cancel::cancel_job as cancel_job_service;
use crate::services::jobs::query::load_job_or_404;
use crate::services::jobs::query::load_ocr_job_or_404;
use crate::services::jobs::support::ensure_cancelable;
use crate::services::jobs::facade::JobsFacade;

impl<'a> JobsFacade<'a> {
    pub async fn cancel_submission(
        &self,
        base_url: &str,
        job_id: &str,
        ocr_only: bool,
    ) -> Result<JobSubmissionView, AppError> {
        let job = if ocr_only {
            load_ocr_job_or_404(self.command.db, job_id)?
        } else {
            load_job_or_404(self.command.db, job_id)?
        };
        ensure_cancelable(&job)?;
        let job = cancel_job_service(&self.command.control, job_id, ocr_only).await?;
        let status = if ocr_only {
            job.status.clone()
        } else {
            JobStatusKind::Canceled
        };
        Ok(self.build_submission_view(base_url, &job, status, job.workflow.clone()))
    }
}
