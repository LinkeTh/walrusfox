use anyhow::Result;
use clap::Parser;
use tracing::error;
use walrusfox::client;
use walrusfox::config::Config;
use walrusfox::installer;
use walrusfox::server;
use walrusfox::utils::cli::{Cli, Commands};
use walrusfox::utils::logging::init_logging;

fn main() {
    let config = Config::new();
    let _guard = init_logging(&config);

    match Cli::try_parse() {
        Ok(cli) => {
            if let Err(e) = run(cli, config) {
                error!("Error: {e}");
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli, config: Config) -> Result<()> {
    match cli.command {
        Commands::Install => installer::Installer::new().install()?,
        Commands::Uninstall => installer::Installer::new().uninstall()?,
        Commands::PrintManifest => installer::Installer::new().print_manifest()?,
        Commands::Start => server::Server::new(&config).init()?,
        Commands::Update => client::Client::new(&config).update()?,
        Commands::Dark => client::Client::new(&config).handle_dark()?,
        Commands::Light => client::Client::new(&config).handle_light()?,
        Commands::Auto => client::Client::new(&config).handle_auto()?,
        Commands::Health => client::Client::new(&config).health()?,
        Commands::Diagnose => client::Client::new(&config).diagnose()?,
    }
    Ok(())
}
