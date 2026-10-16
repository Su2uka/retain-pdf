//! AI HTTP entry points and application-owned sidecar gateway.

mod api;
mod agent_calculations;
mod agent_capabilities;
mod agent_capability_api;
mod agent_runtime_session_api;
mod agent_runtime_sessions;
mod document_operation_api;
mod document_operations;
mod public_document_operations;
mod public_document_operations_api;
mod gateway;

// 门面：领域外只能用这里导出的东西，子模块都是私有的（见 BOUNDARIES.md「Domain Facades」）。
pub(crate) use agent_calculations::api::{
    agent_calculation_artifact_download, complete_agent_calculation, create_agent_calculation,
    fail_agent_calculation, get_agent_calculation, list_agent_calculations, AgentCalculationApiDeps,
    AgentCalculationListQuery, AgentCalculationListView, AgentCalculationView,
    CompleteAgentCalculationInput, CreateAgentCalculationInput, FailAgentCalculationInput,
};
pub(crate) use agent_capabilities::AgentCapabilityAuthority;
pub(crate) use agent_capability_api::{
    issue_agent_capability, AgentCapabilityIssueInput, AgentCapabilityIssueView,
};
pub(crate) use agent_runtime_session_api::{
    clear_agent_runtime_session, get_agent_runtime_session, put_agent_runtime_session,
    AgentRuntimeSessionApiDeps, AgentRuntimeSessionView, ClearAgentRuntimeSessionInput,
    PutAgentRuntimeSessionInput,
};
pub(crate) use api::{ask, get_runtime_config, update_runtime_config};
pub(crate) use gateway::AiGateway;
pub(crate) use document_operation_api::{
    authorize_create_scope, authorize_operation_scope, cancel_document_operation,
    commit_document_operation, create_document_operation, get_document_operation_view,
    run_document_operation, AgentCapabilityClaims, CancelDocumentOperationInput,
    CommitDocumentOperationInput, CreateDocumentOperationInput, DocumentOperationView,
    RunDocumentOperationInput,
};
pub(crate) use public_document_operations_api::{
    cancel_public_document_operation, commit_public_document_operation,
    get_public_document_operation, list_document_agent_versions, list_public_document_operations,
    public_document_operation_candidate_download, retry_public_document_operation,
    run_public_document_operation, DocumentAgentVersionListQuery, DocumentAgentVersionListView,
    PublicDocumentOperationActionInput, PublicDocumentOperationListQuery,
    PublicDocumentOperationListView, PublicDocumentOperationView,
};
// 只给测试用的。
#[cfg(test)]
pub(crate) use document_operations::{
    canonical_program_sha256, DocumentOperationControl, RestrictedPageProgramExecutor,
};

