use crate::config::{Config, MAX_MSG_LEN};
use anyhow::Context;
use anyhow::Result;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::{remove_file, set_permissions, Permissions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tracing::{debug, info, warn};

struct SocketGuard(PathBuf);
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = remove_file(&self.0);
    }
}

struct Client {
    writer: Mutex<UnixStream>,
}

type ClientMap = Arc<Mutex<HashMap<u64, Arc<Client>>>>;

pub struct Server<'a> {
    clients: ClientMap,
    config: &'a Config,
}

impl<'a> Server<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn init(&self) -> Result<()> {
        let path = self.config.socket_file.clone();
        let _guard = SocketGuard(path.clone());
        let listener = Self::bind_socket(&path)?;
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
                    let client: Arc<Client> = Arc::new(Client {
                        writer: Mutex::new(stream),
                    });

                    let id = client_id;
                    client_id += 1;

                    {
                        let mut map = self.clients.lock();
                        info!("Client {} connected", id);
                        map.insert(id, client.clone());
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
                    debug!("Error accepting connection: {}", err);
                }
            }
        }

        Ok(())
    }

    fn handle_client(client_id: u64, stream: Arc<Client>, clients: ClientMap) {
        let reader = match Self::init_client_reader(client_id, stream) {
            Some(value) => value,
            None => return,
        };

        for line in reader.lines() {
            match line {
                Ok(cmd) => {
                    if cmd.len() > MAX_MSG_LEN {
                        warn!(
                            "Ignoring overlong command from client {} ({} bytes)",
                            client_id,
                            cmd.len()
                        );
                        continue;
                    }

                    let targets = Self::filter_target_clients(client_id, &clients);

                    for (cid, client) in targets {
                        if Self::write_to_client(&cmd, cid, client) {
                            continue;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading from client {}: {}", client_id, e);
                    break;
                }
            }
        }
        Self::disconnect_client(&client_id, clients);
    }

    fn write_to_client(cmd: &str, client_id: u64, client: Arc<Client>) -> bool {
        let mut writer = client.writer.lock();
        let mut msg = Vec::with_capacity(cmd.len() + 1);
        msg.extend_from_slice(cmd.as_bytes());
        msg.push(b'\n');
        if let Err(e) = writer.write_all(&msg) {
            warn!("broadcast write error to recipient {}: {}", client_id, e);
            return true;
        }
        if let Err(e) = writer.flush() {
            warn!("broadcast flush error to recipient {}: {}", client_id, e);
        }
        false
    }

    fn disconnect_client(client_id: &u64, clients: ClientMap) {
        // remove client on disconnect
        let mut map = clients.lock();
        map.remove(client_id);
        info!("Client {} disconnected", client_id);
    }

    fn init_client_reader(client_id: u64, stream: Arc<Client>) -> Option<BufReader<UnixStream>> {
        let reader = {
            // Clone a copy for reading only
            let s = stream.writer.lock();
            match s.try_clone() {
                Ok(cloned) => cloned,
                Err(e) => {
                    warn!("client {}: failed to clone stream: {}", client_id, e);
                    return None;
                }
            }
        };
        let reader = BufReader::new(reader);
        Some(reader)
    }

    fn filter_target_clients(client_id: u64, clients: &ClientMap) -> Vec<(u64, Arc<Client>)> {
        let targets: Vec<(u64, Arc<Client>)> = {
            let map = clients.lock();
            map.iter()
                .filter(|(rid, _c)| (**rid != client_id))
                .map(|(rid, c)| (*rid, c.clone()))
                .collect()
        };
        targets
    }

    fn bind_socket(path: &Path) -> Result<UnixListener> {
        if path.exists() && UnixStream::connect(path).is_err() {
            let _ = remove_file(path);
        }
        let listener =
            UnixListener::bind(path).with_context(|| format!("bind {}", path.display()))?;
        if let Err(e) = set_permissions(path, Permissions::from_mode(0o600)) {
            warn!("Failed to set socket permissions to 0600: {}", e);
        }
        Ok(listener)
    }
}
