//! 助手终端（WebSocket）：只在单机版。自己做鉴权，挂在认证之外：浏览器的 WebSocket 设不了请求头，
//! 没法走 `require_api_key`，处理函数自己校验，并额外接受查询参数里的 key。
//! 只有真正需要的路由放这里——放宽全局鉴权会把 key 写进每一条访问日志。

use crate::app::AppState;
use crate::routes::ai_terminal;

crate::endpoints! {
    state: AppState;
    /// 助手终端，在服务器上执行命令（多用户下不注册）
    GET "/api/v1/ai/terminal" => ai_terminal::terminal_proxy, closed;
}
