//! 入口：上传。建任务的 `submit_job` 暂时还在 `jobs/lifecycle/creation/`，拆包时再挪过来。

mod uploads;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use uploads::{UploadedPdfInput, UploadError, UploadService, UploadServiceConfig};
pub(crate) use uploads::api::{store_upload, store_upload_view, UploadApiDeps};
