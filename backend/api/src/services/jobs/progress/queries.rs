use crate::error::AppError;
use crate::models::api::{JobEventListView, ListJobEventsQuery};

use crate::job_loading::load_supported_job;
use crate::services::jobs::query::JobQueries;

impl JobQueries<'_> {
    pub fn live_translation_events_after(
        &self,
        job_id: &str,
        after_seq: i64,
        limit: u32,
    ) -> Result<Vec<crate::models::api::LiveTranslationCommitEventView>, AppError> {
        load_supported_job(self.db, self.data_root, job_id)?;
        crate::services::jobs::progress::live_translation::load_live_translation_events_after(
            self.db,
            job_id,
            after_seq.max(0),
            limit,
        )
    }

    pub fn job_events_view(
        &self,
        job_id: &str,
        query: &ListJobEventsQuery,
        ocr_only: bool,
    ) -> Result<JobEventListView, AppError> {
        crate::services::jobs::progress::event_feed::read_events(self.db, self.data_root, job_id, query, ocr_only)
    }
}
