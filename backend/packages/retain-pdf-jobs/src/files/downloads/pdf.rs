use std::path::{Path, PathBuf};

use retain_pdf_api_base::error::AppError;
use retain_pdf_core::models::domain::JobSnapshot;
use retain_pdf_files::linearized_pdf_or_original as linearize_pdf;

use crate::files::downloads::DownloadJobsDeps;

pub(super) fn linearized_pdf_or_original(
    deps: &DownloadJobsDeps<'_>,
    job: &JobSnapshot,
    input_pdf: &Path,
    label: &str,
) -> Result<PathBuf, AppError> {
    linearize_pdf(deps.data_root, job, input_pdf, label)
}
