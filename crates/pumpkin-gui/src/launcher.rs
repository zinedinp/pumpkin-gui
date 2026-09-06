//! Resolves how to reach a server and connects

use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use pumpkin_gui_api::GUI_ENDPOINT_ENV;

use crate::config::{self, Connection};
use crate::probe;

/// Headline shown while nothing has failed yet: first run, or actively resolving/connecting.
pub const HEADLINE_SETUP: &str = "Connect to a Pumpkin server";
/// Headline for a resolution/connection failure with no single bad binary to blame.
const HEADLINE_FAILED: &str = "Could not reach a Pumpkin server";
/// Headline for a binary the user picked (or gui.conf named) that failed the probe.
const HEADLINE_BAD_BINARY: &str = "That file is not a GUI-capable Pumpkin server";

/// How many stderr lines from a managed child to keep, for the death backstop's error detail.
const STDERR_TAIL_LINES: usize = 50;

/// How far along resolving/connecting we are, for [`crate::SetupController`] to render.
#[derive(Debug, Clone)]
pub enum Status {
    Resolving,
    NeedsSetup,
    /// Running the `--version` probe.
    Validating(String),
    Launching(String),
    Connecting(String),
    Connected,
    /// Resolution or connection failed. Distinct from `NeedsSetup` so the window shows an error
    /// headline instead of the first-run copy, while keeping browse/attach as recovery.
    Failed { headline: String, detail: String },
}

static STATUS: Mutex<Status> = Mutex::new(Status::Resolving);
static MANAGED: AtomicBool = AtomicBool::new(false);
static MANAGED_CHILD: Mutex<Option<std::process::Child>> = Mutex::new(None);
static STDERR_TAIL: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

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

    // Config, then detection next to our own executable, then the setup/error screen.
    let mut reasons = Vec::new();

    match config::load() {
        Some(Connection::Managed { binary_path }) => {
            match validate_and_spawn(&binary_path, false) {
                Ok(()) => return,
                Err(reason) => reasons.push(format!("gui.conf points at {reason}")),
            }
        }
        Some(Connection::Attach { endpoint }) => {
            MANAGED.store(false, Ordering::Release);
            connect_with_retry(&endpoint);
            return;
        }
        None => {}
    }

    if let Some(candidate) = config::detect_binary_next_to_self() {
        match validate_and_spawn(&candidate, true) {
            Ok(()) => return,
            Err(reason) => reasons.push(reason),
        }
    }

    if reasons.is_empty() {
        set_status(Status::NeedsSetup);
    } else {
        set_status(Status::Failed {
            headline: HEADLINE_FAILED.to_owned(),
            detail: reasons.join("\n\n"),
        });
    }
}

/// Probes `path`, spawns it on success, and saves it to `gui.conf` first only when `save` is
/// true.
fn validate_and_spawn(path: &Path, save: bool) -> Result<(), String> {
    set_status(Status::Validating(format!("Checking {}…", path.display())));
    match probe::validate(path) {
        Ok(()) => {
            if save {
                let _ = config::save(&Connection::Managed {
                    binary_path: path.to_path_buf(),
                });
            }
            spawn_and_connect(path);
            Ok(())
        }
        Err(err) => Err(format!("{}: {err}", path.display())),
    }
}

/// Called from [`crate::SetupController::browse_for_binary`].
pub fn choose_binary_and_launch(binary_path: PathBuf) {
    std::thread::spawn(move || {
        set_status(Status::Validating(format!(
            "Checking {}…",
            binary_path.display()
        )));
        match probe::validate(&binary_path) {
            Ok(()) => {
                let _ = config::save(&Connection::Managed {
                    binary_path: binary_path.clone(),
                });
                spawn_and_connect(&binary_path);
            }
            Err(err) => set_status(Status::Failed {
                headline: HEADLINE_BAD_BINARY.to_owned(),
                detail: format!("{}: {err}", binary_path.display()),
            }),
        }
    });
}

/// Called from [`crate::SetupController::use_attach_endpoint`].
pub fn use_attach_endpoint(endpoint: String) {
    std::thread::spawn(move || {
        MANAGED.store(false, Ordering::Release);
        if connect_with_retry(&endpoint) {
            let _ = config::save(&Connection::Attach { endpoint });
        }
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
        // No flag: PUMPKIN_GUI_ENDPOINT alone puts a gui-feature server into IPC mode.
        .env(GUI_ENDPOINT_ENV, &endpoint)
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            STDERR_TAIL
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clear();
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || drain_stderr(stderr));
            }
            *MANAGED_CHILD.lock().unwrap_or_else(PoisonError::into_inner) = Some(child);
            connect_with_retry(&endpoint);
        }
        Err(err) => set_status(Status::Failed {
            headline: HEADLINE_FAILED.to_owned(),
            detail: format!("Could not start {}: {err}", binary_path.display()),
        }),
    }
}

/// Reads the managed slave's stderr line by line: keeps the last [`STDERR_TAIL_LINES`] for the
/// death backstop.
#[allow(clippy::print_stderr)]
fn drain_stderr(stderr: ChildStderr) {
    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
        eprintln!("{line}");
        let mut tail = STDERR_TAIL.lock().unwrap_or_else(PoisonError::into_inner);
        if tail.len() == STDERR_TAIL_LINES {
            tail.pop_front();
        }
        tail.push_back(line);
    }
}

fn stderr_tail() -> String {
    STDERR_TAIL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

/// `Some(status)` only once the managed child has actually exited
fn managed_child_exit_status() -> Option<std::process::ExitStatus> {
    let mut guard = MANAGED_CHILD.lock().unwrap_or_else(PoisonError::into_inner);
    guard
        .as_mut()
        .and_then(|child| child.try_wait().unwrap_or_default())
}

/// Returns `true` once connected.
fn connect_with_retry(endpoint: &str) -> bool {
    set_status(Status::Connecting(format!("Connecting to {endpoint}…")));

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        // If spawned and it has already exited, there is nothing left to wait for.
        if let Some(status) = managed_child_exit_status() {
            let tail = stderr_tail();
            set_status(Status::Failed {
                headline: "The server exited before the GUI could connect".to_owned(),
                detail: if tail.is_empty() {
                    status.to_string()
                } else {
                    format!("{status}\n\n{tail}")
                },
            });
            return false;
        }

        match crate::client::connect(endpoint) {
            Ok(mirror) => {
                crate::install(mirror);
                set_status(Status::Connected);
                return true;
            }
            Err(err) => {
                if Instant::now() >= deadline {
                    set_status(Status::Failed {
                        headline: HEADLINE_FAILED.to_owned(),
                        detail: format!("Could not connect to {endpoint}: {err}"),
                    });
                    return false;
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
