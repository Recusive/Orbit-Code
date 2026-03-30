use async_trait::async_trait;

use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::handlers::multi_agents_v2::message_tool::MessageDeliveryMode;
use crate::tools::handlers::multi_agents_v2::message_tool::MessageToolResult;
use crate::tools::handlers::multi_agents_v2::message_tool::handle_message_tool;
use crate::tools::registry::ToolHandler;
use crate::tools::registry::ToolKind;

pub(crate) struct Handler;

#[async_trait]
impl ToolHandler for Handler {
    type Output = MessageToolResult;

    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<Self::Output, FunctionCallError> {
        handle_message_tool(invocation, MessageDeliveryMode::QueueOnly).await
    }
}
