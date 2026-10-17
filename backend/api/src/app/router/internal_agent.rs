//! 内部：AI 服务里的 Agent 回调 rust_api 用的接口（凭一次性的能力令牌），浏览器不调。

use crate::app::AppState;
use crate::routes::{
    agent_calculations, agent_capabilities, agent_runtime_sessions, document_operations,
};

crate::endpoints! {
    state: AppState;
    /// 发一个 Agent 能力令牌
    POST "/api/v1/internal/agent/capabilities" => agent_capabilities::issue_agent_capability_route, closed;
    /// Agent 新建一个文档操作
    POST "/api/v1/internal/agent/operations" => document_operations::create_document_operation_route, closed;
    /// 一个文档操作
    GET "/api/v1/internal/agent/operations/:operation_id" => document_operations::get_document_operation_route, closed;
    /// 执行文档操作
    POST "/api/v1/internal/agent/operations/:operation_id/run" => document_operations::run_document_operation_route, closed;
    /// 采用文档操作的结果
    POST "/api/v1/internal/agent/operations/:operation_id/commit" => document_operations::commit_document_operation_route, closed;
    /// 取消文档操作
    POST "/api/v1/internal/agent/operations/:operation_id/cancel" => document_operations::cancel_document_operation_route, closed;
    /// Agent 开始一次计算
    POST "/api/v1/internal/agent/calculations" => agent_calculations::create_agent_calculation_route, closed;
    /// 计算完成
    POST "/api/v1/internal/agent/calculations/:calculation_id/complete" => agent_calculations::complete_agent_calculation_route, closed;
    /// 计算失败
    POST "/api/v1/internal/agent/calculations/:calculation_id/fail" => agent_calculations::fail_agent_calculation_route, closed;
    /// Agent 运行时会话
    GET "/api/v1/internal/agent/runtime-sessions/:conversation_id" => agent_runtime_sessions::get_agent_runtime_session_route, closed;
    /// 保存 Agent 运行时会话
    PUT "/api/v1/internal/agent/runtime-sessions/:conversation_id" => agent_runtime_sessions::put_agent_runtime_session_route, closed;
    /// 清掉 Agent 运行时会话
    DELETE "/api/v1/internal/agent/runtime-sessions/:conversation_id" => agent_runtime_sessions::clear_agent_runtime_session_route, closed;
}
