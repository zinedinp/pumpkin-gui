//! Resolves how to reach a server and connects

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use pumpkin_gui_api::GUI_ENDPOINT_ENV;

use crate::config::{self, Connection};

/// How far along resolving/connecting we are, for [`crate::SetupController`] to render.
#[derive(Debug, Clone)]
pub enum Status {
    Resolving,
    NeedsSetup,
    Launching(String),
    Connecting(String),
    Connected,
    Error(String),
}

static STATUS: Mutex<Status> = Mutex::new(Status::Resolving);
static MANAGED: AtomicBool = AtomicBool::new(false);
static MANAGED_CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);

#[must_use]
pub fn status() -> Status {
    STATUS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// True once a source (`gui.conf`, auto-detect, or the wizard) has resolved to spawning the
/// server
#[must_use]
pub fn is_managed() -> bool {
    MANAGED.load(Ordering::Acquire)
}

fn set_status(status: Status) {
    *STATUS.lock().unwrap_or_else(PoisonError::into_inner) = status;
}

/// Kicks off resolution on a background thread. `cli_attach` is the `--attach` flag
pub fn start(cli_attach: Option<String>) {
    std::thread::spawn(move || resolve(cli_attach));
}

fn resolve(cli_attach: Option<String>) {
    if let Ok(endpoint) = std::env::var(GUI_ENDPOINT_ENV) {
        MANAGED.store(false, Ordering::Release);
        connect_with_retry(&endpoint);
        return;
    }

    if let Some(endpoint) = cli_attach {
        MANAGED.store(false, Ordering::Release);
        connect_with_retry(&endpoint);
        return;
    }

    match config::load() {
        Some(Connection::Managed { binary_path }) => spawn_and_connect(&binary_path),
        Some(Connection::Attach { endpoint }) => {
            MANAGED.store(false, Ordering::Release);
            connect_with_retry(&endpoint);
        }
        None => match config::detect_binary_next_to_self() {
            Some(binary_path) => {
                let _ = config::save(&Connection::Managed {
                    binary_path: binary_path.clone(),
                });
                spawn_and_connect(&binary_path);
            }
            None => set_status(Status::NeedsSetup),
        },
    }
}

/// Called from [`crate::SetupController::browse_for_binary`].
pub fn choose_binary_and_launch(binary_path: PathBuf) {
    std::thread::spawn(move || {
        let _ = config::save(&Connection::Managed {
            binary_path: binary_path.clone(),
        });
        spawn_and_connect(&binary_path);
    });
}

/// Called from [`crate::SetupController::use_attach_endpoint`].
pub fn use_attach_endpoint(endpoint: String) {
    std::thread::spawn(move || {
        let _ = config::save(&Connection::Attach {
            endpoint: endpoint.clone(),
        });
        MANAGED.store(false, Ordering::Release);
        connect_with_retry(&endpoint);
    });
}

fn spawn_and_connect(binary_path: &Path) {
    MANAGED.store(true, Ordering::Release);
    set_status(Status::Launching(format!(
        "Starting {}…",
        binary_path.display()
    )));

    let endpoint = pumpkin_gui_api::unique_endpoint();
    let child = std::process::Command::new(binary_path)
        .arg("--gui")
        .env(GUI_ENDPOINT_ENV, &endpoint)
        .spawn();

    match child {
        Ok(child) => {
            *MANAGED_CHILD.lock().unwrap_or_else(PoisonError::into_inner) = Some(child);
            connect_with_retry(&endpoint);
        }
        Err(err) => set_status(Status::Error(format!(
            "Could not start {}: {err}",
            binary_path.display()
        ))),
    }
}

fn connect_with_retry(endpoint: &str) {
    set_status(Status::Connecting(format!("Connecting to {endpoint}…")));

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match crate::client::connect(endpoint) {
            Ok(mirror) => {
                crate::install(mirror);
                set_status(Status::Connected);
                return;
            }
            Err(err) => {
                if Instant::now() >= deadline {
                    set_status(Status::Error(format!(
                        "Could not connect to {endpoint}: {err}"
                    )));
                    return;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
}

/// Called once the Qt event loop has exited. A no-op unless this instance spawned the server
pub fn shutdown_managed_child() {
    let mut guard = MANAGED_CHILD.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(mut child) = guard.take() else {
        return;
    };
    drop(guard);

    for _ in 0..50 {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
}
