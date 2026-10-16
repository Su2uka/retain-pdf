//! 入口：上传。建任务的 `submit_job` 暂时还在 `jobs/lifecycle/creation/`，拆包时再挪过来。

mod uploads;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use uploads::{UploadError, UploadedPdfInput, UploadService, UploadServiceConfig};
pub(crate) use uploads::api::{store_upload, store_upload_view, UploadApiDeps};

use crate::error::AppError;

// 上传只返回自己的 UploadError（uploads/ 里不碰 HTTP），转成对外的 HTTP 错误在领域门面这里做。
impl From<UploadError> for AppError {
    fn from(value: UploadError) -> Self {
        match value {
            UploadError::BadRequest(message) => Self::bad_request(message),
            UploadError::PayloadTooLarge(message) => Self::payload_too_large(message),
            UploadError::Busy => {
                Self::service_unavailable("PDF processing capacity is busy; please retry")
            }
            UploadError::QueueTimeout => {
                Self::service_unavailable("PDF processing queue wait timed out")
            }
            UploadError::RepairUnavailable => {
                Self::service_unavailable("PDF repair tool unavailable")
            }
            UploadError::RepairTimeout => Self::service_unavailable("PDF repair timed out"),
            UploadError::Internal(message) => Self::internal(message),
            UploadError::Io(error) => Self::from(error),
        }
    }
}
