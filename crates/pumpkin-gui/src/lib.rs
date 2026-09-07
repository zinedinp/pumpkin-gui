//! Optional monitoring and console window for the Pumpkin server.
//!
//! This crate deliberately knows nothing about `pumpkin`'s `Server`: it connects to the running
//! server over a local IPC socket (see [`client`]) and the window only reads/sends messages. It
//! is also responsible for finding or starting that server.

mod app;
pub mod client;
mod command;
mod config;
mod format;
mod launcher;
mod probe;
mod tabs;
mod theme;
mod widgets;

pub use pumpkin_gui_api::{
    DiskSpace, LogLevel, LogLine, PlayerRow, PluginKind, PluginPermission, PluginRow, PluginState,
    ServerMeta, Snapshot, StyledRun, SystemStats, ThemePreference, WorldRow,
};

/// Why the window could not start.
#[derive(Debug)]
pub enum GuiError {
    /// The renderer could not create a window, usually because there is no display.
    NoDisplay(iced::Error),
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDisplay(err) => write!(f, "could not open a window: {err}"),
        }
    }
}

impl std::error::Error for GuiError {}

/// Starts the window and returns an exit code once it closes.
///
/// Blocks until the window closes. Must be called on the process's main thread: the windowing
/// system requires its event loop there, and macOS enforces it.
///
/// # Errors
///
/// Returns [`GuiError`] if the window cannot be started at all.
pub fn run(attach: Option<String>) -> Result<i32, GuiError> {
    let result = app::run(attach);

    // Whatever happened to the window, a server this process started must not outlive it.
    launcher::shutdown_managed_child();

    result.map(|()| 0).map_err(GuiError::NoDisplay)
}
