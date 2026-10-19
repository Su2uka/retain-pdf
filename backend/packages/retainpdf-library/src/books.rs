//! 书架（`/api/v1/library/books`，冻结）：列表，和删除的旧地址。
//!
//! 删除 2026-10-19 挪到任务包（`DELETE /api/v1/jobs/:job_id`，`retainpdf_jobs::JobDeletion`）；这里只多
//! 一条旧行为：工作流是 OCR 的任务不算书，404。书架上的一本书（`GET /library/books/:job_id`）没有调用方，同日删了。

use std::path::Path;

use super::LibraryDeps;
use retainpdf_data::db::Db;
use retainpdf_api_base::error::AppError;
use retainpdf_jobs::{JobDeletion, ListJobsQuery};
use crate::views::{LibraryBookListView, LibraryDeleteResultView};
use retainpdf_core::models::domain::{JobSnapshot, WorkflowKind};
use super::book_projection::build_library_book_list_view;

pub fn list_library_books(
    db: &Db,
    data_root: &Path,
    query: &ListJobsQuery,
    base_url: &str,
) -> Result<LibraryBookListView, AppError> {
    build_library_book_list_view(db, data_root, query, base_url)
}

pub fn delete_library_book(
    deps: &LibraryDeps<'_>,
    job_id: &str,
    force: bool,
) -> Result<LibraryDeleteResultView, AppError> {
    let job = load_library_job(deps.db, job_id)?;
    job_deletion(deps).delete_loaded(job, force)
}

/// 强制删除前要先取消的任务编号(删书会动到的全部任务)。路由在 `force=true` 时用它，
/// 经任务门面走和 `POST actions/cancel` 同一条路径，之后才调 [`delete_library_book`]。
pub fn book_delete_job_ids(deps: &LibraryDeps<'_>, job_id: &str) -> Result<Vec<String>, AppError> {
    let job = load_library_job(deps.db, job_id)?;
    Ok(job_deletion(deps).scope_ids(job))
}

/// 书库里删任务、删任务目录都经任务包的 [`JobDeletion`]。
pub(super) fn job_deletion<'a>(deps: &LibraryDeps<'a>) -> JobDeletion<'a> {
    JobDeletion::new(deps.db, deps.data_root, deps.output_root, deps.downloads_dir)
}

fn load_library_job(db: &Db, job_id: &str) -> Result<JobSnapshot, AppError> {
    let job = db
        .get_job(job_id)
        .map_err(|error| AppError::from_lookup(error, format!("book not found: {job_id}")))?;
    if job.workflow == WorkflowKind::Ocr {
        return Err(AppError::not_found(format!("book not found: {job_id}")));
    }
    Ok(job)
}
