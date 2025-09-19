use anyhow::Result;
use clap::Parser;
use tracing::{error, info};
use walrusfox::client;
use walrusfox::config::Config;
use walrusfox::installer;
use walrusfox::server;
use walrusfox::utils::cli::{Cli, Commands};
use walrusfox::utils::logging::init_logging;

fn main() {
    let config = Config::new();
    let _guard = init_logging(&config);

    let cli = Cli::parse();
    if let Err(e) = real_main(cli, config) {
        error!("error: {e}");
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn real_main(cli: Cli, config: Config) -> Result<()> {
    match cli.command {
        Commands::Install => installer::Installer::new().install()?,
        Commands::Uninstall => installer::Installer::new().uninstall()?,
        Commands::Start => server::Server::new(&config).init()?,
        Commands::Update => client::Client::new(&config).update()?,
        Commands::Dark => client::Client::new(&config).handle_dark()?,
        Commands::Light => client::Client::new(&config).handle_light()?,
        Commands::Auto => client::Client::new(&config).handle_auto()?,
        Commands::Health => client::Client::new(&config).health()?,
        Commands::Diagnose => client::Client::new(&config).diagnose()?,
        Commands::PrintManifest => installer::Installer::new().print_manifest()?,
    }
    Ok(())
}
