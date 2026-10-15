use crate::error::AppError;
use crate::models::api::{ArtifactDownloadQuery, LayoutDocxQuery, MarkdownQuery, PagePreviewQuery};
use crate::models::api::ApiResponse;
use crate::services::jobs::{Disposition, DocumentDownloadKind, JobFile};
use crate::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::routes::common::{build_jobs_download_route_deps, ApiPath, ApiQuery};
use crate::routes::download_response::{
    ai_board_file_response, ai_document_response, job_file, markdown_document_response,
    markdown_response, ocr_job_file,
};

// 下载类接口只说"要哪个文件、在浏览器里打开还是下载"，怎么找都在 services::jobs 的
// `open` 里。各接口的打开方式是旧行为，钉在 docs/file-responses.txt 里。

pub async fn download_pdf(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::TranslatedPdf, Disposition::Inline).await
}

pub async fn download_side_by_side_pdf(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::SideBySidePdf, Disposition::Attachment).await
}

pub async fn download_layout_docx(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
    ApiQuery(query): ApiQuery<LayoutDocxQuery>,
) -> Result<Response, AppError> {
    let file = JobFile::translated_docx(&query);
    job_file(&state, &headers, &job_id, file, Disposition::Attachment).await
}

pub async fn download_cover(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::Cover, Disposition::Unnamed).await
}

pub async fn download_thumbnail(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::Thumbnail, Disposition::Unnamed).await
}

pub async fn download_page_preview(
    State(state): State<AppState>,
    ApiPath((job_id, page)): ApiPath<(String, u32)>,
    headers: HeaderMap,
    ApiQuery(query): ApiQuery<PagePreviewQuery>,
) -> Result<Response, AppError> {
    let file = JobFile::page(page, &query);
    job_file(&state, &headers, &job_id, file, Disposition::Unnamed).await
}

pub async fn download_artifact_by_key(
    State(state): State<AppState>,
    ApiPath((job_id, artifact_key)): ApiPath<(String, String)>,
    headers: HeaderMap,
    ApiQuery(query): ApiQuery<ArtifactDownloadQuery>,
) -> Result<Response, AppError> {
    let file = JobFile::from_artifact_key(&artifact_key, query.include_job_dir);
    job_file(&state, &headers, &job_id, file, Disposition::Attachment).await
}

pub async fn download_ocr_artifact_by_key(
    State(state): State<AppState>,
    ApiPath((job_id, artifact_key)): ApiPath<(String, String)>,
    headers: HeaderMap,
    ApiQuery(query): ApiQuery<ArtifactDownloadQuery>,
) -> Result<Response, AppError> {
    let file = JobFile::from_artifact_key(&artifact_key, query.include_job_dir);
    ocr_job_file(&state, &headers, &job_id, file, Disposition::Attachment).await
}

pub async fn download_normalized_document(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let file = JobFile::Registered("normalized_document_json".into());
    job_file(&state, &headers, &job_id, file, Disposition::Unnamed).await
}

pub async fn download_ocr_normalized_document(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let file = JobFile::Registered("normalized_document_json".into());
    ocr_job_file(&state, &headers, &job_id, file, Disposition::Unnamed).await
}

pub async fn download_ai_reading_path(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let deps = build_jobs_download_route_deps(&state);
    ai_document_response(&deps, &headers, &job_id, DocumentDownloadKind::AiReadingPath).await
}

pub async fn download_ai_canvas(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let deps = build_jobs_download_route_deps(&state);
    ai_document_response(&deps, &headers, &job_id, DocumentDownloadKind::AiCanvas).await
}

pub async fn download_normalization_report(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let file = JobFile::Registered("normalization_report_json".into());
    job_file(&state, &headers, &job_id, file, Disposition::Unnamed).await
}

pub async fn download_ocr_normalization_report(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let file = JobFile::Registered("normalization_report_json".into());
    ocr_job_file(&state, &headers, &job_id, file, Disposition::Unnamed).await
}

pub async fn download_markdown(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
    ApiQuery(query): ApiQuery<MarkdownQuery>,
) -> Result<Response, AppError> {
    markdown_response(
        &build_jobs_download_route_deps(&state),
        &headers,
        job_id,
        &query,
    )
    .await
}

pub async fn get_markdown_document(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    markdown_document_response(&build_jobs_download_route_deps(&state), &headers, &job_id).await
}

pub async fn download_markdown_image(
    State(state): State<AppState>,
    ApiPath((job_id, path)): ApiPath<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::OcrImage(path), Disposition::Unnamed).await
}

/// agent 画板里有什么。目录不存在返回空列表，不是 404 —— 画板是叠加层，
/// 「还没往里放过」是正常状态。
///
/// 返回 `Response` 而不写出 `Json<ApiResponse<AiBoardListing>>`：路由只能经 facade 用服务，
/// 写出类型名就得从 services::jobs 导入这个 DTO、再给它开架构白名单。响应体完全一样。
pub async fn list_ai_board(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
) -> Result<Response, AppError> {
    let deps = build_jobs_download_route_deps(&state);
    Ok(Json(ApiResponse::ok(deps.downloads.ai_board_listing(&job_id)?)).into_response())
}

/// 取画板里的一个文件。**这是唯一接受调用方文件名的 agent 产物端点**，
/// 防护（单层名、拒符号链接、解析后仍在目录内）在 services 那边，见
/// `services/jobs/downloads/ai_board.rs`。
pub async fn download_ai_board_file(
    State(state): State<AppState>,
    ApiPath((job_id, name)): ApiPath<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    ai_board_file_response(&build_jobs_download_route_deps(&state), &headers, &job_id, &name).await
}

pub async fn download_bundle(
    State(state): State<AppState>,
    ApiPath(job_id): ApiPath<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    job_file(&state, &headers, &job_id, JobFile::All, Disposition::Attachment).await
}
