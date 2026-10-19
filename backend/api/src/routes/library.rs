//! 书架（冻结）：列表和删除的旧地址。

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use crate::auth::Principal;
use crate::error::AppError;
use crate::models::api::{
    ApiResponse, LibraryBookListView, LibraryDeleteQuery, LibraryDeleteResultView, ListJobsQuery,
};
use crate::routes::common::request_base_url;
use crate::routes::common::{
    build_jobs_route_deps, build_library_route_deps, jobs_facade, ok_json, run_read_query_once,
    ApiPath, ApiQuery,
};
use crate::services::library::{
    book_delete_job_ids_view, delete_library_book_view, list_library_books_view,
};
use crate::AppState;

pub async fn list_books(
    State(state): State<AppState>,
    principal: Principal,
    headers: HeaderMap,
    ApiQuery(mut query): ApiQuery<ListJobsQuery>,
) -> Result<Json<ApiResponse<LibraryBookListView>>, AppError> {
    query.owner = principal.owner_filter().map(str::to_string);
    let deps = build_library_route_deps(&state);
    let base_url = request_base_url(&headers, deps.default_port, &deps.bind_host);
    let view = run_read_query_once(&state, "library:books:list".into(), move |db, root| {
        list_library_books_view(db, root, &query, &base_url)
    })
    .await?;
    Ok(ok_json(view))
}

/// 旧地址，新地址 `DELETE /api/v1/jobs/:job_id`（`routes/jobs/lifecycle.rs::delete_job`）；行为不变，OCR 任务照旧 404。
pub async fn delete_book(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    ApiQuery(query): ApiQuery<LibraryDeleteQuery>,
) -> Result<Json<ApiResponse<LibraryDeleteResultView>>, AppError> {
    let deps = build_library_route_deps(&state);
    if query.force {
        // 强删运行中的书：先走取消的同一条路径（登记取消 + 终止 worker），再删。
        let job_ids = book_delete_job_ids_view(&deps.library, &job_id)?;
        jobs_facade(build_jobs_route_deps(&state))
            .cancel_before_delete(&job_ids)
            .await;
    }
    Ok(ok_json(delete_library_book_view(
        &deps.library,
        &job_id,
        query.force,
    )?))
}
