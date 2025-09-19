use crate::protocol::events::BrowserAction;
use crate::utils::themes;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{self, stdin, Read, Write};
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    action: String,
    success: bool,
    error: Option<String>,
    data: Option<T>,
}

#[derive(Debug, Serialize)]
pub struct ColorData {
    colors: Vec<String>,
    wallpaper: Option<String>,
}

pub fn read_message<T: DeserializeOwned + std::fmt::Debug>() -> Result<Option<T>> {
    let mut len_buf = [0u8; 4];
    if stdin().read_exact(&mut len_buf).is_err() {
        // EOF or no more input from browser; treat as graceful shutdown
        warn!("native messaging: EOF while reading message length");
        return Ok(None);
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    // Bound the maximum message length to avoid excessive allocation
    const MAX_MSG_LEN: usize = 64 * 1024; // 64 KiB
    if len == 0 || len > MAX_MSG_LEN {
        anyhow::bail!(
            "native messaging: invalid length {} (max {})",
            len,
            MAX_MSG_LEN
        );
    }
    let mut data = vec![0u8; len];
    stdin()
        .read_exact(&mut data)
        .context("reading native message body")?;
    let value = decode_message::<T>(&data)?;
    Ok(Some(value))
}

pub fn encode_message<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let data = serde_json::to_vec(value).context("serialize json")?;
    Ok(data)
}

pub fn decode_message<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let value = serde_json::from_slice::<T>(bytes).context("parsing json")?;
    Ok(value)
}

pub fn send_version() -> Result<()> {
    let response = Response {
        action: BrowserAction::Version.value().to_string(),
        success: true,
        error: None,
        data: Some(env!("CARGO_PKG_VERSION").to_string()),
    };
    info!("Sending =>  {:?}", response);
    write_message(&response)
}

pub fn send_colors() -> Result<()> {
    match themes::read_colors() {
        Ok(colors) => {
            let response = Response {
                action: BrowserAction::Colors.value().to_string(),
                success: true,
                error: None,
                data: Some(ColorData {
                    colors: colors.0,
                    wallpaper: colors.1,
                }),
            };
            info!("Sending =>  {:?}", response);
            write_message(&response)
        }
        Err(e) => {
            error!("Failed to load colors: {}", e);
            let err_msg = "Failed to load colors";
            send_error_response(BrowserAction::Colors, err_msg)
        }
    }
}

pub fn send_theme_mode(mode: &str) -> Result<()> {
    let response = Response {
        action: BrowserAction::ThemeMode.value().to_string(),
        success: true,
        error: None,
        data: Some(mode),
    };
    info!("Sending =>  {:?}", response);
    write_message(&response)
}

pub fn send_invalid_response() -> Result<()> {
    let response = build_invalid_response();
    info!("Sending =>  {:?}", response);
    write_message(&response)
}

fn write_message<T: Serialize>(value: &T) -> Result<()> {
    let data = encode_message(value)?;
    let len = data.len() as u32;
    let mut out = io::stdout();
    out.write_all(&len.to_le_bytes()).context("write len")?;
    out.write_all(&data).context("write body")?;
    out.flush()
        .map_err(|e| {
            warn!("native messaging: flush failed: {}", e);
            e
        })
        .context("flush stdout")?;
    Ok(())
}

fn send_error_response(action: BrowserAction, err: &str) -> Result<()> {
    let response = build_error_response(action, err);
    info!("Sending =>  {:?}", response);
    write_message(&response)
}

fn build_error_response(action: BrowserAction, err: &str) -> Response<String> {
    Response {
        action: action.value().to_string(),
        success: false,
        error: Some(err.to_string()),
        data: Some("Backend error".to_string()),
    }
}

fn build_invalid_response() -> Response<String> {
    Response {
        action: BrowserAction::Invalid.value().to_string(),
        success: false,
        error: None,
        data: Some("Invalid action".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct T {
        a: u32,
    }

    #[test]
    fn encode_decode_roundtrip() {
        let value = T { a: 42 };
        let buf = encode_message(&value).expect("encode");
        let out: T = decode_message(&buf).expect("decode");
        assert_eq!(value, out);
    }

    #[test]
    fn decode_rejects_invalid_json() {
        let bad = b"not json";
        let err = decode_message::<T>(bad).unwrap_err();
        let s = format!("{}", err);
        assert!(s.contains("parsing json"));
    }
}
