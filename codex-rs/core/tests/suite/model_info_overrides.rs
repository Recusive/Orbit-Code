use core_test_support::load_default_config_for_test;
use orbit_code_core::CodexAuth;
use orbit_code_core::models_manager::collaboration_mode_presets::CollaborationModesConfig;
use orbit_code_core::models_manager::manager::ModelsManager;
use orbit_code_protocol::openai_models::TruncationPolicyConfig;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn offline_model_info_without_tool_output_override() {
    let orbit_code_home = TempDir::new().expect("create temp dir");
    let config = load_default_config_for_test(&orbit_code_home).await;
    let auth_manager = orbit_code_core::test_support::auth_manager_from_auth(
        CodexAuth::create_dummy_chatgpt_auth_for_testing(),
    );
    let manager = ModelsManager::new(
        config.orbit_code_home.clone(),
        auth_manager,
        None,
        CollaborationModesConfig::default(),
    );

    let model_info = manager.get_model_info("gpt-5.1", &config).await;

    assert_eq!(
        model_info.truncation_policy,
        TruncationPolicyConfig::bytes(10_000)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn offline_model_info_with_tool_output_override() {
    let orbit_code_home = TempDir::new().expect("create temp dir");
    let mut config = load_default_config_for_test(&orbit_code_home).await;
    config.tool_output_token_limit = Some(123);
    let auth_manager = orbit_code_core::test_support::auth_manager_from_auth(
        CodexAuth::create_dummy_chatgpt_auth_for_testing(),
    );
    let manager = ModelsManager::new(
        config.orbit_code_home.clone(),
        auth_manager,
        None,
        CollaborationModesConfig::default(),
    );

    let model_info = manager.get_model_info("gpt-5.1-codex", &config).await;

    assert_eq!(
        model_info.truncation_policy,
        TruncationPolicyConfig::tokens(123)
    );
}
