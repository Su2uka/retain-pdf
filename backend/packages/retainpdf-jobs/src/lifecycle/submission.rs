use retainpdf_api_base::error::AppError;
use crate::views::JobSubmissionView;
use retainpdf_core::models::domain::{JobStatusKind, UploadRecord, WorkflowKind};
use retainpdf_core::models::request::CreateJobInput;

use crate::lifecycle::creation::{reject_client_job_id, submit_job, NewJob};
use crate::facade::JobsFacade;
use retainpdf_ingest::UploadedPdfInput;

impl<'a> JobsFacade<'a> {
    /// `POST /jobs`：已经上传过的文件（`source.upload_id`）或已有任务的产物（`source.artifact_job_id`）。
    pub fn create_submission(
        &self,
        base_url: &str,
        request: &CreateJobInput,
    ) -> Result<JobSubmissionView, AppError> {
        reject_client_job_id(request)?;
        let job = submit_job(&self.command.submit, NewJob::new(request.clone()))?;
        Ok(self.build_submission_view(base_url, &job, JobStatusKind::Queued, request.workflow.clone()))
    }

    /// 书库发起的提交（`/documents/:id/translate`、`/ocr`）：请求已经绑好这本书的上传。
    pub fn create_document_submission(
        &self,
        base_url: &str,
        request: &CreateJobInput,
        document_id: &str,
    ) -> Result<JobSubmissionView, AppError> {
        reject_client_job_id(request)?;
        let job = submit_job(
            &self.command.submit,
            NewJob::new(request.clone()).for_document(document_id),
        )?;
        Ok(self.build_submission_view(base_url, &job, JobStatusKind::Queued, request.workflow.clone()))
    }

    /// `POST /translate/bundle`：文件随请求一起传，一步建翻译任务。
    pub async fn create_translation_bundle_submission(
        &self,
        base_url: &str,
        request: CreateJobInput,
        filename: String,
        bytes: Vec<u8>,
        developer_mode: bool,
    ) -> Result<JobSubmissionView, AppError> {
        reject_client_job_id(&request)?;
        // 先存文件：任务建不成，上传也留着（之后可以用 upload_id 再提交）。
        let upload = self
            .store_upload(UploadedPdfInput {
                filename,
                bytes,
                developer_mode,
            })
            .await?;
        if matches!(request.workflow, WorkflowKind::Ocr) {
            return Err(AppError::bad_request("workflow=ocr: upload the PDF with POST /api/v1/uploads, then POST /api/v1/jobs with workflow \"ocr\""));
        }
        let workflow = request.workflow.clone();
        let job = submit_job(&self.command.submit, NewJob::new(request).with_upload(upload))?;
        Ok(self.build_submission_view(base_url, &job, JobStatusKind::Queued, workflow))
    }

    async fn store_upload(&self, file: UploadedPdfInput) -> Result<UploadRecord, AppError> {
        Ok(self.command.submit.uploads.store(file).await?)
    }
}
