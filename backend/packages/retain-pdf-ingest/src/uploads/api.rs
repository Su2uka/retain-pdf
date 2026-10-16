use super::{UploadService, UploadedPdfInput};
use retain_pdf_api_base::error::AppError;
use retain_pdf_accounts::Principal;
use retain_pdf_core::models::api::{upload_to_response, UploadView};
use retain_pdf_core::models::domain::UploadRecord;

pub struct UploadApiDeps<'a> {
    uploads: &'a UploadService,
}

impl<'a> UploadApiDeps<'a> {
    pub fn new(uploads: &'a UploadService) -> Self {
        Self { uploads }
    }
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
