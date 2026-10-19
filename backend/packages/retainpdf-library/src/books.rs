use std::path::{Path, PathBuf};

use super::LibraryDeps;
use retainpdf_data::db::Db;
use retainpdf_api_base::error::AppError;
use retainpdf_jobs::ListJobsQuery;
use crate::views::{LibraryBookDetailView, LibraryBookListView, LibraryDeleteResultView};
use retainpdf_core::models::domain::{JobSnapshot, JobStatusKind, WorkflowKind};
use super::book_projection::{
    build_library_book_detail_view, build_library_book_list_view,
};
use retainpdf_jobs::cleanup_deleted_job_credentials;

pub fn list_library_books(
    db: &Db,
    data_root: &Path,
    query: &ListJobsQuery,
    base_url: &str,
) -> Result<LibraryBookListView, AppError> {
    build_library_book_list_view(db, data_root, query, base_url)
}

pub fn get_library_book(
    db: &Db,
    data_root: &Path,
    job_id: &str,
    base_url: &str,
) -> Result<LibraryBookDetailView, AppError> {
    let job = load_library_job(db, job_id)?;
    Ok(build_library_book_detail_view(
        db, data_root, &job, base_url,
    ))
}

pub fn delete_library_book(
    deps: &LibraryDeps<'_>,
    job_id: &str,
    force: bool,
) -> Result<LibraryDeleteResultView, AppError> {
    let jobs = book_delete_jobs(deps.db, job_id)?;
    for job in &jobs {
        ensure_deletable(job, force)?;
    }
    ensure_no_dependents(deps.db, &jobs, force)?;

    // 删 job 前先记住它所属文档,删后 reconcile 其 active_job_id,
    // 否则文档行会悬空指向已删 job,前端 join 不到而渲染成僵尸卡。
    // 读不出来是 500，不能当成「这个任务不属于任何书」悄悄跳过对账。
    let affected_document = deps
        .db
        .get_document_by_job_id(job_id)?
        .map(|doc| doc.document_id);

    // 先一次性提交行删除,再动磁盘。逐个 `remove_job_files` + `delete_job`
    // 自动提交时,book job 删完而 -ocr 子 job 那步失败就留下一个没有父任务的
    // 孤儿行,没有任何自动修复路径。反过来,事务提交后文件删失败只是留下可
    // 重删的孤儿目录。
    let job_ids: Vec<String> = jobs.iter().map(|job| job.job_id.clone()).collect();
    deps.db.delete_jobs(&job_ids)?;

    let mut removed_paths = Vec::new();
    let mut removed_child_jobs = Vec::new();
    for job in &jobs {
        removed_paths.extend(remove_job_files(deps, &job.job_id)?);
        if job.job_id != job_id {
            removed_child_jobs.push(job.job_id.clone());
        }
    }
    cleanup_deleted_job_credentials(deps.db, deps.data_root, &jobs);

    if let Some(document_id) = affected_document {
        // reconcile 失败不阻断删除(job 已删),仅记录——文档最坏保持悬空,
        // 下次任一 job 删除或启动回填会再修。
        if let Err(error) = deps.db.reconcile_document_active_job(&document_id) {
            eprintln!(
                "[library] reconcile active_job for {document_id} after deleting {job_id} failed: {error}"
            );
        }
        // 被删任务的全文索引行已在删除事务里清掉;它独占的页之外,别的成功任务覆盖的页
        // 要按剩下的任务重建回来。索引是可重建的派生数据,失败只记日志。
        if let Err(error) = deps.db.rebuild_document_fts(&document_id) {
            eprintln!(
                "[library] rebuild fts for {document_id} after deleting {job_id} failed: {error}"
            );
        }
    }

    Ok(LibraryDeleteResultView {
        deleted: true,
        job_id: job_id.to_string(),
        removed_paths,
        removed_child_jobs,
    })
}

/// 删一本书会动到的任务:book 任务本身 + 它的 `-ocr` 子任务。找不到书 → 404。
fn book_delete_jobs(db: &Db, job_id: &str) -> Result<Vec<JobSnapshot>, AppError> {
    let mut jobs = vec![load_library_job(db, job_id)?];
    if let Some(child) = find_job(db, &format!("{job_id}-ocr"))? {
        jobs.push(child);
    }
    Ok(jobs)
}

/// 强制删除前要先取消的任务编号(删书会动到的全部任务)。路由在 `force=true` 时用它，
/// 经任务门面走和 `POST actions/cancel` 同一条路径，之后才调 [`delete_library_book`]。
pub fn book_delete_job_ids(db: &Db, job_id: &str) -> Result<Vec<String>, AppError> {
    Ok(book_delete_jobs(db, job_id)?
        .into_iter()
        .map(|job| job.job_id.clone())
        .collect())
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

/// 按编号找一个可能不存在的任务：没有这条记录是 `None`；有记录但读不出来是 500，
/// 不能当成「没有」跳过（删除时跳过一个在跑的任务，就绕过了 `ensure_deletable`）。
pub(super) fn find_job(db: &Db, job_id: &str) -> Result<Option<JobSnapshot>, AppError> {
    match db.get_job(job_id) {
        Ok(job) => Ok(Some(job)),
        Err(error) if AppError::is_record_missing(&error) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// 名下还有排队 / 在跑的任务：409 + `job_active`（和任务操作同一个原因），带 `force=true` 先取消再删。
pub(super) fn ensure_deletable(job: &JobSnapshot, force: bool) -> Result<(), AppError> {
    if !force && matches!(job.status, JobStatusKind::Queued | JobStatusKind::Running) {
        return Err(AppError::conflict_because("job_active", format!(
            "book is {:?}; pass force=true to delete it",
            job.status
        )));
    }
    Ok(())
}

/// 删除范围之外的任务还要用范围内任务的产物时，不带 `force` 拒绝(409 + `job_has_dependents`)。
///
/// 典型：重新排版产生的 render 任务，译文目录就是源任务的 `translated/`。源任务一删，
/// 这本书的合并阅读(`/documents/:id/reading`)会因为找不到译文目录整个 500。
pub(super) fn ensure_no_dependents(
    db: &Db,
    jobs: &[JobSnapshot],
    force: bool,
) -> Result<(), AppError> {
    if force {
        return Ok(());
    }
    let ids: Vec<String> = jobs.iter().map(|job| job.job_id.clone()).collect();
    let dependents = db.jobs_depending_on(&ids)?;
    if dependents.is_empty() {
        return Ok(());
    }
    let list = dependents
        .iter()
        .map(|(dependent, target)| format!("{dependent} (uses {target})"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(AppError::conflict_because(
        "job_has_dependents",
        format!("other jobs still use this job's artifacts: {list}; delete them first or pass force=true"),
    ))
}

pub(super) fn remove_job_files(
    deps: &LibraryDeps<'_>,
    job_id: &str,
) -> Result<Vec<String>, AppError> {
    let mut removed = Vec::new();
    remove_path_if_exists(deps.output_root.join(job_id), &mut removed)?;
    remove_path_if_exists(
        deps.downloads_dir.join(format!("{job_id}.zip")),
        &mut removed,
    )?;
    Ok(removed)
}

pub(super) fn remove_path_if_exists(
    path: PathBuf,
    removed: &mut Vec<String>,
) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path)?;
    } else {
        std::fs::remove_file(&path)?;
    }
    removed.push(path.to_string_lossy().to_string());
    Ok(())
}
