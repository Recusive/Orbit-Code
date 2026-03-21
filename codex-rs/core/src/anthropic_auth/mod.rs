//! Consolidated Anthropic authentication module.
//!
//! Gathers all Anthropic-specific auth types, token refresh logic, and
//! request-time credential resolution that were previously scattered across
//! `auth/anthropic.rs`, `auth.rs`, `client.rs`, and `anthropic_bridge.rs`.

mod refresh;
mod request;
mod types;

pub use types::AnthropicApiKeyAuth;
pub use types::AnthropicOAuthAuth;

pub(crate) use refresh::force_refresh;
pub(crate) use refresh::refresh_if_needed;
pub(crate) use request::apply_oauth_modifications;
pub(crate) use request::resolve_auth;
pub(crate) use request::strip_oauth_tool_prefix;
