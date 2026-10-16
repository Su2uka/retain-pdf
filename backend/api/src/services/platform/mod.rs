//! 平台：多设备同步（`sync`）、数据库备份（`backup`）、字体（`fonts`）、健康检查（`health_api`）。

mod backup;
mod fonts;
mod health_api;
mod sync;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use backup::BackupService;
pub(crate) use backup::api::{
    backup_status_view, create_backup_view, delete_backup_view, restore_backup_view, BackupApiDeps,
};
pub(crate) use fonts::api::{list_fonts, upload_font, FontApiDeps, FontInfo};
pub(crate) use health_api::{
    build_current_readiness_view, build_health_view, HealthApiDeps, HealthView, ReadinessView,
};
pub(crate) use sync::SyncService;
pub(crate) use sync::api::{
    run_sync_now_view, sync_status_view, test_sync_target_view, update_sync_settings_view,
    SyncApiDeps,
};
// 只给测试用的。
#[cfg(test)]
pub(crate) use health_api::build_readiness_view;
