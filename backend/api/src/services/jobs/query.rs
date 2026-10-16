//! 只读查询的入口 `JobQueries`。各类的查询方法写在各自目录的 `queries.rs` 里；按编号取任务在 `crate::job_loading`。

#[cfg(test)]
mod tests;

use std::path::Path;

use crate::db::Db;
use crate::error::AppError;
use crate::job_loading::{load_ocr_job_with_supported_layout, load_supported_job};
use crate::models::api::ListJobsQuery;
use crate::models::domain::JobSnapshot;

/// Read-only job views require only persisted state and its artifacts, never a
/// launcher, upload service, or download/replay configuration.
pub struct JobQueries<'a> {
    pub(in crate::services::jobs) db: &'a Db,
    pub(in crate::services::jobs) data_root: &'a Path,
}

impl<'a> JobQueries<'a> {
    pub(crate) fn new(db: &'a Db, data_root: &'a Path) -> Self {
        Self { db, data_root }
    }

    pub(in crate::services::jobs) fn load_supported_job_snapshot(
        &self,
        job_id: &str,
        ocr_only: bool,
    ) -> Result<JobSnapshot, AppError> {
        if ocr_only {
            load_ocr_job_with_supported_layout(self.db, self.data_root, job_id)
        } else {
            load_supported_job(self.db, self.data_root, job_id)
        }
    }
}

pub(super) fn list_jobs_filtered(
    db: &Db,
    query: &ListJobsQuery,
) -> Result<Vec<JobSnapshot>, AppError> {
    Ok(db.select_jobs(
        &crate::db::JobListSelection {
            status: query.status.as_ref(),
            workflow: query.workflow.as_ref(),
            provider: query.provider.as_deref(),
            limit: Some(query.limit.clamp(1, crate::config::limits::MAX_JOB_LIMIT)),
            offset: query.offset,
            owner: query.owner.as_deref(),
            ..Default::default()
        },
        |_, _| true,
    )?)
}

#[cfg(test)]
#[path = "query/list_selection_tests.rs"]
mod list_selection_tests;
