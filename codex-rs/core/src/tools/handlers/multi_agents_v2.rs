//! Implements the MultiAgentV2 collaboration tool surface.
//!
//! TODO(upstream-0.118.0): wire these handlers into the tool registry once the
//! path-based agent protocol and `AgentControl` APIs land on this branch.

pub(crate) use assign_task::Handler as AssignTaskHandler;
pub(crate) use close_agent::Handler as CloseAgentHandler;
pub(crate) use list_agents::Handler as ListAgentsHandler;
pub(crate) use send_message::Handler as SendMessageHandler;
pub(crate) use spawn::Handler as SpawnAgentHandler;
pub(crate) use wait::Handler as WaitAgentHandler;

mod assign_task;
mod close_agent;
mod list_agents;
mod message_tool;
mod send_message;
mod spawn;
pub(crate) mod wait;
