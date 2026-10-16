use crate::models::domain::JobArtifactRecord;
use crate::storage_paths::ARTIFACT_KIND_DIR;

pub fn artifact_is_direct_downloadable(item: &JobArtifactRecord) -> bool {
    item.artifact_kind != ARTIFACT_KIND_DIR
}
