//! AI 助手：问答、对话、Agent 的计算和文档操作。多用户下整组关闭：助手能读全部数据，
//! 等按用户隔离做完再开。

use crate::app::AppState;
use crate::routes::{agent_calculations, ai_proxy, library_extras, public_document_operations};

crate::endpoints! {
    state: AppState;
    /// 问一个问题（转给 AI 服务，流式返回）
    POST "/api/v1/ai/ask" => ai_proxy::ask_proxy, closed;
    /// AI 服务的运行配置
    GET "/api/v1/ai/runtime-config" => ai_proxy::get_runtime_config_proxy, closed;
    /// 保存 AI 服务的运行配置
    PUT "/api/v1/ai/runtime-config" => ai_proxy::update_runtime_config_proxy, closed;
    /// 新建对话
    POST "/api/v1/ai/conversations" => library_extras::create_conversation_route, closed;
    /// 对话列表
    GET "/api/v1/ai/conversations" => library_extras::list_conversations_route, closed;
    /// 从某条消息分叉出新对话
    POST "/api/v1/ai/conversations/fork" => library_extras::fork_conversation_route, closed;
    /// 一个对话
    GET "/api/v1/ai/conversations/:conversation_id" => library_extras::get_conversation_route, closed;
    /// 改对话（标题、置顶……）
    PATCH "/api/v1/ai/conversations/:conversation_id" => library_extras::patch_conversation_route, closed;
    /// 删除对话
    DELETE "/api/v1/ai/conversations/:conversation_id" => library_extras::delete_conversation_route, closed;
    /// 往对话里加一条消息
    POST "/api/v1/ai/conversations/:conversation_id/messages" => library_extras::append_message_route, closed;
    /// 对话里 Agent 发起的文档操作
    GET "/api/v1/ai/conversations/:conversation_id/operations" => public_document_operations::list_public_document_operations_route, closed;
    /// 对话里 Agent 做的计算
    GET "/api/v1/ai/conversations/:conversation_id/calculations" => agent_calculations::list_agent_calculations_route, closed;
    /// 一次计算
    GET "/api/v1/ai/calculations/:calculation_id" => agent_calculations::get_agent_calculation_route, closed;
    /// 计算产出的一个文件
    GET "/api/v1/ai/calculations/:calculation_id/artifacts/:artifact_id" => agent_calculations::download_agent_calculation_artifact_route, closed;
    /// 一次文档操作
    GET "/api/v1/ai/operations/:operation_id" => public_document_operations::get_public_document_operation_route, closed;
    /// 执行文档操作
    POST "/api/v1/ai/operations/:operation_id/run" => public_document_operations::run_public_document_operation_route, closed;
    /// 重试文档操作
    POST "/api/v1/ai/operations/:operation_id/retry" => public_document_operations::retry_public_document_operation_route, closed;
    /// 取消文档操作
    POST "/api/v1/ai/operations/:operation_id/cancel" => public_document_operations::cancel_public_document_operation_route, closed;
    /// 采用文档操作的结果
    POST "/api/v1/ai/operations/:operation_id/commit" => public_document_operations::commit_public_document_operation_route, closed;
    /// 文档操作改出来的候选 PDF
    GET "/api/v1/ai/operations/:operation_id/candidate.pdf" => public_document_operations::download_public_document_operation_candidate_route, closed;
}
