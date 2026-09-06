//! Validates that a candidate path is a GUI-capable Pumpkin server, before it is persisted to
//! `gui.conf` or spawned.

use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use pumpkin_gui_api::GUI_ENDPOINT_ENV;

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(50);

/// Why a candidate binary was rejected.
#[derive(Debug)]
pub enum ProbeError {
    /// The file could not be executed at all.
    Spawn(std::io::Error),
    /// It did not answer `--version` within [`PROBE_TIMEOUT`].
    Timeout,
    /// It exited non-zero.
    Failed { status: String, stderr: String },
    /// It ran, but its output is not a Pumpkin version line.
    NotPumpkin { first_line: String },
    /// It is a Pumpkin server, but built without the `gui` feature.
    NoGuiFeature { version: String },
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(err) => write!(f, "could not run it: {err}"),
            Self::Timeout => write!(f, "it did not answer --version in time"),
            Self::Failed { status, stderr } => {
                write!(f, "it exited with {status}")?;
                if !stderr.is_empty() {
                    write!(f, ": {stderr}")?;
                }
                Ok(())
            }
            Self::NotPumpkin { first_line } => {
                write!(f, "this does not look like a Pumpkin server (it printed: {first_line})")
            }
            Self::NoGuiFeature { version } => write!(
                f,
                "this Pumpkin build has no GUI support ({version}); rebuild it with \
                 `cargo build --release --features gui`"
            ),
        }
    }
}

/// Runs `<path> --version` and decides whether it is a server this GUI can drive.
pub fn validate(path: &Path) -> Result<(), ProbeError> {
    let mut child = Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove(GUI_ENDPOINT_ENV)
        .spawn()
        .map_err(ProbeError::Spawn)?;

    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ProbeError::Timeout);
            }
            Ok(None) => std::thread::sleep(POLL),
            Err(err) => return Err(ProbeError::Spawn(err)),
        }
    }

    let output = child.wait_with_output().map_err(ProbeError::Spawn)?;
    classify(&output)
}

/// The pure half of [`validate`], so the decision table is testable without spawning anything.
fn classify(output: &Output) -> Result<(), ProbeError> {
    if !output.status.success() {
        return Err(ProbeError::Failed {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or_default().trim();

    if !pumpkin_gui_api::is_pumpkin_version_line(first_line) {
        return Err(ProbeError::NotPumpkin {
            first_line: first_line.to_owned(),
        });
    }
    if !pumpkin_gui_api::is_gui_capable_version_line(first_line) {
        return Err(ProbeError::NoGuiFeature {
            version: first_line.to_owned(),
        });
    }
    Ok(())
}
