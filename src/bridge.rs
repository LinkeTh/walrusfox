//! Bridge between the Unix socket server and the browser native-messaging.
//! - Listens for commands from the local Unix socket and forwards responses to the browser.
//! - Answers browser-originated requests on stdin/stdout (native messaging protocol).
//! - Gracefully shuts down when stdin closes or on fatal socket errors.

use crate::config::Config;
use crate::protocol::events::{BrowserAction, SocketCommand};
use crate::protocol::native_messaging::{
    read_message, send_colors, send_invalid_response, send_theme_mode, send_version, Request,
};
use anyhow::{Context, Result};
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

pub(crate) struct Bridge<'a> {
    config: &'a Config,
}

impl<'a> Bridge<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self) -> Result<()> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_socket = shutdown.clone();
        let socket = self.config.socket_path.clone();

        let _ = thread::Builder::new()
            .name("walrusfox-bridge-socket".to_string())
            .spawn(move || {
                let ss = shutdown_socket;
                if let Err(e) = Self::socket_loop(ss.clone(), &socket) {
                    if !ss.load(Ordering::SeqCst) {
                        error!("Socket loop failed: {e}");
                    }
                }
            });

        while let Some(msg) = read_message::<Request>()? {
            match msg.action.parse::<BrowserAction>() {
                Ok(action) => {
                    if let BrowserAction::Invalid = action {
                        // Unknown action from browser
                        warn!("browser sent invalid action: {}", msg.action);
                        send_invalid_response()?;
                        continue;
                    }
                    Self::handle_browser_request(action)?;
                }
                Err(_) => {
                    warn!("failed to parse browser action: {}", msg.action);
                    send_invalid_response()?;
                    continue;
                }
            }
        }

        warn!("stdin closed; initiating graceful shutdown");
        shutdown.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn socket_loop(shutdown: Arc<AtomicBool>, path: &PathBuf) -> Result<()> {
        loop {
            if shutdown.load(Ordering::SeqCst) {
                break;
            }

            match UnixStream::connect(path) {
                Ok(stream) => {
                    info!("Connected to server at {}", path.display());
                    if let Err(e) = Self::handle_command(stream) {
                        if shutdown.load(Ordering::SeqCst) {
                            break;
                        }
                        warn!("Socket handler ended: {e}");
                    }
                }
                Err(e) => {
                    if shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                    warn!("Cannot connect to {}: {e} (will retry)", path.display());
                }
            }

            for _ in 0..10 {
                if shutdown.load(Ordering::SeqCst) {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
        Ok(())
    }

    fn handle_command(stream: UnixStream) -> Result<()> {
        let reader = BufReader::new(&stream);
        for line in reader.lines() {
            debug!("Received line: {:?}", line);
            match line {
                Ok(cmd) => {
                    info!("Received command: {}", cmd);
                    match cmd.parse::<SocketCommand>() {
                        Ok(SocketCommand::Update) => send_colors()?,
                        Ok(SocketCommand::Auto) => send_theme_mode(SocketCommand::Auto.value())?,
                        Ok(SocketCommand::Dark) => send_theme_mode(SocketCommand::Dark.value())?,
                        Ok(SocketCommand::Light) => send_theme_mode(SocketCommand::Light.value())?,
                        Ok(SocketCommand::Unknown(_)) | Err(_) => send_invalid_response()?,
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(e))
                        .with_context(|| "reading from Unix socket".to_string());
                }
            }
        }
        Ok(())
    }

    fn handle_browser_request(action: BrowserAction) -> Result<()> {
        info!("Action received {:?}", action);
        match action {
            BrowserAction::Version => send_version()?,
            BrowserAction::Colors => send_colors()?,
            BrowserAction::Invalid => send_invalid_response()?,
            BrowserAction::ThemeMode => send_theme_mode("auto")?,
        }
        Ok(())
    }
}
