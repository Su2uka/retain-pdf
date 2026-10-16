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

pub mod actions;
mod ai;
pub mod data;
mod deps;
mod facade;
pub mod files;
pub mod lifecycle;
mod presentation;
pub mod progress;
mod query;
pub(crate) mod query_execution;
pub mod reader;
pub mod reports;
pub mod runtime_gateway;
pub(super) mod summary_loaders;
mod support;
mod translations;

pub use actions::wait_for_terminal_job;
pub(crate) use ai::DocumentDownloadKind;
pub(crate) use deps::{
    CommandJobsDeps, ControlDeps, JobSubmitDeps, QueryJobsDeps, ReplayDeps, SnapshotBuildDeps,
};
pub(crate) use facade::build_jobs_facade;
pub use facade::JobsFacade;
pub(crate) use files::{
    job_readiness, Disposition, FileDownload, FileGroup, JobDownloads, JobFile, JobFilesView,
    JobScope,
};
pub(crate) use lifecycle::with_dev_translation_defaults;
#[cfg(test)]
pub(crate) use lifecycle::{DevDefaultsGuard, DevTranslationDefaults};
pub use query::JobQueries;
pub(crate) use reader::MarkdownDownload;

pub use lifecycle::validation::{
    validate_mineru_upload_limits, validate_ocr_provider_request, validate_provider_credentials,
};
