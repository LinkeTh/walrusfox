use directories::ProjectDirs;
use serde::Deserialize;
use std::env;
use std::fs;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use tracing::warn;

pub const HOST_NAME: &str = "pywalfox"; // keep the same host name used by the Python implementation
pub const ALLOWED_EXTENSION: &str = "pywalfox@frewacom.org"; // Firefox add-on id

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub socket_path: PathBuf,
    pub log_file: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let socket_path = Self::socket_path();
        let log_file = Self::log_file_path();
        Self {
            socket_path,
            log_file,
        }
    }

    fn socket_path() -> PathBuf {
        if let Ok(p) = env::var("WALRUSFOX_SOCKET") {
            return PathBuf::from(p);
        }
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            let dir = Path::new(&runtime_dir).join("walrusfox");
            Self::ensure_dir_mode_0700(&dir);
            return dir.join("walrusfox.sock");
        }
        PathBuf::from("/tmp/walrusfox.sock")
    }

    fn log_file_path() -> PathBuf {
        if let Ok(p) = env::var("WALRUSFOX_LOG") {
            return PathBuf::from(p);
        }
        if let Some(proj) = ProjectDirs::from("de", "linket", "walrusfox") {
            if let Some(state_dir) = proj.state_dir() {
                let path = state_dir.to_path_buf();
                if fs::create_dir_all(&path).is_ok() {
                    return path.join("walrusfox.log");
                }
            }
        }
        PathBuf::from("/tmp/walrusfox.log")
    }

    fn ensure_dir_mode_0700(dir: &Path) {
        if dir.exists() {
            return;
        }
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        builder.mode(0o700);
        if let Err(e) = builder.create(dir) {
            warn!("failed to create directory {}: {}", dir.display(), e);
        }
    }
}
