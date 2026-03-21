pub mod anthropic;
mod device_code_auth;
mod pkce;
mod server;

pub use anthropic::AnthropicAuthMode;
pub use anthropic::AnthropicLoginError;
pub use anthropic::AnthropicTokens;
pub use anthropic::anthropic_authorize_url;
pub use anthropic::anthropic_create_api_key;
pub use anthropic::anthropic_exchange_code;
pub use anthropic::anthropic_refresh_token;
pub use device_code_auth::DeviceCode;
pub use device_code_auth::complete_device_code_login;
pub use device_code_auth::request_device_code;
pub use device_code_auth::run_device_code_login;
pub use orbit_code_client::BuildCustomCaTransportError as BuildLoginHttpClientError;
pub use server::LoginServer;
pub use server::ServerOptions;
pub use server::ShutdownHandle;
pub use server::run_login_server;

// Re-export commonly used auth types and helpers from codex-core for compatibility
pub use orbit_code_app_server_protocol::AuthMode;
pub use orbit_code_core::AuthManager;
pub use orbit_code_core::CodexAuth;
pub use orbit_code_core::auth::AuthDotJson;
pub use orbit_code_core::auth::CLIENT_ID;
pub use orbit_code_core::auth::OPENAI_API_KEY_ENV_VAR;
pub use orbit_code_core::auth::ORBIT_API_KEY_ENV_VAR;
pub use orbit_code_core::auth::login_with_api_key;
pub use orbit_code_core::auth::logout;
pub use orbit_code_core::auth::save_auth;
pub use orbit_code_core::token_data::TokenData;
