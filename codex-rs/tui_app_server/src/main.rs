use clap::Parser;
use orbit_code_arg0::Arg0DispatchPaths;
use orbit_code_arg0::arg0_dispatch_or_else;
use orbit_code_tui_app_server::Cli;
use orbit_code_tui_app_server::run_main;
use orbit_code_utils_cli::CliConfigOverrides;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let exit_info = run_main(
            inner,
            arg0_paths,
            orbit_code_core::config_loader::LoaderOverrides::default(),
            /*remote*/ None,
        )
        .await?;
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
