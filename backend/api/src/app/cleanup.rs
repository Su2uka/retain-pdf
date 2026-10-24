//! Background retention/cleanup for unbounded-growth DB state and orphaned
//! upload files.
//!
//! Three independent sweeps, all age based and each disabled by setting its
//! retention env var to `0`:
//!
//! - `events` rows for jobs that have reached a terminal state
//!   (`succeeded` / `failed` / `canceled`) are deleted once older than
//!   `RUST_API_EVENTS_RETENTION_DAYS` (default 30). Jobs that are still
//!   `queued` or `running` are never touched by this, regardless of age.
//! - `uploads` rows (and their `uploads_dir/<upload_id>/` directory on disk)
//!   are deleted once older than `RUST_API_ORPHAN_UPLOAD_RETENTION_HOURS`
//!   (default 48) *and* no row in `jobs` references that `upload_id`.
//! - 派生缓存（Word、对照 PDF、线性化 PDF、页预览）：终态任务的和书的，生成超过
//!   `RUST_API_DERIVED_CACHE_RETENTION_DAYS` 天就删（默认 `0` 不清），下次请求重新生成。
//!   认哪些文件在文件包的 `derived_artifacts/retention.rs`，管线写的东西不碰。
//!
//! This module only knows about `Db` plus plain paths, not `AppState` — it
//! is wired up from `app::state` (one-shot sweep at startup) and
//! `app::server` (recurring sweep while the server is running).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use tracing::{info, warn};

use crate::db::Db;
use crate::services::files::{
    prune_document_page_previews, prune_job_derived_cache, DerivedCachePruned,
};

pub use retainpdf_core::config::CleanupConfig;

/// How often the recurring sweep runs once the server is up. Deliberately
/// coarse: this is maintenance, not something latency-sensitive.
/// Env `RUST_API_CLEANUP_INTERVAL_SECS` (default 21600) via
/// `retainpdf_core::config::env_vars::env_u64` — the interval is env-tunable
/// without rebuilding (see `retainpdf_core::config::CleanupConfig`).
fn cleanup_interval_from_config() -> Duration {
    // Single source: retainpdf_core::config::env_vars::env_u64 (env `RUST_API_CLEANUP_INTERVAL_SECS`, default 21600)
    // Also keep AppConfig facet in sync via CleanupConfig.
    let direct = Duration::from_secs(retainpdf_core::config::env_vars::env_u64(
        "RUST_API_CLEANUP_INTERVAL_SECS",
        21600,
    ));
    let via_config = CleanupConfig::from_env().interval;
    debug_assert_eq!(direct, via_config);
    direct
}

#[derive(Clone, Copy, Debug)]
pub struct RetentionSettings {
    pub events_retention_days: u64,
    pub orphan_upload_retention_hours: u64,
    pub derived_cache_retention_days: u64,
}

impl RetentionSettings {
    /// 解析在 `CleanupConfig::from_env`（`0` 是关掉这项清理，不当成没设），启动时 `AppConfig::from_env` 已经读过一遍、
    /// 取值不对就不启动了。
    pub fn from_env() -> Self {
        let config = CleanupConfig::from_env();
        Self {
            events_retention_days: config.events_retention_days,
            orphan_upload_retention_hours: config.orphan_upload_retention_hours,
            derived_cache_retention_days: config.derived_cache_retention_days,
        }
    }
}

pub fn log_startup_settings(settings: &RetentionSettings) {
    log_startup_settings_with_interval(settings, cleanup_interval_from_config());
}

pub fn log_startup_settings_with_interval(settings: &RetentionSettings, interval: Duration) {
    let enabled = |value: u64| if value == 0 { "disabled" } else { "enabled" };
    info!(
        "retention cleanup: events_retention_days={} ({}), orphan_upload_retention_hours={} ({}), derived_cache_retention_days={} ({}), sweep_interval_secs={}",
        settings.events_retention_days,
        enabled(settings.events_retention_days),
        settings.orphan_upload_retention_hours,
        enabled(settings.orphan_upload_retention_hours),
        settings.derived_cache_retention_days,
        enabled(settings.derived_cache_retention_days),
        interval.as_secs(),
    );
}

/// Runs one cleanup sweep (events, orphaned uploads, then derived caches).
/// Safe to call repeatedly - each call re-evaluates retention windows against
/// the current time, and the underlying deletes are idempotent. It does
/// blocking DB and filesystem work: call it from a blocking thread.
pub fn run_cleanup_once(settings: &RetentionSettings, db: &Db, data_root: &Path) -> Result<()> {
    if settings.events_retention_days > 0 {
        let deleted = db.cleanup_expired_events(settings.events_retention_days)?;
        if deleted > 0 {
            info!(
                "retention cleanup: deleted {deleted} event row(s) older than {} day(s) for finished jobs",
                settings.events_retention_days
            );
        }
    }

    if settings.orphan_upload_retention_hours > 0 {
        let orphans = db.cleanup_orphaned_uploads(settings.orphan_upload_retention_hours)?;
        if !orphans.is_empty() {
            let mut removed_dirs = 0usize;
            for upload in &orphans {
                if remove_upload_dir(&upload.stored_path) {
                    removed_dirs += 1;
                }
            }
            info!(
                "retention cleanup: deleted {} orphaned upload record(s) older than {} hour(s), removed {removed_dirs} on-disk upload directory(ies)",
                orphans.len(),
                settings.orphan_upload_retention_hours
            );
        }
    }

    if settings.derived_cache_retention_days > 0 {
        let pruned = prune_derived_caches(settings.derived_cache_retention_days, db, data_root)?;
        if pruned.files > 0 {
            info!(
                "retention cleanup: deleted {} derived cache file(s) ({} bytes) generated more than {} day(s) ago",
                pruned.files, pruned.bytes, settings.derived_cache_retention_days
            );
        }
        if pruned.failed > 0 {
            warn!(
                "retention cleanup: {} expired derived cache file(s) could not be removed; retrying next sweep",
                pruned.failed
            );
        }
    }

    Ok(())
}

fn prune_derived_caches(retention_days: u64, db: &Db, data_root: &Path) -> Result<DerivedCachePruned> {
    let older_than = SystemTime::now()
        .checked_sub(Duration::from_secs(retention_days.saturating_mul(24 * 3600)))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut pruned = prune_document_page_previews(data_root, older_than);
    for job_id in db.terminal_job_ids()? {
        pruned.add(prune_job_derived_cache(data_root, &job_id, older_than));
    }
    Ok(pruned)
}

/// 在阻塞线程里跑一轮清理：删事件、删目录、扫任务目录都是同步的，不能占着 async 工作线程。
pub async fn run_cleanup_blocking(settings: RetentionSettings, db: Arc<Db>, data_root: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || run_cleanup_once(&settings, &db, &data_root)).await?
}

/// `stored_path` is `uploads_dir/<upload_id>/<filename>` (see
/// `services/ingest/uploads/staging.rs`), so its parent is
/// exactly the per-upload directory to remove.
fn remove_upload_dir(stored_path: &str) -> bool {
    let Some(dir) = Path::new(stored_path).parent() else {
        return false;
    };
    match std::fs::remove_dir_all(dir) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            warn!(
                "retention cleanup: failed to remove orphaned upload dir {}: {error}",
                dir.display()
            );
            false
        }
    }
}

/// Spawns the recurring cleanup sweep on the current Tokio runtime. Intended
/// to be called once, right after the server starts listening; the returned
/// handle runs for the lifetime of the process (best-effort maintenance, not
/// joined on shutdown).
pub fn spawn_periodic_cleanup(
    settings: RetentionSettings,
    db: Arc<Db>,
    data_root: PathBuf,
) -> tokio::task::JoinHandle<()> {
    spawn_periodic_cleanup_with_interval(settings, db, data_root, cleanup_interval_from_config())
}

/// Same as `spawn_periodic_cleanup` but with an explicit interval (e.g. from `AppConfig::cleanup`).
/// This is the AppConfig-wired entry point; the zero-arg version above remains as
/// a convenience that reads `RUST_API_CLEANUP_INTERVAL_SECS` directly.
pub fn spawn_periodic_cleanup_with_interval(
    settings: RetentionSettings,
    db: Arc<Db>,
    data_root: PathBuf,
    interval_duration: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(interval_duration);
        // The first tick fires immediately; skip it since a startup sweep
        // already ran in `app::state::build_state`.
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = run_cleanup_blocking(settings, db.clone(), data_root.clone()).await {
                warn!("retention cleanup sweep failed: {error:#}");
            }
        }
    })
}
