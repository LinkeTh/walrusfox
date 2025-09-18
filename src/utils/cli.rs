use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "walrusfox",
    about = "Linux-only native host for Pywalfox Extension (Firefox)",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Install the Firefox native messaging manifest (user scope)
    Install,
    /// Uninstall the Firefox native messaging manifest (user scope)
    Uninstall,
    /// Start the native host in the foreground (stdin/stdout)
    Start,
    /// Connect
    Connect,
    /// Trigger an update (refetch colors)
    Update,
    /// Set theme mode to dark
    Dark,
    /// Set theme mode to light
    Light,
    /// Set theme mode to auto
    Auto,
    /// Check connectivity to the local server
    Health,
    /// Print diagnostics about configuration, socket, and logs
    Diagnose,
    /// Print the native messaging manifest JSON to stdout (no file changes)
    PrintManifest,
}
