//! 任务的业务逻辑，和 `routes/jobs/`、`app/router/jobs/` 一样按 9 类分目录（分类表见
//! `routes/jobs/mod.rs`）：
//!
//! | 目录 | 类 |
//! |---|---|
//! | `lifecycle/` | 任务：建任务、校验、发射、列表和详情 |
//! | `actions/` | 操作：取消、重跑、重试阶段、续跑计划、OCR 歧义 |
//! | `progress/` | 进度：事件流、当前阶段、边翻边看 |
//! | `files/` | 文件：列出、下载、是否就绪 |
//! | `reports/` | 报告：失败原因、翻译诊断和质检、排版、精修 |
//! | `reader/` | 阅读：区块位置、元信息、Markdown |
//! | `translations/` | 译文：逐条看、人工改、重翻一条 |
//! | `data/` | 数据：按登记表通用取数 |
//! | `ai/` | AI：阅读路径、概念图、画板 |
//!
//! 根目录只放各类共用的：门面 `facade`、依赖 `deps`、返回视图 `presentation`、只读查询的入口
//! `query`、运行时 `runtime_gateway`、产物摘要 `summary_loaders`。

mod actions;
mod ai;
mod data;
mod deps;
mod facade;
mod files;
mod lifecycle;
mod presentation;
mod progress;
mod query;
mod query_execution;
mod reader;
mod reports;
mod runtime_gateway;
mod summary_loaders;
mod support;
mod translations;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use ai::DocumentDownloadKind;
pub(crate) use data::api::{
    job_data_catalog_view, job_data_view, JobDataApiDeps, JobDataCatalogView, JobDataView,
};
pub(crate) use deps::{
    CommandJobsDeps, ControlDeps, JobSubmitDeps, QueryJobsDeps, ReplayDeps, SnapshotBuildDeps,
};
pub(crate) use facade::{build_jobs_facade, JobsFacade};
pub(crate) use files::{
    job_readiness, Disposition, FileDownload, FileGroup, JobDownloads, JobFile, JobFilesView,
    JobScope,
};
pub(crate) use lifecycle::launcher::JobLaunchDeps;
pub(crate) use lifecycle::managed_credential_gc::cleanup_deleted_job_credentials;
pub(crate) use lifecycle::ocr_artifact_reuse::{reusable_ocr_coverage, validate_ocr_artifact_reuse};
pub(crate) use lifecycle::validation::{
    validate_ocr_credential_reference, validate_provider_credentials,
};
pub(crate) use lifecycle::with_dev_translation_defaults;
pub(crate) use progress::live_stage::{
    load_live_stage_snapshot, load_live_stage_snapshots, LiveStageSnapshot,
};
pub(crate) use progress::stage_view::{build_job_stage_view, terminal_completion_note};
pub(crate) use query::JobQueries;
pub(crate) use query_execution::QueryExecution;
pub(crate) use reader::MarkdownDownload;
pub(crate) use reports::quality::api::{
    quality_items_view, quality_summary_view, QualityApiDeps, QualityItemsParams, QualityItemsView,
    QualitySummaryView,
};
pub(crate) use reports::usage::api::{
    all_usage_view, document_usage_view, job_usage_view, UsageApiDeps, UsageSummaryView,
};
pub(crate) use runtime_gateway::{JobDriverRegistry, JobRuntime, JobRuntimeLauncher};
pub(crate) use summary_loaders::SummaryCache;
// 只给测试用的。
#[cfg(test)]
pub(crate) use actions::translation_request_recovery::{
    load_translation_request_recovery, load_translation_request_recovery_with_db,
};
#[cfg(test)]
pub(crate) use lifecycle::validation::validate_translation_credentials;
#[cfg(test)]
pub(crate) use lifecycle::{DevDefaultsGuard, DevTranslationDefaults};
#[cfg(test)]
pub(crate) use reports::usage::job_usage_ledger_path;
