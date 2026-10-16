//! 任务的业务逻辑，和 `routes/jobs/` 一样按 9 类分目录（见 `routes/jobs/mod.rs` 的分类表）。
//! 根目录只放各类共用的：门面、依赖、返回视图、只读查询的入口、运行时。

pub mod actions;
mod debug;
mod deps;
mod downloads;
mod facade;
pub mod files;
pub mod lifecycle;
mod presentation;
pub mod progress;
mod query;
pub mod reader;
pub mod runtime_gateway;
pub(super) mod summary_loaders;
mod support;

pub use actions::wait_for_terminal_job;
pub(crate) use deps::{
    CommandJobsDeps, ControlDeps, JobSubmitDeps, QueryJobsDeps, ReplayDeps, SnapshotBuildDeps,
};
pub(crate) use downloads::{
    Disposition, DocumentDownloadKind, FileDownload, FileGroup, JobDownloads, JobFile,
    JobFilesView, JobScope, MarkdownDownload,
};
pub(crate) use lifecycle::with_dev_translation_defaults;
#[cfg(test)]
pub(crate) use lifecycle::{DevDefaultsGuard, DevTranslationDefaults};
pub(crate) use facade::build_jobs_facade;
pub use facade::JobsFacade;
pub use query::JobQueries;
pub(crate) use files::job_readiness;

pub use lifecycle::validation::{
    validate_mineru_upload_limits, validate_ocr_provider_request, validate_provider_credentials,
};
