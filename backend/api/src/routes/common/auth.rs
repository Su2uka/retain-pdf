use std::collections::HashSet;

use axum::http::{header, HeaderMap, HeaderValue, Uri};

use crate::app::AppState;
use crate::config::OriginPolicyConfig;

pub struct AuthRouteDeps<'a> {
    pub api_keys: &'a HashSet<String>,
}

pub fn build_auth_route_deps(state: &AppState) -> AuthRouteDeps<'_> {
    AuthRouteDeps {
        api_keys: &state.config.api_keys,
    }
}

/// 终端 WebSocket 代理要的东西：一把校验用的 key 集合 + ai_service 的基址。
///
/// 和 AuthRouteDeps 分开，是因为它多要一个上游地址，而那个地址与鉴权无关 ——
/// 合在一起会让每个鉴权点都拖着一个它用不到的字段。
pub struct AiTerminalRouteDeps<'a> {
    pub api_keys: &'a HashSet<String>,
    pub ai_service_base_url: &'a str,
    /// 握手时查 `Origin`：浏览器不对 WebSocket 做 CORS，只能这里自己查。
    pub origin_policy: &'a OriginPolicyConfig,
}

pub fn build_ai_terminal_route_deps(state: &AppState) -> AiTerminalRouteDeps<'_> {
    AiTerminalRouteDeps {
        api_keys: &state.config.api_keys,
        ai_service_base_url: state.ai_gateway.base_url(),
        origin_policy: &state.config.origin_policy,
    }
}

/// 这次请求的主机：`Host`，以及反向代理写的 `X-Forwarded-Host`（取第一个）。
fn request_hosts(headers: &HeaderMap) -> impl Iterator<Item = &str> {
    [header::HOST.as_str(), "x-forwarded-host"]
        .into_iter()
        .filter_map(|name| headers.get(name))
        .filter_map(|value| value.to_str().ok())
}

/// 单机模式下，浏览器带来的 `Origin` 能不能用本机 API。CORS 和助手终端握手共用这一个判断。
pub fn request_origin_is_allowed(
    policy: &OriginPolicyConfig,
    headers: &HeaderMap,
    origin: &HeaderValue,
) -> bool {
    origin
        .to_str()
        .is_ok_and(|origin| policy.allows_origin(origin, request_hosts(headers)))
}

/// 单机模式防 DNS rebinding：只看 `Host`（HTTP/2 没有它时看 `:authority`），不看
/// `X-Forwarded-Host`——rebinding 的页面和 API 同源，能随便设它。
pub fn request_host_is_allowed(
    policy: &OriginPolicyConfig,
    headers: &HeaderMap,
    uri: &Uri,
    listening_on_loopback: bool,
) -> bool {
    let host = match headers.get(header::HOST) {
        // 写不成字符串的 Host 当作一个谁都不认的主机。
        Some(value) => Some(value.to_str().unwrap_or("\u{0}")),
        None => uri.authority().map(|authority| authority.as_str()),
    };
    policy.allows_host(host, listening_on_loopback)
}

