mod control;
mod creation;
mod debug;
mod deps;
mod downloads;
pub(crate) mod event_feed;
mod facade;
pub mod launcher;
pub(super) mod live_stage;
mod live_translation;
pub(crate) mod managed_credential_gc;
pub(crate) mod ocr_artifact_reuse;
mod presentation;
mod query;
mod reader_regions;
mod readiness;
pub mod runtime_gateway;
pub mod snapshot_factory;
mod stage_plan;
pub(crate) mod stage_view;
pub(super) mod summary_loaders;
mod support;
pub(crate) mod translation_request_recovery;
pub mod validation;

pub use control::wait_for_terminal_job;
pub(crate) use deps::{
    CommandJobsDeps, ControlDeps, JobSubmitDeps, QueryJobsDeps, ReplayDeps, SnapshotBuildDeps,
};
pub(crate) use downloads::{DocumentDownloadKind, FileDownload, JobDownloads, MarkdownDownload};
pub(crate) use creation::with_dev_translation_defaults;
#[cfg(test)]
pub(crate) use creation::{DevDefaultsGuard, DevTranslationDefaults};
pub(crate) use facade::build_jobs_facade;
pub use facade::JobsFacade;
pub use query::JobQueries;
pub(crate) use readiness::job_readiness;

pub use validation::{
    validate_mineru_upload_limits, validate_ocr_provider_request, validate_provider_credentials,
};
