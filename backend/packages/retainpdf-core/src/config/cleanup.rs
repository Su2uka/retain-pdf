use std::time::Duration;

use super::env_vars::{env_u64, env_u64_allow_zero};

#[derive(Clone, Debug)]
pub struct CleanupConfig {
    pub interval: Duration,
    /// 终态任务的事件保留几天（`RUST_API_EVENTS_RETENTION_DAYS`，默认 30，`0` 不清）。
    pub events_retention_days: u64,
    /// 没被任务引用的上传保留几小时（`RUST_API_ORPHAN_UPLOAD_RETENTION_HOURS`，默认 48，`0` 不清）。
    pub orphan_upload_retention_hours: u64,
}

impl CleanupConfig {
    pub fn from_env() -> Self {
        Self {
            interval: Duration::from_secs(env_u64("RUST_API_CLEANUP_INTERVAL_SECS", 21600)),
            events_retention_days: env_u64_allow_zero("RUST_API_EVENTS_RETENTION_DAYS", 30),
            orphan_upload_retention_hours: env_u64_allow_zero(
                "RUST_API_ORPHAN_UPLOAD_RETENTION_HOURS",
                48,
            ),
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
