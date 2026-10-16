//! AI HTTP entry points and application-owned sidecar gateway.

pub(crate) mod api;
pub mod agent_calculations;
pub mod agent_capabilities;
pub mod agent_capability_api;
pub mod agent_runtime_session_api;
pub(crate) mod agent_runtime_sessions;
pub mod document_operation_api;
pub mod document_operations;
pub mod public_document_operations;
pub mod public_document_operations_api;
mod gateway;

pub(crate) use gateway::AiGateway;
