use async_trait::async_trait;
use orbit_code_protocol::models::ResponseInputItem;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::agent::AgentStatus;
use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::multi_agents_common::function_arguments;
use crate::tools::handlers::multi_agents_common::tool_output_code_mode_result;
use crate::tools::handlers::multi_agents_common::tool_output_json_text;
use crate::tools::handlers::multi_agents_common::tool_output_response_item;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::ToolHandler;
use crate::tools::registry::ToolKind;

pub(crate) struct Handler;

#[async_trait]
impl ToolHandler for Handler {
    type Output = ListAgentsResult;

    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<Self::Output, FunctionCallError> {
        let ToolInvocation { payload, .. } = invocation;
        let arguments = function_arguments(payload)?;
        let _args: ListAgentsArgs = parse_arguments(&arguments)?;

        // TODO(upstream-0.118.0): implement this once the branch gains the
        // `AgentControl::list_agents` registry API and path-based agent tracking.
        Err(FunctionCallError::RespondToModel(
            "list_agents is not available until agent registry support lands in orbit-code-core"
                .to_string(),
        ))
    }
}

#[derive(Debug, Deserialize)]
struct ListAgentsArgs {
    path_prefix: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ListedAgent {
    agent_name: String,
    agent_status: AgentStatus,
    last_task_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ListAgentsResult {
    agents: Vec<ListedAgent>,
}

impl ToolOutput for ListAgentsResult {
    fn log_preview(&self) -> String {
        tool_output_json_text(self, "list_agents")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "list_agents")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "list_agents")
    }
}
