//! 上传，以及只做 OCR 的旧别名 `/ocr/jobs/*`（多用户下关闭，内容对应 `/jobs/*`）。

use crate::app::AppState;
use crate::routes::{jobs, uploads};

crate::endpoints! {
    state: AppState;
    /// 上传一个 PDF（不限请求体大小），建任务时用返回的 upload_id
    POST "/api/v1/uploads" => uploads::upload_pdf, signed [unlimited_body];
    /// 带文件建一个只做 OCR 的任务
    POST "/api/v1/ocr/jobs" => jobs::lifecycle::create_ocr_job, closed [unlimited_body] old "/api/v1/jobs";
    GET "/api/v1/ocr/jobs" => jobs::lifecycle::list_ocr_jobs, closed old "/api/v1/jobs";
    GET "/api/v1/ocr/jobs/:job_id" => jobs::lifecycle::get_ocr_job, closed old "/api/v1/jobs/:job_id";
    GET "/api/v1/ocr/jobs/:job_id/events" => jobs::progress::get_ocr_job_events, closed old "/api/v1/jobs/:job_id/progress/events";
    GET "/api/v1/ocr/jobs/:job_id/artifacts" => jobs::files::get_ocr_job_artifacts, closed old "/api/v1/jobs/:job_id/files";
    GET "/api/v1/ocr/jobs/:job_id/artifacts-manifest" => jobs::files::get_ocr_job_artifacts_manifest, closed old "/api/v1/jobs/:job_id/files";
    GET "/api/v1/ocr/jobs/:job_id/artifacts/:artifact_key" => jobs::files::download_ocr_artifact_by_key, closed old "/api/v1/jobs/:job_id/files/internal/*name";
    GET "/api/v1/ocr/jobs/:job_id/normalized-document" => jobs::files::download_ocr_normalized_document, closed old "/api/v1/jobs/:job_id/files/internal/*name";
    GET "/api/v1/ocr/jobs/:job_id/normalization-report" => jobs::files::download_ocr_normalization_report, closed old "/api/v1/jobs/:job_id/files/internal/*name";
    POST "/api/v1/ocr/jobs/:job_id/cancel" => jobs::actions::cancel_ocr_job, closed old "/api/v1/jobs/:job_id/actions/cancel";
}
