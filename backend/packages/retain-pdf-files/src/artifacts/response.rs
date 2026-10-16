use retain_pdf_core::models::domain::JobArtifactRecord;
use retain_pdf_core::storage_paths::ARTIFACT_KIND_DIR;

pub fn artifact_is_direct_downloadable(item: &JobArtifactRecord) -> bool {
    item.artifact_kind != ARTIFACT_KIND_DIR
}
