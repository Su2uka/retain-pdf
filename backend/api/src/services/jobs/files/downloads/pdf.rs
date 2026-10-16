use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::domain::JobSnapshot;
use crate::services::files::linearized_pdf_or_original as linearize_pdf;

use crate::services::jobs::files::downloads::DownloadJobsDeps;

pub(super) fn linearized_pdf_or_original(
    deps: &DownloadJobsDeps<'_>,
    job: &JobSnapshot,
    input_pdf: &Path,
    label: &str,
) -> Result<PathBuf, AppError> {
    linearize_pdf(deps.data_root, job, input_pdf, label)
}
