use crate::config::Config;
use crate::utils::themes;
use anyhow::Result;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;

pub struct Client<'a> {
    config: &'a Config,
}

impl<'a> Client<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn update(&self) -> Result<()> {
        self.send_command("update")
    }

    pub fn handle_dark(&self) -> Result<()> {
        self.send_command("dark")
    }

    pub fn handle_light(&self) -> Result<()> {
        self.send_command("light")
    }

    pub fn handle_auto(&self) -> Result<()> {
        self.send_command("auto")
    }

    pub fn health(&self) -> Result<()> {
        let socket = self.config.socket_path.clone();
        match UnixStream::connect(&socket) {
            Ok(_) => {
                println!("Server is reachable at {}", socket.display());
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Cannot connect to server at {}: {}\nHint: run `walrusfox start` to launch the server.", socket.display(), e)
            }
        }
    }

    pub fn diagnose(&self) -> Result<()> {
        println!("walrusfox diagnostics");
        println!("Version: {}", env!("CARGO_PKG_VERSION"));

        // Socket
        let socket = self.config.socket_path.clone();
        println!("Socket path: {}", socket.display());
        if socket.exists() {
            println!("Socket exists: yes");
            match fs::metadata(&socket) {
                Ok(meta) => {
                    #[cfg(unix)]
                    {
                        let mode = meta.permissions().mode() & 0o7777;
                        println!("Socket permissions: {:o}", mode);
                    }
                }
                Err(e) => println!("Socket metadata error: {}", e),
            }
        } else {
            println!("Socket exists: no");
        }

        // Connectivity
        match UnixStream::connect(&socket) {
            Ok(_) => println!("Connectivity: OK (can connect)"),
            Err(e) => println!("Connectivity: FAIL ({})", e),
        }

        // Log file
        let log_file_path = self.config.log_file.clone();
        println!("Log file: {}", log_file_path.display());
        if log_file_path.exists() {
            match fs::read_to_string(&log_file_path) {
                Ok(contents) => {
                    let mut lines: Vec<&str> = contents.lines().collect();
                    let n = lines.len();
                    let tail = 10usize.min(n);
                    println!("-- Last {} log lines --", tail);
                    for line in lines.drain(n - tail..) {
                        println!("{}", line);
                    }
                }
                Err(e) => println!("Could not read log file: {}", e),
            }
        } else {
            println!("Log file does not exist yet");
        }

        // Colors
        match themes::read_colors() {
            Ok((colors, wall)) => {
                println!("Colors: OK ({} colors)", colors.len());
                if let Some(w) = wall {
                    println!("Wallpaper: {}", w);
                }
            }
            Err(e) => println!("Colors: ERROR ({})", e),
        }

        Ok(())
    }

    fn send_command(&self, cmd: &str) -> Result<()> {
        let socket = self.config.socket_path.clone();
        let mut stream = match UnixStream::connect(&socket) {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!(
                    "Cannot connect to server at {}: {}\nHint: run `walrusfox start` to launch the server.",
                    socket.display(),
                    e
                )
            }
        };
        writeln!(stream, "{}", cmd)?;
        Ok(())
    }
}
