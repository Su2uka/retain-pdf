//! Launcher routes require the application API key. Worker routes instead use
//! a short-lived capability bound to exactly one job; never accept an API key
//! as a worker capability and never accept an upstream URL in a model request.
//! 两组路由的登记表在 `app/router/model_requests.rs`。
use crate::services::providers::{ModelConnection, ModelRequest};
use crate::{
    app::{build_model_requests_api_from_state, AppState},
    error::AppError,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde_json::{json, Value};

fn token(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|s| s.len() == 64)
        .ok_or_else(|| AppError::Unauthorized("worker capability required".into()))
}
pub(crate) async fn issue(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(profile): Json<ModelConnection>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        build_model_requests_api_from_state(&state).issue(&job_id, profile)?,
    ))
}
pub(crate) async fn submit(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ModelRequest>,
) -> Result<Json<Value>, AppError> {
    let operation = build_model_requests_api_from_state(&state)
        .submit(&job_id, token(&headers)?, request)
        .await?;
    Ok(Json(json!(operation)))
}
pub(crate) async fn status(
    State(state): State<AppState>,
    Path((job_id, operation_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let operation = build_model_requests_api_from_state(&state).status(
        &job_id,
        token(&headers)?,
        &operation_id,
    )?;
    Ok(Json(json!(operation)))
}
pub(crate) async fn cancel(
    State(state): State<AppState>,
    Path((job_id, operation_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let changed = build_model_requests_api_from_state(&state).cancel(
        &job_id,
        token(&headers)?,
        &operation_id,
    )?;
    Ok(Json(json!({"changed":changed})))
}
