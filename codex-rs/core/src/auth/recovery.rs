//! Unauthorized recovery state machine.
//!
//! When a request to the API fails with a 401, the client creates an
//! [`UnauthorizedRecovery`] and drives it through successive recovery steps
//! (reload from disk, OAuth token refresh, external refresh callback) until
//! either the auth state is successfully refreshed or all options are exhausted.

use std::sync::Arc;

use crate::auth::CodexAuth;
use crate::auth::ExternalAuthRefreshReason;
use crate::auth::REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE;
use crate::auth::RefreshTokenError;
use crate::auth::manager::AuthManager;
use crate::auth::manager::ReloadOutcome;
use crate::error::RefreshTokenFailedError;
use crate::error::RefreshTokenFailedReason;

pub(super) enum UnauthorizedRecoveryStep {
    Reload,
    RefreshToken,
    ExternalRefresh,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnauthorizedRecoveryMode {
    Managed,
    External,
    AnthropicOAuth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnauthorizedRecoveryStepResult {
    auth_state_changed: Option<bool>,
}

impl UnauthorizedRecoveryStepResult {
    pub fn auth_state_changed(&self) -> Option<bool> {
        self.auth_state_changed
    }
}

// UnauthorizedRecovery is a state machine that handles an attempt to refresh the authentication when requests
// to API fail with 401 status code.
// The client calls next() every time it encounters a 401 error, one time per retry.
// For API key based authentication, we don't do anything and let the error bubble to the user.
//
// For ChatGPT based authentication, we:
// 1. Attempt to reload the auth data from disk. We only reload if the account id matches the one the current process is running as.
// 2. Attempt to refresh the token using OAuth token refresh flow.
// If after both steps the server still responds with 401 we let the error bubble to the user.
//
// For external ChatGPT auth tokens (chatgptAuthTokens), UnauthorizedRecovery does not touch disk or refresh
// tokens locally. Instead it calls the ExternalAuthRefresher (account/chatgptAuthTokens/refresh) to ask the
// parent app for new tokens, stores them in the ephemeral auth store, and retries once.
pub struct UnauthorizedRecovery {
    pub(super) manager: Arc<AuthManager>,
    pub(super) step: UnauthorizedRecoveryStep,
    pub(super) expected_account_id: Option<String>,
    pub(super) mode: UnauthorizedRecoveryMode,
}

impl UnauthorizedRecovery {
    pub(super) fn new(manager: Arc<AuthManager>) -> Self {
        let cached_auth = manager.auth_cached();
        let expected_account_id = cached_auth.as_ref().and_then(CodexAuth::get_account_id);
        let mode = if cached_auth
            .as_ref()
            .is_some_and(CodexAuth::is_external_chatgpt_tokens)
        {
            UnauthorizedRecoveryMode::External
        } else if matches!(cached_auth.as_ref(), Some(CodexAuth::AnthropicOAuth(_))) {
            UnauthorizedRecoveryMode::AnthropicOAuth
        } else {
            UnauthorizedRecoveryMode::Managed
        };
        let step = match mode {
            UnauthorizedRecoveryMode::Managed => UnauthorizedRecoveryStep::Reload,
            UnauthorizedRecoveryMode::External => UnauthorizedRecoveryStep::ExternalRefresh,
            UnauthorizedRecoveryMode::AnthropicOAuth => UnauthorizedRecoveryStep::Reload,
        };
        Self {
            manager,
            step,
            expected_account_id,
            mode,
        }
    }

    pub fn has_next(&self) -> bool {
        let auth = self.manager.auth_cached();
        let is_recoverable = auth
            .as_ref()
            .is_some_and(|a| a.is_chatgpt_auth() || matches!(a, CodexAuth::AnthropicOAuth(_)));
        if !is_recoverable {
            return false;
        }

        if self.mode == UnauthorizedRecoveryMode::External
            && !self.manager.has_external_auth_refresher()
        {
            return false;
        }

        !matches!(self.step, UnauthorizedRecoveryStep::Done)
    }

    pub fn unavailable_reason(&self) -> &'static str {
        let auth = self.manager.auth_cached();
        let is_recoverable = auth
            .as_ref()
            .is_some_and(|a| a.is_chatgpt_auth() || matches!(a, CodexAuth::AnthropicOAuth(_)));
        if !is_recoverable {
            return "not_recoverable_auth";
        }

        if self.mode == UnauthorizedRecoveryMode::External
            && !self.manager.has_external_auth_refresher()
        {
            return "no_external_refresher";
        }

        if matches!(self.step, UnauthorizedRecoveryStep::Done) {
            return "recovery_exhausted";
        }

        "ready"
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            UnauthorizedRecoveryMode::Managed => "managed",
            UnauthorizedRecoveryMode::External => "external",
            UnauthorizedRecoveryMode::AnthropicOAuth => "anthropic_oauth",
        }
    }

    pub fn step_name(&self) -> &'static str {
        match self.step {
            UnauthorizedRecoveryStep::Reload => "reload",
            UnauthorizedRecoveryStep::RefreshToken => "refresh_token",
            UnauthorizedRecoveryStep::ExternalRefresh => "external_refresh",
            UnauthorizedRecoveryStep::Done => "done",
        }
    }

    pub async fn next(&mut self) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        if !self.has_next() {
            return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                RefreshTokenFailedReason::Other,
                "No more recovery steps available.",
            )));
        }

        // Anthropic OAuth has its own recovery path — skip account-id guard
        if self.mode == UnauthorizedRecoveryMode::AnthropicOAuth {
            return self.next_anthropic_oauth().await;
        }

        match self.step {
            UnauthorizedRecoveryStep::Reload => {
                match self
                    .manager
                    .reload_if_account_id_matches(self.expected_account_id.as_deref())
                {
                    ReloadOutcome::ReloadedChanged => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(true),
                        });
                    }
                    ReloadOutcome::ReloadedNoChange => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(false),
                        });
                    }
                    ReloadOutcome::Skipped => {
                        self.step = UnauthorizedRecoveryStep::Done;
                        return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                            RefreshTokenFailedReason::Other,
                            REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
                        )));
                    }
                }
            }
            UnauthorizedRecoveryStep::RefreshToken => {
                self.manager.refresh_token_from_authority().await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::ExternalRefresh => {
                self.manager
                    .refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
                    .await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::Done => {}
        }
        Ok(UnauthorizedRecoveryStepResult {
            auth_state_changed: None,
        })
    }

    /// Anthropic OAuth recovery: reload from storage, then force-refresh.
    /// No account-id guard — Anthropic doesn't use account IDs.
    async fn next_anthropic_oauth(
        &mut self,
    ) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        match self.step {
            UnauthorizedRecoveryStep::Reload => {
                // Reload from storage — another process may have refreshed the token.
                let old_auth = self.manager.auth_cached();
                self.manager.reload();
                let new_auth = self.manager.auth_cached();
                let changed = !AuthManager::auths_equal(old_auth.as_ref(), new_auth.as_ref());
                self.step = UnauthorizedRecoveryStep::RefreshToken;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(changed),
                })
            }
            UnauthorizedRecoveryStep::RefreshToken => {
                // Force refresh — we got a 401, the token is definitely bad.
                self.manager.force_refresh_anthropic_oauth().await?;
                self.step = UnauthorizedRecoveryStep::Done;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                })
            }
            _ => {
                self.step = UnauthorizedRecoveryStep::Done;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: None,
                })
            }
        }
    }
}
