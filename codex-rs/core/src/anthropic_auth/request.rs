//! Request-time Anthropic authentication resolution and OAuth modifications.
//!
//! `resolve_auth` determines which credentials to use for an Anthropic API
//! call, checking the [`AuthManager`], provider config headers, environment
//! variables, and experimental bearer tokens in priority order.
//!
//! `apply_oauth_modifications` applies the OAuth-specific request transforms
//! required by the Anthropic API (tool name prefixing and system prompt
//! prepending).

use crate::AuthManager;
use crate::auth::CodexAuth;
use crate::auth::ProviderName;
use crate::error::CodexErr;
use crate::error::EnvVarError;
use crate::error::Result;
use crate::model_provider_info::ModelProviderInfo;
use http::HeaderMap as ApiHeaderMap;
use orbit_code_anthropic::AnthropicAuth;
use orbit_code_anthropic::Content;
use orbit_code_anthropic::ContentBlock;
use orbit_code_anthropic::MessagesRequest;

/// Tool name prefix required by the Anthropic OAuth endpoint.
pub(crate) const OAUTH_TOOL_PREFIX: &str = "mcp_";

/// Resolve Anthropic authentication from AuthManager, falling back to
/// provider config headers and env vars.
///
/// Priority order:
/// 1. AuthManager stored credentials (OAuth or API key)
/// 2. Provider config headers (`x-api-key`) or env var (`ANTHROPIC_API_KEY`)
/// 3. Experimental bearer token from provider config
pub(crate) fn resolve_auth(
    auth_manager: Option<&AuthManager>,
    provider: &ModelProviderInfo,
    extra_headers: &ApiHeaderMap,
) -> Result<AnthropicAuth> {
    // 1. Check AuthManager for stored Anthropic credentials
    if let Some(manager) = auth_manager {
        let found = manager.auth_cached_for_provider(ProviderName::Anthropic);
        tracing::info!(
            found = found.is_some(),
            auth_mode = ?found.as_ref().map(CodexAuth::auth_mode),
            "resolve_anthropic_auth: checking AuthManager"
        );
        if let Some(auth) = found {
            match auth {
                CodexAuth::AnthropicOAuth(ref oauth) => {
                    return Ok(AnthropicAuth::BearerToken(oauth.access_token().to_string()));
                }
                CodexAuth::AnthropicApiKey(ref api_key_auth) => {
                    return Ok(AnthropicAuth::ApiKey(api_key_auth.api_key().to_string()));
                }
                _ => {
                    tracing::warn!(auth_mode = ?auth.auth_mode(), "resolve_anthropic_auth: unexpected variant");
                }
            }
        }
    } else {
        tracing::warn!("resolve_anthropic_auth: no AuthManager available");
    }

    // 2. Fall back to provider config headers or env var (existing 3a behavior)
    if let Some(api_key) = extra_headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or(provider.api_key()?)
    {
        return Ok(AnthropicAuth::ApiKey(api_key));
    }

    // 3. Check experimental bearer token
    if let Some(bearer) = &provider.experimental_bearer_token {
        return Ok(AnthropicAuth::BearerToken(bearer.clone()));
    }

    Err(CodexErr::EnvVar(EnvVarError {
        var: "ANTHROPIC_API_KEY".to_string(),
        instructions: provider.env_key_instructions.clone(),
    }))
}

/// Apply OAuth-specific modifications to an Anthropic request:
/// - Prefix tool names with `mcp_` (Anthropic requirement for OAuth)
/// - Prepend the required "You are Claude Code" system block
pub(crate) fn apply_oauth_modifications(request: &mut MessagesRequest) {
    prefix_tool_names_for_oauth(request);
    // The OAuth endpoint requires "You are Claude Code..." as a separate
    // first system block for access to premium models (opus-4-6, sonnet-4-6).
    let prefix_block = orbit_code_anthropic::SystemBlock {
        r#type: "text".to_string(),
        text: "You are Claude Code, Anthropic's official CLI for Claude.".to_string(),
        cache_control: None,
    };
    if let Some(system) = &mut request.system {
        // Prepend as separate first block (must not be concatenated)
        system.insert(0, prefix_block);
    } else {
        request.system = Some(vec![prefix_block]);
    }
}

/// Prefix tool names with `mcp_` for OAuth mode (Anthropic requirement).
/// Must prefix BOTH tool definitions AND tool_use blocks in message history.
pub(crate) fn prefix_tool_names_for_oauth(request: &mut MessagesRequest) {
    // 1. Prefix tool definitions
    if let Some(tools) = &mut request.tools {
        for tool in tools {
            if !tool.name.starts_with(OAUTH_TOOL_PREFIX) {
                tool.name = format!("{OAUTH_TOOL_PREFIX}{}", tool.name);
            }
        }
    }

    // 2. Prefix tool_use blocks in message history
    for message in &mut request.messages {
        if let Content::Blocks(blocks) = &mut message.content {
            for block in blocks {
                if let ContentBlock::ToolUse { name, .. } = block
                    && !name.starts_with(OAUTH_TOOL_PREFIX)
                {
                    *name = format!("{OAUTH_TOOL_PREFIX}{name}");
                }
            }
        }
    }
}

/// Strip `mcp_` prefix from tool names in responses (Anthropic requirement).
pub(crate) fn strip_oauth_tool_prefix(name: &str) -> String {
    name.strip_prefix(OAUTH_TOOL_PREFIX)
        .unwrap_or(name)
        .to_string()
}
