//! 管理后台的平台设置（/api/v1/admin/platform*，只给管理员、只在多用户模式）：多用户模式下所有任务
//! 统一用的翻译模型和 OCR。规则见 services::platform_settings。

use axum::extract::State;
use axum::Json;

use super::accounts::{require_admin, require_multi};
use crate::error::AppError;
use crate::models::api::ApiResponse;
use crate::routes::common::{build_accounts_route_deps, ok_json, ApiJson};
use crate::services::accounts::api::Principal;
use crate::services::platform_settings::{
    self, PlatformSettingsView, PlatformTestView, ProbeRuntimes, SavePlatformSettingsInput,
    TestPlatformInput,
};
use crate::AppState;

fn admin_only(state: &AppState, principal: &Principal) -> Result<(), AppError> {
    require_multi(&build_accounts_route_deps(state))?;
    require_admin(principal)
}

/// GET /api/v1/admin/platform
pub async fn get_platform_route(
    State(state): State<AppState>,
    principal: Principal,
) -> Result<Json<ApiResponse<PlatformSettingsView>>, AppError> {
    admin_only(&state, &principal)?;
    let view = tokio::task::spawn_blocking(move || {
        platform_settings::view(
            &state.db,
            &state.config.data_root,
            &state.config.accounts.platform,
        )
    })
    .await
    .map_err(|_| AppError::internal("platform settings task failed"))??;
    Ok(ok_json(view))
}

/// PUT /api/v1/admin/platform：保存后下一个任务就用新的，不用重启。
pub async fn put_platform_route(
    State(state): State<AppState>,
    principal: Principal,
    ApiJson(input): ApiJson<SavePlatformSettingsInput>,
) -> Result<Json<ApiResponse<PlatformSettingsView>>, AppError> {
    admin_only(&state, &principal)?;
    let view = tokio::task::spawn_blocking(move || {
        platform_settings::save(
            &state.db,
            &state.config.data_root,
            &state.config.accounts.platform,
            &state.config.provider_runtime.deepseek,
            &principal.user_id,
            input,
        )
    })
    .await
    .map_err(|_| AppError::internal("platform settings task failed"))??;
    Ok(ok_json(view))
}

/// POST /api/v1/admin/platform/test：用传进来的值（没传的用已保存的）实际连一次。
pub async fn test_platform_route(
    State(state): State<AppState>,
    principal: Principal,
    ApiJson(input): ApiJson<TestPlatformInput>,
) -> Result<Json<ApiResponse<PlatformTestView>>, AppError> {
    admin_only(&state, &principal)?;
    let runtimes = ProbeRuntimes {
        deepseek: state.config.provider_runtime.deepseek.clone(),
        mineru: state.config.provider_runtime.mineru.clone(),
        paddle: state.config.provider_runtime.paddle.clone(),
    };
    let view = platform_settings::test(
        &state.db,
        &state.config.data_root,
        &state.config.accounts.platform,
        &runtimes,
        input,
    )
    .await?;
    Ok(ok_json(view))
}
