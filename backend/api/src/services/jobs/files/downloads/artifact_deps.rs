use crate::services::files::DerivedArtifactDeps;

use crate::services::jobs::files::downloads::DownloadJobsDeps;

pub(super) fn derived_artifact_deps<'a>(deps: &'a DownloadJobsDeps<'a>) -> DerivedArtifactDeps<'a> {
    DerivedArtifactDeps::with_pipeline_command(deps.python_bin, deps.pipeline_command)
}
