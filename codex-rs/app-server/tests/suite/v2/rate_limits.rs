use anyhow::Result;
use app_test_support::ChatGptAuthFixture;
use app_test_support::McpProcess;
use app_test_support::to_response;
use app_test_support::write_chatgpt_auth;
use orbit_code_app_server_protocol::GetAccountRateLimitsResponse;
use orbit_code_app_server_protocol::JSONRPCResponse;
use orbit_code_app_server_protocol::LoginAccountResponse;
use orbit_code_app_server_protocol::RateLimitSnapshot;
use orbit_code_app_server_protocol::RequestId;
use orbit_code_core::auth::AuthCredentialsStoreMode;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[tokio::test]
async fn get_account_rate_limits_returns_empty_snapshot_without_auth() -> Result<()> {
    let orbit_code_home = TempDir::new()?;

    let mut mcp =
        McpProcess::new_with_env(orbit_code_home.path(), &[("OPENAI_API_KEY", None)]).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let request_id = mcp.send_get_account_rate_limits_request().await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(request_id)),
    )
    .await??;
    let received: GetAccountRateLimitsResponse = to_response(response)?;
    assert_eq!(received, empty_rate_limits_response());

    Ok(())
}

#[tokio::test]
async fn get_account_rate_limits_returns_empty_snapshot_for_api_key_auth() -> Result<()> {
    let orbit_code_home = TempDir::new()?;

    let mut mcp = McpProcess::new(orbit_code_home.path()).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    login_with_api_key(&mut mcp, "sk-test-key").await?;

    let request_id = mcp.send_get_account_rate_limits_request().await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(request_id)),
    )
    .await??;
    let received: GetAccountRateLimitsResponse = to_response(response)?;
    assert_eq!(received, empty_rate_limits_response());

    Ok(())
}

#[tokio::test]
async fn get_account_rate_limits_returns_empty_snapshot_for_chatgpt_auth() -> Result<()> {
    let orbit_code_home = TempDir::new()?;
    write_chatgpt_auth(
        orbit_code_home.path(),
        ChatGptAuthFixture::new("chatgpt-token")
            .account_id("account-123")
            .plan_type("pro"),
        AuthCredentialsStoreMode::File,
    )?;

    let mut mcp =
        McpProcess::new_with_env(orbit_code_home.path(), &[("OPENAI_API_KEY", None)]).await?;
    timeout(DEFAULT_READ_TIMEOUT, mcp.initialize()).await??;

    let request_id = mcp.send_get_account_rate_limits_request().await?;

    let response: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(request_id)),
    )
    .await??;

    let received: GetAccountRateLimitsResponse = to_response(response)?;
    assert_eq!(received, empty_rate_limits_response());

    Ok(())
}

async fn login_with_api_key(mcp: &mut McpProcess, api_key: &str) -> Result<()> {
    let request_id = mcp.send_login_account_api_key_request(api_key).await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_READ_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(request_id)),
    )
    .await??;
    let login: LoginAccountResponse = to_response(response)?;
    assert_eq!(login, LoginAccountResponse::ApiKey {});

    Ok(())
}

fn empty_rate_limits_response() -> GetAccountRateLimitsResponse {
    GetAccountRateLimitsResponse {
        rate_limits: RateLimitSnapshot {
            limit_id: Some("codex".to_string()),
            limit_name: None,
            primary: None,
            secondary: None,
            credits: None,
            plan_type: None,
        },
        rate_limits_by_limit_id: Some(Default::default()),
    }
}
