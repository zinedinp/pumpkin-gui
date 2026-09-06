//! Optional Qt6 monitoring and console window for the Pumpkin server.
//!
//! This crate deliberately knows nothing about `pumpkin`'s `Server`: the server fills a
//! [`Snapshot`] and hands over a [`GuiCommands`] sink, and the window only reads them. That keeps
//! the dependency one-directional (`pumpkin` -> `pumpkin-gui`)

mod qobjects;

pub use pumpkin_gui_api::{
    DiskSpace, GuiCommands, GuiSide, LogLevel, LogLine, LogRing, PlayerRow, ServerMeta, Snapshot,
    SystemSampler, SystemStats, ThemePreference, WorldRow, directory_size,
};

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// The handle the `QObject`s read from.
static GUI_SIDE: OnceLock<GuiSide> = OnceLock::new();

/// The active [`GuiSide`], or `None` if [`run`] has not been called.
pub(crate) fn gui_side() -> Option<&'static GuiSide> {
    GUI_SIDE.get()
}

/// QML entry point, resolved from the module URI declared in `build.rs`.
const MAIN_QML: &str = "qrc:/qt/qml/org/pumpkin/gui/qml/Main.qml";

/// Set by Qt if the root QML component fails to build.
static LOAD_FAILED: AtomicBool = AtomicBool::new(false);

/// Set when the server is shutting down (Ctrl+C, `stop`, window close) so the event loop can exit.
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Tells the window to leave the Qt event loop.
pub fn notify_shutdown() {
    SHUTTING_DOWN.store(true, Ordering::Release);
}

pub(crate) fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::Acquire)
}

/// Why the GUI could not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiError {
    /// [`run`] was called more than once in this process.
    AlreadyRunning,
    /// Qt could not create a `QGuiApplication`, usually because there is no display.
    NoQtApplication,
    /// The root QML component failed to load; Qt has logged the details.
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

/// Runs the GUI, returning Qt's exit code once the window closes.
///
/// Blocks until then. Must be called on the process's main thread: Qt requires its event loop
/// there, and macOS enforces it.
///
/// # Errors
///
/// Returns [`GuiError`] if the GUI cannot be started at all, so the caller can carry on headless
/// instead of sitting on an invisible event loop.
pub fn run(side: GuiSide) -> Result<i32, GuiError> {
    GUI_SIDE.set(side).map_err(|_| GuiError::AlreadyRunning)?;

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

    if is_shutting_down() {
        return Ok(0);
    }

    app.as_mut()
        .map(cxx_qt_lib::QGuiApplication::exec)
        .ok_or(GuiError::NoQtApplication)
}
