use crate::config::{ALLOWED_EXTENSION, HOST_NAME};
use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize)]
struct Manifest<'a> {
    name: &'a str,
    description: &'a str,
    path: String,
    r#type: &'a str,
    allowed_extensions: [&'a str; 1],
}

pub struct Installer {}

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}

impl Installer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn install(&self) -> Result<()> {
        Self::install_manifest()
    }

    pub fn uninstall(&self) -> Result<()> {
        Self::uninstall_manifest()
    }

    pub fn print_manifest(&self) -> Result<()> {
        let manifest = Self::build_manifest();
        let data = serde_json::to_string_pretty(&manifest)?;
        println!("{}", data);
        Ok(())
    }

    fn install_manifest() -> Result<()> {
        let manifest_dir = Self::mozilla_native_hosts_dir_user();
        fs::create_dir_all(&manifest_dir)
            .with_context(|| format!("creating {}", manifest_dir.display()))?;

        let manifest = Self::build_manifest();
        let manifest_path = Self::manifest_path_user();
        let data = serde_json::to_vec_pretty(&manifest)?;
        fs::write(&manifest_path, data)
            .with_context(|| format!("writing manifest {}", manifest_path.display()))?;

        println!("Installed manifest at {}", manifest_path.display());

        Ok(())
    }

    fn build_manifest() -> Manifest<'static> {
        let path = std::env::current_exe().expect("resolve current exe path");
        let bin = format!("{}-ext", path.display());
        Manifest {
            name: HOST_NAME,
            description: "Automatically theme your browser using external colors",
            path: bin,
            r#type: "stdio",
            allowed_extensions: [ALLOWED_EXTENSION],
        }
    }

    fn uninstall_manifest() -> Result<()> {
        let path = Self::manifest_path_user();
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
            println!("Removed manifest {}", path.display());
        } else {
            println!("Manifest not found at {}", path.display());
        }

        Ok(())
    }

    fn mozilla_native_hosts_dir_user() -> PathBuf {
        let home = BaseDirs::new()
            .expect("xdg base dirs")
            .home_dir()
            .to_path_buf();
        home.join(".mozilla").join("native-messaging-hosts")
    }

    fn manifest_path_user() -> PathBuf {
        Self::mozilla_native_hosts_dir_user().join(format!("{}.json", HOST_NAME))
    }
}
