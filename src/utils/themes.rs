use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct ColorFile {
    colors: Vec<String>,
    wallpaper: Option<String>,
}

pub fn read_colors() -> Result<(Vec<String>, Option<String>)> {
    let path = colors_path();
    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("color definition not found at {}", path.display());
        }
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let parsed: ColorFile = serde_json::from_str(&data).context("json parse color definition")?;

    if parsed.colors.len() < 16 {
        warn!("color definition contains fewer than 16 colors");
    }

    info!(
        "loaded {} colors from {}",
        parsed.colors.len(),
        path.display()
    );
    Ok((parsed.colors, parsed.wallpaper))
}
fn colors_default_path() -> PathBuf {
    let home = directories::BaseDirs::new()
        .expect("xdg base")
        .home_dir()
        .to_path_buf();
    home.join(".cache").join("wal").join("walrusfox.json")
}

fn colors_path() -> PathBuf {
    if let Ok(p) = std::env::var("WALRUSFOX_COLORS") {
        return PathBuf::from(p);
    }
    colors_default_path()
}
