use crate::config::Config;
use tracing_subscriber::EnvFilter;

pub fn init_logging(config: &Config) -> Option<tracing_appender::non_blocking::WorkerGuard> {
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

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
    None
}
