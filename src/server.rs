use crate::config::Config;
use std::collections::HashMap;
use std::fs::{remove_file, set_permissions, Permissions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{info, warn};

pub(crate) struct Server<'a> {
    clients: Arc<Mutex<HashMap<u64, Arc<Mutex<UnixStream>>>>>,
    config: &'a Config,
}

impl<'a> Server<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub(crate) fn init(&self) -> anyhow::Result<()> {
        let path = self.config.socket_path.clone();

        let listener = UnixListener::bind(&path)?;
        if let Err(e) = set_permissions(&path, Permissions::from_mode(0o600)) {
            warn!("Failed to set socket permissions to 0600: {}", e);
        }
        info!("Server listening on {}", path.display());

        {
            let cleanup_path = path.clone();
            let _ = ctrlc::set_handler(move || {
                let _ = remove_file(&cleanup_path);
                std::process::exit(0);
            });
        }

        let mut client_id = 0;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client = Arc::new(Mutex::new(stream));
                    let id = client_id;
                    client_id += 1;

                    match self.clients.lock() {
                        Ok(mut map) => {
                            map.insert(id, client.clone());
                        }
                        Err(_) => {
                            warn!("clients lock poisoned; cannot register client {}", id);
                            continue;
                        }
                    }
                    let clients_clone = self.clients.clone();

                    let name = format!("walrusfox-client-{}", id);
                    if let Err(e) = thread::Builder::new().name(name).spawn(move || {
                        Self::handle_client(id, client, clients_clone);
                    }) {
                        warn!("failed to spawn client handler thread for {}: {}", id, e);
                    }
                }
                Err(err) => {
                    warn!("Error accepting connection: {}", err);
                }
            }
        }

        Ok(())
    }

    fn handle_client(
        client_id: u64,
        stream: Arc<Mutex<UnixStream>>,
        clients: Arc<Mutex<HashMap<u64, Arc<Mutex<UnixStream>>>>>,
    ) {
        let reader = {
            // Clone a copy for reading only
            match stream.lock() {
                Ok(s) => match s.try_clone() {
                    Ok(cloned) => cloned,
                    Err(e) => {
                        warn!("client {}: failed to clone stream: {}", client_id, e);
                        return;
                    }
                },
                Err(_) => {
                    warn!("client {}: stream lock poisoned", client_id);
                    return;
                }
            }
        };
        let reader = BufReader::new(reader);

        for line in reader.lines() {
            match line {
                Ok(cmd) => {
                    if cmd.len() > 1024 {
                        warn!(
                            "Ignoring overlong command from client {} ({} bytes)",
                            client_id,
                            cmd.len()
                        );
                        continue;
                    }
                    // Clone target client streams first to avoid holding the mutex during writes
                    let targets: Vec<Arc<Mutex<UnixStream>>> = match clients.lock() {
                        Ok(map) => map
                            .iter()
                            .filter_map(|(id, c)| {
                                if *id != client_id {
                                    Some(c.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                        Err(_) => {
                            warn!("clients lock poisoned; skipping broadcast");
                            Vec::new()
                        }
                    };
                    for client in targets {
                        match client.lock() {
                            Ok(mut c) => {
                                if let Err(e) = c.write_all(cmd.as_bytes()) {
                                    warn!("broadcast write error to client {}: {}", client_id, e);
                                    continue;
                                }
                                if let Err(e) = c.write_all(b"\n") {
                                    warn!(
                                        "broadcast newline write error to client {}: {}",
                                        client_id, e
                                    );
                                    continue;
                                }
                                if let Err(e) = c.flush() {
                                    warn!("broadcast flush error to client {}: {}", client_id, e);
                                }
                            }
                            Err(_) => {
                                warn!("clients lock poisoned during broadcast");
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading from client {}: {}", client_id, e);
                    break;
                }
            }
        }

        // remove client on disconnect
        match clients.lock() {
            Ok(mut map) => {
                map.remove(&client_id);
            }
            Err(_) => {
                warn!("clients lock poisoned while removing client {}", client_id);
            }
        }
        info!("Client {} disconnected", client_id);
    }
}
