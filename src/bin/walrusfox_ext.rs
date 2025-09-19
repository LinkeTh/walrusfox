use std::env;
use std::ffi::OsString;
use tracing::{error, info, warn};
use walrusfox::bridge::Bridge;
use walrusfox::config::{Config, ALLOWED_EXTENSION};
use walrusfox::utils::logging::init_logging;

fn main() {
    let config = Config::new();
    let _guard = init_logging(&config);

    // Firefox passes [manifest_path, extension_id]
    let argv: Vec<OsString> = env::args_os().collect();
    info!("native host binary called with : {:?}", argv);
    if argv.len() >= 3 {
        let caller = argv[2].to_string_lossy().to_string();
        if caller != ALLOWED_EXTENSION {
            // Log and exit quietly; stdout must stay clean
            warn!("blocked origin: {}", caller);
            std::process::exit(1);
        }
    }

    if let Err(e) = Bridge::new(&config).run() {
        error!("host error: {e}");
        eprintln!("host error: {e}");
        std::process::exit(1);
    }
}
