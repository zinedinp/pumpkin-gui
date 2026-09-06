//! Persisted connection preference (`gui.conf`)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "gui.conf";

#[cfg(windows)]
const SERVER_BIN_NAME: &str = "pumpkin.exe";
#[cfg(not(windows))]
const SERVER_BIN_NAME: &str = "pumpkin";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Connection {
    Managed { binary_path: PathBuf },
    Attach { endpoint: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct GuiConfigFile {
    connection: Connection,
}

fn config_path() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| {
        std::io::Error::other("the pumpkin-gui executable has no parent directory")
    })?;
    Ok(dir.join(FILE_NAME))
}

/// if config available
#[must_use]
pub fn load() -> Option<Connection> {
    let path = config_path().ok()?;
    let text = std::fs::read_to_string(path).ok()?;
    let file: GuiConfigFile = toml::from_str(&text).ok()?;
    Some(file.connection)
}

/// Writes config
pub fn save(connection: &Connection) -> std::io::Result<()> {
    let path = config_path()?;
    let file = GuiConfigFile {
        connection: connection.clone(),
    };
    let text = toml::to_string_pretty(&file).map_err(std::io::Error::other)?;
    std::fs::write(path, text)
}

/// Looks for a `pumpkin`/`pumpkin.exe` binary next to the `pumpkin-gui` executable.
#[must_use]
pub fn detect_binary_next_to_self() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join(SERVER_BIN_NAME);
    candidate.is_file().then_some(candidate)
}
