//! 运行期巡检：库里是 queued/running，进程内却没有 driver 在管的任务。
//!
//! 发射可能丢（壳转发给 jobsd 失败、driver 出错后落终态的那次写也失败、进程内 panic），丢了之后
//! 行就停在 queued/running，原先只有下一次启动才会被 `startup_recovery` 收拾。这里让拥有 driver
//! 登记表的那个进程（进程内模式是壳，远端模式是 jobsd）定期扫一遍，用和启动时同一套规则处理：
//! - running 且 worker 已死 / 没记 pid / 成了孤儿：终止孤儿，有可续的 durable attempt 就回 queued，
//!   否则落失败（`reconcile_running_jobs_matching`）；
//! - queued 且有可续的 attempt，或根本没起过 attempt：重新发射（`requeue_stuck_queued_jobs_matching`）。
//!
//! 「没有 driver」以登记表为准；`DRIVERLESS_GRACE_SECS` 的宽限期只为避开刚落库、发射还在路上
//! （壳到 jobsd 的 HTTP 转发）的任务。扫描只读几次按状态过滤的查询，没有任务时开销可以忽略。

use std::time::Duration;

use anyhow::Result;
use tracing::{error, info, warn};

use crate::models::domain::{seconds_since_iso, JobStatusKind};

use super::{
    lifecycle::spawn_job,
    startup_recovery::{reconcile_running_jobs_matching, requeue_stuck_queued_jobs_matching},
    ProcessRuntimeDeps,
};

/// 多久扫一次。
pub const DRIVERLESS_SWEEP_INTERVAL_SECS: u64 = 30;
/// 行上 `updated_at` 距现在超过多久，才算「没有 driver」而不是「刚发射还没到」。
pub const DRIVERLESS_GRACE_SECS: u64 = 120;

/// 起一个后台任务，每 [`DRIVERLESS_SWEEP_INTERVAL_SECS`] 秒巡检一次，直到进程退出。
pub fn spawn_driverless_job_sweeper(deps: ProcessRuntimeDeps) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(DRIVERLESS_SWEEP_INTERVAL_SECS));
        // 第一拍立即触发；启动时的恢复刚做完，跳过它。
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let sweep_deps = deps.clone();
            match tokio::task::spawn_blocking(move || {
                sweep_driverless_jobs(&sweep_deps, Duration::from_secs(DRIVERLESS_GRACE_SECS))
            })
            .await
            {
                Ok(Ok(0)) => {}
                Ok(Ok(count)) => info!("driverless job sweep handled {count} job(s)"),
                Ok(Err(err)) => error!("driverless job sweep failed: {err:#}"),
                Err(err) => error!("driverless job sweep panicked: {err}"),
            }
        }
    })
}

/// 巡检一次，返回处理（回收或重新发射）的任务数。需要在 tokio 运行时里调用（发射要 spawn）。
pub fn sweep_driverless_jobs(deps: &ProcessRuntimeDeps, grace: Duration) -> Result<usize> {
    sweep_driverless_jobs_with(deps, grace, |deps, job_id| spawn_job(deps, job_id))
}

pub(crate) fn sweep_driverless_jobs_with(
    deps: &ProcessRuntimeDeps,
    grace: Duration,
    spawn: impl Fn(ProcessRuntimeDeps, String),
) -> Result<usize> {
    let drivers = &deps.job_drivers;
    // `{父}-ocr` 子任务由父任务的 driver 内联执行，父任务有 driver 它就算有。
    let has_driver = |job_id: &str| {
        drivers.is_live(job_id)
            || job_id
                .strip_suffix("-ocr")
                .is_some_and(|parent| drivers.is_live(parent))
    };
    let idle_long_enough = |updated_at: &str| {
        seconds_since_iso(updated_at).is_some_and(|secs| secs >= grace.as_secs() as i64)
    };
    let mut handled = 0usize;

    // 1. running 没有 driver：同启动恢复。结果可能是 failed / succeeded / 回到 queued 等重新发射。
    handled += reconcile_running_jobs_matching(
        &deps.config,
        &deps.db,
        &|record| !has_driver(&record.job_id) && idle_long_enough(&record.updated_at),
        false,
    )?;

    // 2. queued 且有可续 attempt（含上一步刚回到 queued 的）：重新发射。
    for job_id in deps.db.list_resumable_pipeline_job_ids()? {
        if has_driver(&job_id) || !queued_idle_long_enough(deps, &job_id, &idle_long_enough) {
            continue;
        }
        warn!("driverless sweep: relaunching resumable queued job {job_id}");
        spawn(deps.clone(), job_id);
        handled += 1;
    }

    // 3. queued 且从没起过 attempt：同启动时的 requeue。
    let stuck = requeue_stuck_queued_jobs_matching(&deps.config, &deps.db, &|job_id| {
        !has_driver(job_id) && queued_idle_long_enough(deps, job_id, &idle_long_enough)
    })?;
    for job_id in stuck {
        warn!("driverless sweep: relaunching stuck queued job {job_id}");
        spawn(deps.clone(), job_id);
        handled += 1;
    }
    Ok(handled)
}

fn queued_idle_long_enough(
    deps: &ProcessRuntimeDeps,
    job_id: &str,
    idle_long_enough: &dyn Fn(&str) -> bool,
) -> bool {
    match deps.db.get_job(job_id) {
        Ok(job) => job.status == JobStatusKind::Queued && idle_long_enough(&job.updated_at),
        Err(err) => {
            warn!("driverless sweep: cannot read job {job_id}: {err:#}");
            false
        }
    }
}
