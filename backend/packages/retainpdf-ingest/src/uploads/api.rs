use super::{UploadService, UploadedPdfInput};
use retainpdf_api_base::error::AppError;
use retainpdf_accounts::Principal;
use super::view::{upload_to_response, UploadView};
use retainpdf_core::models::domain::UploadRecord;

pub struct UploadApiDeps<'a> {
    uploads: &'a UploadService,
}

impl<'a> UploadApiDeps<'a> {
    pub fn new(uploads: &'a UploadService) -> Self {
        Self { uploads }
    }
}

/// 读请求体之前先看磁盘放不放得下（只看下限，文件大小这会儿还不知道）。
pub fn ensure_upload_disk_space(deps: &UploadApiDeps<'_>) -> Result<(), AppError> {
    deps.uploads.ensure_free_space(0).map_err(AppError::from)
}

pub async fn store_upload(
    deps: &UploadApiDeps<'_>,
    filename: String,
    bytes: Vec<u8>,
    developer_mode: bool,
    owner: Principal,
) -> Result<UploadRecord, AppError> {
    deps.uploads
        .store_for(
            UploadedPdfInput {
                filename,
                bytes,
                developer_mode,
            },
            owner,
        )
        .await
        .map_err(AppError::from)
}

pub async fn store_upload_view(
    deps: &UploadApiDeps<'_>,
    filename: String,
    bytes: Vec<u8>,
    developer_mode: bool,
    owner: Principal,
) -> Result<UploadView, AppError> {
    let upload = store_upload(deps, filename, bytes, developer_mode, owner).await?;
    Ok(upload_to_response(&upload))
}
