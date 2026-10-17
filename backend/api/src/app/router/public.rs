//! 不需要登录的接口：健康检查，登录注册。挂在认证之外。

use crate::app::AppState;
use crate::routes::{accounts, health};

crate::endpoints! {
    state: AppState;
    /// 进程活着
    GET "/health" => health::health, public;
    /// 能接请求了（AI 服务、jobsd 都就绪）
    GET "/ready" => health::ready, public;
    /// 现在是什么模式、登没登录（前端据此决定显示不显示登录页）
    GET "/api/v1/auth/session" => accounts::session_route, public;
    /// 登录
    POST "/api/v1/auth/login" => accounts::login_route, public;
    /// 注册
    POST "/api/v1/auth/register" => accounts::register_route, public;
}
