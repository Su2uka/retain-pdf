use retainpdf_jobs::JobProgressView;
use retainpdf_core::models::domain::JobSnapshot;
use retainpdf_jobs::{build_job_stage_view, LiveStageSnapshot};

pub(super) struct BookLiveProjection {
    pub stage: Option<String>,
    pub stage_detail: Option<String>,
    pub progress: JobProgressView,
}

pub(super) fn project_live_stage(
    job: &JobSnapshot,
    live_stage: Option<&LiveStageSnapshot>,
) -> BookLiveProjection {
    let stage = build_job_stage_view(job, live_stage);
    BookLiveProjection {
        stage: stage.stage,
        stage_detail: stage.stage_detail,
        progress: stage.progress,
    }
}
