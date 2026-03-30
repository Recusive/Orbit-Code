//! Resolves tool-facing agent targets to concrete thread IDs.

use std::sync::Arc;

use orbit_code_protocol::ThreadId;

use crate::codex::Session;
use crate::codex::TurnContext;
use crate::function_tool::FunctionCallError;

/// Resolves a single tool-facing agent target to a thread id.
pub(crate) async fn resolve_agent_target(
    session: &Arc<Session>,
    turn: &Arc<TurnContext>,
    target: &str,
) -> Result<ThreadId, FunctionCallError> {
    if let Ok(thread_id) = ThreadId::from_string(target) {
        return Ok(thread_id);
    }

    let _ = (session, turn);

    // TODO(upstream-0.118.0): restore the upstream path-based flow once AgentControl
    // grows `register_session_root` + `resolve_agent_reference` support on this branch.
    Err(FunctionCallError::RespondToModel(format!(
        "agent target `{target}` is not a valid thread id, and path-based agent resolution is not available in orbit-code-core yet"
    )))
}

/// Resolves multiple tool-facing agent targets to thread ids.
pub(crate) async fn resolve_agent_targets(
    session: &Arc<Session>,
    turn: &Arc<TurnContext>,
    targets: Vec<String>,
) -> Result<Vec<ThreadId>, FunctionCallError> {
    if targets.is_empty() {
        return Err(FunctionCallError::RespondToModel(
            "agent targets must be non-empty".to_string(),
        ));
    }

    let mut resolved = Vec::with_capacity(targets.len());
    for target in &targets {
        resolved.push(resolve_agent_target(session, turn, target).await?);
    }
    Ok(resolved)
}
