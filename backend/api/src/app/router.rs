use axum::middleware;
use axum::Router;
use axum::http::HeaderValue;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::app::AppState;
use crate::auth;
use crate::config::AccountsConfig;
use crate::routes::common::{method_not_allowed, unknown_route};

mod accounts;
mod ai;
mod collections;
mod credentials;
mod documents;
mod fonts;
mod glossaries;
mod backups;
mod sync;
mod ingestion;
mod internal_agent;
mod jobs;
mod library;
mod model_requests;
mod providers;
mod public;
mod terminal;

/// 登记表里的全部接口（`app/endpoints.rs`）：访问控制按它查谁能用，接口目录和接口清单快照也从这里汇总。
pub(crate) fn registered_endpoints() -> Vec<crate::app::endpoints::Endpoint> {
    let mut tables = vec![
        public::ENDPOINTS,
        accounts::ENDPOINTS,
        credentials::ENDPOINTS,
        providers::ENDPOINTS,
        ingestion::ENDPOINTS,
    ];
    tables.extend(jobs::endpoints());
    tables.extend([
        documents::ENDPOINTS,
        library::ENDPOINTS,
        collections::ENDPOINTS,
        glossaries::ENDPOINTS,
        fonts::ENDPOINTS,
        sync::ENDPOINTS,
        backups::ENDPOINTS,
        ai::ENDPOINTS,
        internal_agent::ENDPOINTS,
        model_requests::launcher::ENDPOINTS,
        model_requests::worker::ENDPOINTS,
        terminal::ENDPOINTS,
    ]);
    tables.into_iter().flatten().copied().collect()
}

pub fn build_app(state: AppState) -> Router {
    // 多用户模式下没有终端：助手终端能在服务器上执行命令，只给单机版。
    let websocket_routes = if state.config.accounts.mode.is_multi() {
        Router::new()
    } else {
        terminal::routes()
    };
    public::routes()
        .method_not_allowed_fallback(method_not_allowed)
        .merge(authenticated_api_routes(&state))
        .merge(websocket_routes)
        .merge(model_requests::worker::routes())
        .fallback(unknown_route)
        // 计端点使用量。挂在 CORS/Trace 之下、路由之上,这样能从 `MatchedPath`
        // 拿到路由模板而不是具体 URL。见 `route_usage`：删端点之前先量。
        .layer(middleware::from_fn(super::route_usage::record))
        .layer(cors_layer(&state.config.accounts))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// 单机：任何来源都可以（本机页面、桌面壳、命令行），靠 X-API-Key 认证，不带 Cookie。
/// 多用户：认证靠 Cookie，跨域只放行配置里列出的来源并允许带凭据；没列就只能同域访问。
fn cors_layer(accounts: &AccountsConfig) -> CorsLayer {
    if !accounts.mode.is_multi() {
        return CorsLayer::permissive();
    }
    let origins: Vec<HeaderValue> = accounts
        .allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    if origins.is_empty() {
        return CorsLayer::new();
    }
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods(AllowMethods::mirror_request())
        .allow_headers(AllowHeaders::mirror_request())
}

/// 另一个端口（simple_port）上的应用：只有 `/health`、`/ready`，给健康检查用。
pub fn build_simple_app(state: AppState) -> Router {
    public::routes()
        .method_not_allowed_fallback(method_not_allowed)
        .fallback(unknown_route)
        .layer(cors_layer(&state.config.accounts))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn authenticated_api_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(accounts::routes())
        .merge(credentials::routes())
        .merge(ingestion::routes())
        .merge(glossaries::routes())
        .merge(sync::routes())
        .merge(backups::routes())
        .merge(documents::routes())
        .merge(ai::routes())
        .merge(collections::routes())
        .merge(internal_agent::routes())
        .merge(library::routes())
        .merge(jobs::routes())
        .merge(providers::routes())
        .merge(fonts::routes())
        .merge(model_requests::launcher::routes())
        .method_not_allowed_fallback(method_not_allowed)
        // 先挂的在里层、后跑：先认证（require_api_key），再看能不能访问（enforce_access）。
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::enforce_access,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ))
}
