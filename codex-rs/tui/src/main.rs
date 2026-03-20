use clap::Parser;
use orbit_code_arg0::Arg0DispatchPaths;
use orbit_code_arg0::arg0_dispatch_or_else;
use orbit_code_tui::AppExitInfo;
use orbit_code_tui::Cli;
use orbit_code_tui::ExitReason;
use orbit_code_tui::run_main;
use orbit_code_tui::update_action::UpdateAction;
use orbit_code_utils_cli::CliConfigOverrides;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn into_app_server_cli(cli: Cli) -> orbit_code_tui_app_server::Cli {
    orbit_code_tui_app_server::Cli {
        prompt: cli.prompt,
        images: cli.images,
        resume_picker: cli.resume_picker,
        resume_last: cli.resume_last,
        resume_session_id: cli.resume_session_id,
        resume_show_all: cli.resume_show_all,
        fork_picker: cli.fork_picker,
        fork_last: cli.fork_last,
        fork_session_id: cli.fork_session_id,
        fork_show_all: cli.fork_show_all,
        model: cli.model,
        model_provider: cli.model_provider,
        oss: cli.oss,
        oss_provider: cli.oss_provider,
        config_profile: cli.config_profile,
        sandbox_mode: cli.sandbox_mode,
        approval_policy: cli.approval_policy,
        full_auto: cli.full_auto,
        dangerously_bypass_approvals_and_sandbox: cli.dangerously_bypass_approvals_and_sandbox,
        cwd: cli.cwd,
        web_search: cli.web_search,
        add_dir: cli.add_dir,
        no_alt_screen: cli.no_alt_screen,
        config_overrides: cli.config_overrides,
    }
}

fn into_legacy_update_action(
    action: orbit_code_tui_app_server::update_action::UpdateAction,
) -> UpdateAction {
    match action {
        orbit_code_tui_app_server::update_action::UpdateAction::NpmGlobalLatest => {
            UpdateAction::NpmGlobalLatest
        }
        orbit_code_tui_app_server::update_action::UpdateAction::BunGlobalLatest => {
            UpdateAction::BunGlobalLatest
        }
        orbit_code_tui_app_server::update_action::UpdateAction::BrewUpgrade => {
            UpdateAction::BrewUpgrade
        }
    }
}

fn into_legacy_exit_reason(reason: orbit_code_tui_app_server::ExitReason) -> ExitReason {
    match reason {
        orbit_code_tui_app_server::ExitReason::UserRequested => ExitReason::UserRequested,
        orbit_code_tui_app_server::ExitReason::Fatal(message) => ExitReason::Fatal(message),
    }
}

fn into_legacy_exit_info(exit_info: orbit_code_tui_app_server::AppExitInfo) -> AppExitInfo {
    AppExitInfo {
        token_usage: exit_info.token_usage,
        thread_id: exit_info.thread_id,
        thread_name: exit_info.thread_name,
        update_action: exit_info.update_action.map(into_legacy_update_action),
        exit_reason: into_legacy_exit_reason(exit_info.exit_reason),
    }
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let use_app_server_tui = orbit_code_tui::should_use_app_server_tui(&inner).await?;
        let exit_info = if use_app_server_tui {
            into_legacy_exit_info(
                orbit_code_tui_app_server::run_main(
                    into_app_server_cli(inner),
                    arg0_paths,
                    orbit_code_core::config_loader::LoaderOverrides::default(),
                    /*remote*/ None,
                )
                .await?,
            )
        } else {
            run_main(
                inner,
                arg0_paths,
                orbit_code_core::config_loader::LoaderOverrides::default(),
            )
            .await?
        };
        let token_usage = exit_info.token_usage;
        if !token_usage.is_zero() {
            println!(
                "{}",
                orbit_code_protocol::protocol::FinalOutput::from(token_usage),
            );
        }
        Ok(())
    })
}
