pub mod app;
pub mod auth;
pub mod error;
pub(crate) mod job_loading;
pub mod routes;
pub(crate) mod runtime;
pub mod services;

pub use retain_pdf_core::{config, job_failure, models, storage_paths};
pub use retain_pdf_data::{db, job_events, ocr_provider, worker_command};
pub use retain_pdf_runner::job_runner;
pub use retain_pdf_proc as process;

#[cfg(test)]
mod api_tests;

#[cfg(test)]
pub(crate) mod test_support;

pub use app::{
    build_app, build_simple_app, build_state, run_servers, run_servers_with_shutdown,
    spawn_servers, AppState, RunningServers,
};
