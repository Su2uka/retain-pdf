use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use crate::auth::Principal;
use crate::error::AppError;
use crate::models::api::{
    ApiResponse, LibraryBookDetailView,
    LibraryBookListView, LibraryDeleteQuery, LibraryDeleteResultView, ListJobsQuery,
};
use crate::routes::common::request_base_url;
use crate::routes::common::{
    build_jobs_route_deps, build_library_route_deps, jobs_facade, ok_json, run_library_blocking,
    run_read_query_once, ApiPath, ApiQuery,
};
use crate::services::library::{
    book_delete_job_ids_view, delete_library_book_view, get_library_book_view,
    list_library_books_view,
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

pub async fn get_book(
    State(state): State<AppState>,
    headers: HeaderMap,
    ApiPath(job_id): ApiPath<String>,
) -> Result<Json<ApiResponse<LibraryBookDetailView>>, AppError> {
    let deps = build_library_route_deps(&state);
    let base_url = request_base_url(&headers, deps.default_port, &deps.bind_host);
    let view = run_read_query_once(
        &state,
        format!("library:books:{job_id}"),
        move |db, root| get_library_book_view(db, root, &job_id, &base_url),
    )
    .await?;
    Ok(ok_json(view))
}

pub async fn delete_book(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    ApiQuery(query): ApiQuery<LibraryDeleteQuery>,
) -> Result<Json<ApiResponse<LibraryDeleteResultView>>, AppError> {
    if query.force {
        // 强删运行中的书：先走取消的同一条路径（登记取消 + 终止 worker），再删。
        let id = job_id.clone();
        let job_ids = run_library_blocking(&state, move |library| book_delete_job_ids_view(library, &id)).await?;
        jobs_facade(build_jobs_route_deps(&state))
            .cancel_before_delete(&job_ids)
            .await;
    }
    // 删库行、删任务目录放进阻塞线程。
    let force = query.force;
    Ok(ok_json(
        run_library_blocking(&state, move |library| delete_library_book_view(library, &job_id, force)).await?,
    ))
}
