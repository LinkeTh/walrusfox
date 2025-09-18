mod bridge;
mod client;
mod config;
mod installer;
mod protocol;
mod server;
mod utils;
use anyhow::Result;
use clap::Parser;
use config::Config;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;
use utils::cli::{Cli, Commands};

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

fn init_logging(config: &Config) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if let Ok(file_appender) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.log_file)
    {
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(non_blocking)
            .init();
        return Some(guard);
    }

    // Fallback to stderr
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
    None
}

fn real_main(cli: Cli, config: Config) -> Result<()> {
    debug!("Enter walrusfox");

    match cli.command {
        Commands::Install => installer::Installer::new().install()?,
        Commands::Uninstall => installer::Installer::new().uninstall()?,
        Commands::Connect => bridge::Bridge::new(&config).run()?,
        Commands::Start => server::Server::new(&config).init()?,
        Commands::Update => client::Client::new(&config).update()?,
        Commands::Dark => client::Client::new(&config).handle_dark()?,
        Commands::Light => client::Client::new(&config).handle_light()?,
        Commands::Auto => client::Client::new(&config).handle_auto()?,
        Commands::Health => client::Client::new(&config).health()?,
        Commands::Diagnose => client::Client::new(&config).diagnose()?,
        Commands::PrintManifest => installer::Installer::new().print_manifest()?,
    }

    debug!("Exit walrusfox");

    Ok(())
}
