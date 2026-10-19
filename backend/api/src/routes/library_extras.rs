//! AI 问答会话历史。
//!
//! 会话遵循"软锚点"原则:引用只存 JSON 快照,不做 job 删除保护。
//!
//! All handlers go through library_api (PR5).

use axum::extract::State;
use axum::Json;

use crate::error::AppError;
use crate::models::api::{
    ApiResponse, AppendMessageInput, ConversationDetailView, ConversationListView,
    ConversationMutationResult, ConversationRecord, CreateConversationInput, ForkConversationInput,
    ListConversationsQuery, MessageRecord, PatchConversationInput,
};
use crate::routes::common::{build_library_route_deps, ok_json, ApiJson, ApiPath, ApiQuery};
use crate::services::library::{
    append_message_view, create_conversation_view, delete_conversation_view, fork_conversation_view,
    get_conversation_view, list_conversations_view, patch_conversation_view,
};
use crate::AppState;

pub async fn create_conversation_route(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<CreateConversationInput>,
) -> Result<Json<ApiResponse<ConversationRecord>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(create_conversation_view(&deps.library, &payload)?))
}

pub async fn list_conversations_route(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<ListConversationsQuery>,
) -> Result<Json<ApiResponse<ConversationListView>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(list_conversations_view(&deps.library, &query)?))
}

pub async fn get_conversation_route(
    State(state): State<AppState>,
    ApiPath(conversation_id): ApiPath<String>,
) -> Result<Json<ApiResponse<ConversationDetailView>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(get_conversation_view(
        &deps.library,
        &conversation_id,
    )?))
}

pub async fn delete_conversation_route(
    State(state): State<AppState>,
    ApiPath(conversation_id): ApiPath<String>,
) -> Result<Json<ApiResponse<ConversationMutationResult>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(delete_conversation_view(
        &deps.library,
        &conversation_id,
    )?))
}

pub async fn patch_conversation_route(
    State(state): State<AppState>,
    ApiPath(conversation_id): ApiPath<String>,
    ApiJson(payload): ApiJson<PatchConversationInput>,
) -> Result<Json<ApiResponse<ConversationRecord>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(patch_conversation_view(
        &deps.library,
        &conversation_id,
        &payload,
    )?))
}

pub async fn append_message_route(
    State(state): State<AppState>,
    ApiPath(conversation_id): ApiPath<String>,
    ApiJson(payload): ApiJson<AppendMessageInput>,
) -> Result<Json<ApiResponse<MessageRecord>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(append_message_view(
        &deps.library,
        &conversation_id,
        payload,
    )?))
}

pub async fn fork_conversation_route(
    State(state): State<AppState>,
    ApiJson(payload): ApiJson<ForkConversationInput>,
) -> Result<Json<ApiResponse<ConversationDetailView>>, AppError> {
    let deps = build_library_route_deps(&state);
    Ok(ok_json(fork_conversation_view(&deps.library, &payload)?))
}
