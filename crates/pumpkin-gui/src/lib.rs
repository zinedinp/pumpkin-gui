//! Optional Qt6 monitoring and console window for the Pumpkin server.
//!
//! This crate deliberately knows nothing about `pumpkin`'s `Server`: it connects to the running
//! server over a local IPC socket (see [`client`]) and the window only reads/sends messages. It
//! is also responsible for finding or starting that server
pub mod client;
mod config;
mod launcher;
mod qobjects;

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

pub use client::GuiMirror;
pub use pumpkin_gui_api::{
    DiskSpace, LogLevel, LogLine, PlayerRow, ServerMeta, Snapshot, SystemSampler, SystemStats,
    ThemePreference, WorldRow, directory_size,
};

static GUI: OnceLock<Arc<GuiMirror>> = OnceLock::new();

pub(crate) fn gui_side() -> Option<&'static Arc<GuiMirror>> {
    GUI.get()
}

/// Installs the resolved connection. Returns `false` if one was already installed (should not
/// happen: [`launcher`] only ever resolves once per process).
pub(crate) fn install(client: Arc<GuiMirror>) -> bool {
    GUI.set(client).is_ok()
}

/// QML entry point, resolved from the module URI declared in `build.rs`.
const MAIN_QML: &str = "qrc:/qt/qml/org/pumpkin/gui/qml/Main.qml";

/// Set by Qt if the root QML component fails to build.
static LOAD_FAILED: AtomicBool = AtomicBool::new(false);

/// Guards against [`run`] being called more than once in this process.
static STARTED: AtomicBool = AtomicBool::new(false);

/// True once the IPC connection has gone away (server shutdown or crash), so the event loop can
/// exit.
pub(crate) fn is_shutting_down() -> bool {
    gui_side().is_some_and(|gui| !gui.is_connected())
}

/// Why the GUI could not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiError {
    /// [`run`] was called more than once in this process.
    AlreadyRunning,
    /// Qt could not create a `QGuiApplication`, usually because there is no display.
    NoQtApplication,
    /// The root QML component failed to load, Qt has logged the details.
    QmlLoadFailed,
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::AlreadyRunning => "the GUI is already running in this process",
            Self::NoQtApplication => {
                "could not create a Qt application; is a display server available?"
            }
            Self::QmlLoadFailed => "the QML interface failed to load",
        };
        f.write_str(message)
    }
}

impl std::error::Error for GuiError {}

/// Starts the window and returns Qt's exit code once it closes.
///
/// Blocks until the window closes. Must be called on the process's main thread: Qt requires its
/// event loop there, and macOS enforces it.
///
/// # Errors
///
/// Returns [`GuiError`] if the GUI cannot be started at all.
pub fn run(attach: Option<String>) -> Result<i32, GuiError> {
    if STARTED.swap(true, Ordering::AcqRel) {
        return Err(GuiError::AlreadyRunning);
    }

    let mut app = cxx_qt_lib::QGuiApplication::new();
    let mut engine = cxx_qt_lib::QQmlApplicationEngine::new();

    let Some(mut engine) = engine.as_mut() else {
        return Err(GuiError::NoQtApplication);
    };

    let _guard = engine.as_mut().on_object_creation_failed(|_engine, _url| {
        LOAD_FAILED.store(true, Ordering::Release);
    });

    engine.load(&cxx_qt_lib::QUrl::from(MAIN_QML));

    if LOAD_FAILED.load(Ordering::Acquire) {
        return Err(GuiError::QmlLoadFailed);
    }

    launcher::start(attach);

    let code = if is_shutting_down() {
        Ok(0)
    } else {
        app.as_mut()
            .map(cxx_qt_lib::QGuiApplication::exec)
            .ok_or(GuiError::NoQtApplication)
    };

    launcher::shutdown_managed_child();
    code
}
