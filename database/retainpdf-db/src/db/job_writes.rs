use anyhow::Result;
use rusqlite::{params, OptionalExtension, TransactionBehavior};

use crate::models::domain::{JobArtifactRecord, JobSnapshot};
use crate::storage_paths::{collect_job_artifact_entries, normalize_job_paths_for_storage};

use super::Db;

pub(super) struct PreparedJobWrite {
    stored_job: JobSnapshot,
    artifacts_json: Option<String>,
    artifact_entries: Vec<JobArtifactRecord>,
    runtime_json: Option<String>,
    failure_json: Option<String>,
}

pub(super) fn prepare_job_write(db: &Db, job: &JobSnapshot) -> Result<PreparedJobWrite> {
    let mut stored_job = job.clone();
    stored_job.sync_runtime_state();
    normalize_job_paths_for_storage(&db.data_root, &mut stored_job)?;
    let artifacts_json = stored_job
        .artifacts
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let artifact_entries = collect_job_artifact_entries(&stored_job, &db.data_root)?;
    let runtime_json = stored_job
        .runtime
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let failure_json = stored_job
        .failure
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    Ok(PreparedJobWrite {
        stored_job,
        artifacts_json,
        artifact_entries,
        runtime_json,
        failure_json,
    })
}

pub(super) fn persist_prepared_job(
    tx: &rusqlite::Transaction<'_>,
    prepared: PreparedJobWrite,
) -> Result<()> {
    let PreparedJobWrite {
        stored_job,
        artifacts_json,
        artifact_entries,
        runtime_json,
        failure_json,
    } = prepared;
    tx.execute(
        r#"
        INSERT INTO jobs (
            job_id, workflow, status_json, created_at, updated_at, started_at, finished_at,
            upload_id, pid, command_json, request_json, error, stage, stage_detail,
            progress_current, progress_total, log_tail_json, result_json, runtime_json, failure_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(job_id) DO UPDATE SET
            workflow=excluded.workflow,
            status_json=excluded.status_json,
            created_at=excluded.created_at,
            updated_at=excluded.updated_at,
            started_at=excluded.started_at,
            finished_at=excluded.finished_at,
            upload_id=excluded.upload_id,
            pid=excluded.pid,
            command_json=excluded.command_json,
            request_json=excluded.request_json,
            error=excluded.error,
            stage=excluded.stage,
            stage_detail=excluded.stage_detail,
            progress_current=excluded.progress_current,
            progress_total=excluded.progress_total,
            log_tail_json=excluded.log_tail_json,
            result_json=excluded.result_json,
            runtime_json=excluded.runtime_json,
            failure_json=excluded.failure_json
        "#,
        params![
            stored_job.job_id,
            serde_json::to_string(&stored_job.workflow)?,
            serde_json::to_string(&stored_job.status)?,
            stored_job.created_at,
            stored_job.updated_at,
            stored_job.started_at,
            stored_job.finished_at,
            stored_job.upload_id,
            stored_job.pid.map(|v| v as i64),
            serde_json::to_string(&stored_job.command)?,
            serde_json::to_string(&stored_job.request_payload)?,
            stored_job.error,
            stored_job.stage,
            stored_job.stage_detail,
            stored_job.progress_current,
            stored_job.progress_total,
            serde_json::to_string(&stored_job.log_tail)?,
            serde_json::to_string(&stored_job.result)?,
            runtime_json,
            failure_json,
        ],
    )?;
    persist_job_artifacts(tx, &stored_job.job_id, artifacts_json, &artifact_entries)
}

impl Db {
    pub fn save_job(&self, job: &JobSnapshot) -> Result<()> {
        let prepared = prepare_job_write(self, job)?;
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        persist_prepared_job(&tx, prepared)?;
        tx.commit()?;
        Ok(())
    }

    /// 开始新的一次运行：整行落库（同 `save_job`），并在**同一个事务里**把 `run_generation` 加一，
    /// 返回新的代次。建任务、原地重跑（`relaunch_in_place`）走这里；此后上一次运行遗留的 driver
    /// 拿着旧代次做的 CAS 写全部落空。
    ///
    /// 代次只在这里变：`save_job` / `cas_save_job` 的 upsert 都不写这一列，所以启动恢复等别的
    /// 整行写不会把它冲掉。
    pub fn save_job_as_new_run(&self, job: &JobSnapshot) -> Result<u64> {
        let prepared = prepare_job_write(self, job)?;
        let mut conn = self.connect()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        persist_prepared_job(&tx, prepared)?;
        tx.execute(
            "UPDATE jobs SET run_generation = run_generation + 1 WHERE job_id = ?1",
            params![job.job_id],
        )?;
        let generation: i64 = tx.query_row(
            "SELECT run_generation FROM jobs WHERE job_id = ?1",
            params![job.job_id],
            |row| row.get(0),
        )?;
        tx.commit()?;
        Ok(generation.max(0) as u64)
    }

    /// 只读任务当前的运行代次；没有这条任务返回 None。
    pub fn job_run_generation(&self, job_id: &str) -> Result<Option<u64>> {
        let conn = self.connect()?;
        let generation: Option<i64> = conn
            .query_row(
                "SELECT run_generation FROM jobs WHERE job_id = ?1",
                params![job_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(generation.map(|value| value.max(0) as u64))
    }

    /// CAS terminal write: only succeeds if current DB status is in expected_statuses.
    /// Returns true if row was updated, false if already terminal (no-op).
    pub fn cas_save_job(&self, job: &JobSnapshot, expected_statuses: &[&str]) -> Result<bool> {
        let mut stored_job = job.clone();
        stored_job.sync_runtime_state();
        normalize_job_paths_for_storage(&self.data_root, &mut stored_job)?;
        let artifacts_json = stored_job
            .artifacts
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let artifact_entries = collect_job_artifact_entries(&stored_job, &self.data_root)?;
        let runtime_json = stored_job
            .runtime
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let failure_json = stored_job
            .failure
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let mut conn = self.connect()?;
        // BEGIN IMMEDIATE，不能用默认的 DEFERRED：本事务先 SELECT 再写，
        // DEFERRED 会先拿 SHARED 读锁、写时再升级成写锁，而 SQLite 对**锁升级**
        // 不应用 busy_timeout（两个连接都持 SHARED 又都想升级时等待必然死锁，
        // 所以直接返回 SQLITE_BUSY）。表现就是配了 5s 超时却瞬间 "database is
        // locked"。IMMEDIATE 在 BEGIN 时就取写锁，此时 busy_timeout 正常生效。
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Check current status
        let current_row: Option<(String, i64)> = tx
            .query_row(
                "SELECT status_json, run_generation FROM jobs WHERE job_id = ?1",
                params![stored_job.job_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();
        if let Some((current, current_generation)) = current_row {
            let is_expected = expected_statuses.iter().any(|s| {
                let quoted = format!("\"{}\"", s);
                current == quoted || current == *s
            });
            if !is_expected {
                return Ok(false);
            }
            // 状态对得上还不够：行可能已经被原地重跑重新排队（状态又是 queued），而写的人
            // 还是上一次运行遗留的 driver。代次不同就是别人的运行，一律不写。
            if stored_job.run_generation != current_generation.max(0) as u64 {
                return Ok(false);
            }
        }
        tx.execute(
            r#"
            INSERT INTO jobs (
                job_id, workflow, status_json, created_at, updated_at, started_at, finished_at,
                upload_id, pid, command_json, request_json, error, stage, stage_detail,
                progress_current, progress_total, log_tail_json, result_json, runtime_json, failure_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(job_id) DO UPDATE SET
                workflow=excluded.workflow,
                status_json=excluded.status_json,
                created_at=excluded.created_at,
                updated_at=excluded.updated_at,
                started_at=excluded.started_at,
                finished_at=excluded.finished_at,
                upload_id=excluded.upload_id,
                pid=excluded.pid,
                command_json=excluded.command_json,
                request_json=excluded.request_json,
                error=excluded.error,
                stage=excluded.stage,
                stage_detail=excluded.stage_detail,
                progress_current=excluded.progress_current,
                progress_total=excluded.progress_total,
                log_tail_json=excluded.log_tail_json,
                result_json=excluded.result_json,
                runtime_json=excluded.runtime_json,
                failure_json=excluded.failure_json
            "#,
            params![
                stored_job.job_id,
                serde_json::to_string(&stored_job.workflow)?,
                serde_json::to_string(&stored_job.status)?,
                stored_job.created_at,
                stored_job.updated_at,
                stored_job.started_at,
                stored_job.finished_at,
                stored_job.upload_id,
                stored_job.pid.map(|v| v as i64),
                serde_json::to_string(&stored_job.command)?,
                serde_json::to_string(&stored_job.request_payload)?,
                stored_job.error,
                stored_job.stage,
                stored_job.stage_detail,
                stored_job.progress_current,
                stored_job.progress_total,
                serde_json::to_string(&stored_job.log_tail)?,
                serde_json::to_string(&stored_job.result)?,
                runtime_json,
                failure_json,
            ],
        )?;
        persist_job_artifacts(&tx, &stored_job.job_id, artifacts_json, &artifact_entries)?;
        tx.commit()?;
        Ok(true)
    }
}

fn persist_job_artifacts(
    tx: &rusqlite::Transaction<'_>,
    job_id: &str,
    artifacts_json: Option<String>,
    artifact_entries: &[JobArtifactRecord],
) -> Result<()> {
    if let Some(artifacts_json) = artifacts_json {
        tx.execute(
            r#"
            INSERT INTO artifacts (job_id, artifacts_json)
            VALUES (?1, ?2)
            ON CONFLICT(job_id) DO UPDATE SET
                artifacts_json=excluded.artifacts_json
            "#,
            params![job_id, artifacts_json],
        )?;
    } else {
        tx.execute("DELETE FROM artifacts WHERE job_id = ?1", params![job_id])?;
    }
    tx.execute(
        "DELETE FROM job_artifact_entries WHERE job_id = ?1",
        params![job_id],
    )?;
    for item in artifact_entries {
        tx.execute(
            r#"
            INSERT INTO job_artifact_entries (
                job_id, artifact_key, artifact_group, artifact_kind, relative_path,
                file_name, content_type, ready, size_bytes, checksum, source_stage,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                item.job_id,
                item.artifact_key,
                item.artifact_group,
                item.artifact_kind,
                item.relative_path,
                item.file_name,
                item.content_type,
                if item.ready { 1 } else { 0 },
                item.size_bytes.map(|value| value as i64),
                item.checksum,
                item.source_stage,
                item.created_at,
                item.updated_at,
            ],
        )?;
    }
    Ok(())
}
