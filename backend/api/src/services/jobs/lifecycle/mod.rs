//! 任务：建任务（唯一入口 `submit_job`）、校验、发射，删任务时清理凭据。

pub(crate) mod creation;
pub mod launcher;
pub(crate) mod managed_credential_gc;
pub(crate) mod ocr_artifact_reuse;
pub mod snapshot_factory;
pub mod validation;

pub(crate) use creation::with_dev_translation_defaults;
#[cfg(test)]
pub(crate) use creation::{DevDefaultsGuard, DevTranslationDefaults};
