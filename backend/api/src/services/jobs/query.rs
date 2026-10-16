//! 只读查询的入口 `JobQueries` 和各类共用的取任务函数。各类的查询方法写在各自目录的 `queries.rs` 里。

#[cfg(test)]
mod tests;

use std::path::Path;

use crate::db::Db;
use crate::error::AppError;
use crate::models::api::ListJobsQuery;
use crate::models::domain::{JobSnapshot, WorkflowKind};
use crate::storage_paths::{
    job_uses_legacy_output_layout, job_uses_legacy_path_storage, LEGACY_JOB_UNSUPPORTED_MESSAGE,
};

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

pub(super) fn load_job_or_404(db: &Db, job_id: &str) -> Result<JobSnapshot, AppError> {
    db.get_job(job_id)
        .map_err(|_| AppError::not_found(format!("job not found: {job_id}")))
}

fn ensure_supported_job_layout(data_root: &Path, job: &JobSnapshot) -> Result<(), AppError> {
    if job_uses_legacy_output_layout(job, data_root) || job_uses_legacy_path_storage(job) {
        return Err(AppError::conflict(LEGACY_JOB_UNSUPPORTED_MESSAGE));
    }
    Ok(())
}

pub(super) fn load_supported_job(
    db: &Db,
    data_root: &Path,
    job_id: &str,
) -> Result<JobSnapshot, AppError> {
    // 合并结果的虚拟 id（`merged-<文档>-<指纹>`）只在读路径上认。写操作走
    // `load_job_or_404`，数据库里查不到它，重试/取消/删除自然被拒绝。
    if crate::services::merge::reading::is_virtual_job_id(job_id) {
        return crate::services::merge::reading::load_virtual_job(db, data_root, job_id);
    }
    let job = load_job_or_404(db, job_id)?;
    ensure_supported_job_layout(data_root, &job)?;
    Ok(job)
}

pub(super) fn load_ocr_job_or_404(db: &Db, job_id: &str) -> Result<JobSnapshot, AppError> {
    let job = load_job_or_404(db, job_id)?;
    if !matches!(job.workflow, WorkflowKind::Ocr) {
        return Err(AppError::not_found(format!("ocr job not found: {job_id}")));
    }
    Ok(job)
}

pub(super) fn load_ocr_job_with_supported_layout(
    db: &Db,
    data_root: &Path,
    job_id: &str,
) -> Result<JobSnapshot, AppError> {
    let job = load_ocr_job_or_404(db, job_id)?;
    ensure_supported_job_layout(data_root, &job)?;
    Ok(job)
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
