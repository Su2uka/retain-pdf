//! 任务本身：有哪些任务、建一个、看一个（`/jobs`、`/jobs/:id`）。
//!
//! 建任务内部都走 services::jobs 的 `submit_job`。

use crate::auth::Principal;
use crate::error::AppError;
use crate::models::api::{ApiResponse, JobDetailView, JobListView, JobSubmissionView, ListJobsQuery};
use crate::models::request::CreateJobInput;
use crate::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde_json::Value;

use crate::routes::common::{
    build_jobs_query_route_deps, build_jobs_route_deps, jobs_facade, ok_json, request_base_url,
    run_job_query_once, ApiJson, ApiPath, ApiQuery,
};

pub async fn create_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiJson(payload): ApiJson<Value>,
) -> Result<Json<ApiResponse<JobSubmissionView>>, AppError> {
    let request = CreateJobInput::from_api_value(payload)
        .map_err(|e| AppError::bad_request(format!("invalid job payload: {e}")))?;
    let deps = build_jobs_route_deps(&state);
    let base_url = request_base_url(&headers, deps.default_port, &deps.bind_host);
    Ok(ok_json(
        jobs_facade(deps).create_submission(&base_url, &request)?,
    ))
}

pub async fn list_jobs(
    State(state): State<AppState>,
    principal: Principal,
    headers: HeaderMap,
    ApiQuery(mut query): ApiQuery<ListJobsQuery>,
) -> Result<Json<ApiResponse<JobListView>>, AppError> {
    query.owner = principal.owner_filter().map(str::to_string);
    let deps = build_jobs_query_route_deps(&state);
    let base_url = request_base_url(&headers, deps.default_port, &deps.bind_host);
    let view = run_job_query_once(&state, "jobs:list".into(), move |jobs| {
        jobs.list_jobs_view(&base_url, &query)
    })
    .await?;
    Ok(ok_json(view))
}

pub async fn get_job(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<JobDetailView>>, AppError> {
    let deps = build_jobs_query_route_deps(&state);
    let base_url = request_base_url(&headers, deps.default_port, &deps.bind_host);
    let view = run_job_query_once(&state, format!("jobs:detail:{job_id}"), move |jobs| {
        jobs.job_detail_view(&base_url, &job_id)
    })
    .await?;
    Ok(ok_json(view))
}
