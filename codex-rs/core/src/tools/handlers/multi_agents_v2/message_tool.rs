//! Shared argument parsing and dispatch for the v2 text-only agent messaging tools.
//!
//! TODO(upstream-0.118.0): preserve the `send_message` queue-only behavior once
//! `InterAgentCommunication` support lands on this branch. For now both tools
//! fall back to `send_input`, which triggers the receiver immediately.

use orbit_code_protocol::models::ResponseInputItem;
use orbit_code_protocol::protocol::CollabAgentInteractionBeginEvent;
use orbit_code_protocol::protocol::CollabAgentInteractionEndEvent;
use orbit_code_protocol::user_input::UserInput;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::agent::agent_resolver::resolve_agent_target;
use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::multi_agents_common::function_arguments;
use crate::tools::handlers::multi_agents_common::tool_output_code_mode_result;
use crate::tools::handlers::multi_agents_common::tool_output_json_text;
use crate::tools::handlers::multi_agents_common::tool_output_response_item;
use crate::tools::handlers::parse_arguments;

#[derive(Clone, Copy)]
pub(crate) enum MessageDeliveryMode {
    QueueOnly,
    TriggerTurn,
}

impl MessageDeliveryMode {
    fn unsupported_items_error(self) -> &'static str {
        match self {
            Self::QueueOnly => "send_message only supports text content in MultiAgentV2 for now",
            Self::TriggerTurn => "assign_task only supports text content in MultiAgentV2 for now",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct MessageToolArgs {
    pub(crate) target: String,
    pub(crate) items: Vec<UserInput>,
    #[serde(default)]
    pub(crate) interrupt: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct MessageToolResult {
    submission_id: String,
}

impl ToolOutput for MessageToolResult {
    fn log_preview(&self) -> String {
        tool_output_json_text(self, "multi_agent_message")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "multi_agent_message")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "multi_agent_message")
    }
}

fn text_content(
    items: &[UserInput],
    mode: MessageDeliveryMode,
) -> Result<String, FunctionCallError> {
    if items.is_empty() {
        return Err(FunctionCallError::RespondToModel(
            "Items can't be empty".to_string(),
        ));
    }
    if items
        .iter()
        .all(|item| matches!(item, UserInput::Text { .. }))
    {
        let text = items
            .iter()
            .filter_map(|item| match item {
                UserInput::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(text);
    }
    Err(FunctionCallError::RespondToModel(
        mode.unsupported_items_error().to_string(),
    ))
}

pub(crate) async fn handle_message_tool(
    invocation: ToolInvocation,
    mode: MessageDeliveryMode,
) -> Result<MessageToolResult, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        payload,
        call_id,
        ..
    } = invocation;
    let arguments = function_arguments(payload)?;
    let args: MessageToolArgs = parse_arguments(&arguments)?;
    let receiver_thread_id = resolve_agent_target(&session, &turn, &args.target).await?;
    let prompt = text_content(&args.items, mode)?;
    let (receiver_agent_nickname, receiver_agent_role) = session
        .services
        .agent_control
        .get_agent_nickname_and_role(receiver_thread_id)
        .await
        .unwrap_or((None, None));
    if args.interrupt {
        session
            .services
            .agent_control
            .interrupt_agent(receiver_thread_id)
            .await
            .map_err(|err| {
                crate::tools::handlers::multi_agents_common::collab_agent_error(
                    receiver_thread_id,
                    err,
                )
            })?;
    }
    session
        .send_event(
            &turn,
            CollabAgentInteractionBeginEvent {
                call_id: call_id.clone(),
                sender_thread_id: session.conversation_id,
                receiver_thread_id,
                prompt: prompt.clone(),
            }
            .into(),
        )
        .await;
    let result = session
        .services
        .agent_control
        .send_input(receiver_thread_id, args.items)
        .await
        .map_err(|err| {
            crate::tools::handlers::multi_agents_common::collab_agent_error(receiver_thread_id, err)
        });
    let status = session
        .services
        .agent_control
        .get_status(receiver_thread_id)
        .await;
    session
        .send_event(
            &turn,
            CollabAgentInteractionEndEvent {
                call_id,
                sender_thread_id: session.conversation_id,
                receiver_thread_id,
                receiver_agent_nickname,
                receiver_agent_role,
                prompt,
                status,
            }
            .into(),
        )
        .await;
    let submission_id = result?;

    Ok(MessageToolResult { submission_id })
}
