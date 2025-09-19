use std::env;
use std::ffi::OsString;
use std::os::unix::net::UnixStream;
use tracing::{error, info, warn};
use walrusfox::bridge::Bridge;
use walrusfox::config::{Config, ALLOWED_EXTENSION};
use walrusfox::server::Server;
use walrusfox::utils::logging::init_logging;

fn main() {
    let config = Config::new();
    let _guard = init_logging(&config);

    validate_args();
    maybe_spawn_server(&config);

    if let Err(e) = Bridge::new(&config).run() {
        error!("Host error: {e}");
        eprintln!("Host error: {e}");
        std::process::exit(1);
    }
}

fn maybe_spawn_server(config: &Config) {
    if UnixStream::connect(&config.socket_file).is_ok() {
        return; // server already up
    }

    let config = config.clone();
    let _ = std::thread::Builder::new()
        .name("walrusfox-embedded-server".to_string())
        .spawn(move || {
            if let Err(e) = Server::new(&config).init() {
                warn!("Embedded server failed to start: {}", e);
            }
        });
}

fn validate_args() {
    let argv: Vec<OsString> = env::args_os().collect();
    info!("Called with : {:?}", argv);
    // Firefox passes [manifest_path, extension_id]
    if argv.len() >= 3 {
        let caller = argv[2].to_string_lossy().to_string();
        if caller != ALLOWED_EXTENSION {
            warn!("Blocked origin: {}", caller);
            std::process::exit(1);
        }
    }
}
