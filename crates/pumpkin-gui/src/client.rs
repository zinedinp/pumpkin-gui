//! The background IPC connection and the local mirror the `QObject`s read from.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use arc_swap::ArcSwap;
use pumpkin_gui_api::{
    GuiMessage, LogLine, RequestId, ServerMessage, ServerMeta, Snapshot, ThemePreference,
    read_message, write_message,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

/// How long [`GuiMirror::completions`] waits for a response before giving up.
const COMPLETIONS_TIMEOUT: Duration = Duration::from_millis(500);

pub struct LocalLogRing {
    inner: Mutex<LocalLogRingInner>,
}

struct LocalLogRingInner {
    lines: VecDeque<LogLine>,
    capacity: usize,
    next_seq: u64,
}

impl LocalLogRing {
    const fn new() -> Self {
        let capacity = 20_000;
        Self {
            inner: Mutex::new(LocalLogRingInner {
                lines: VecDeque::new(),
                capacity,
                next_seq: 0,
            }),
        }
    }

    fn extend(&self, new_lines: Vec<LogLine>) {
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        for line in new_lines {
            inner.next_seq = line.seq + 1;
            inner.lines.push_back(line);
        }
        while inner.lines.len() > inner.capacity {
            inner.lines.pop_front();
        }
    }

    /// Appends every line newer than `cursor` to `out` and returns the new cursor. Mirrors
    /// `pumpkin_gui_api::LogRing::drain_since`'s contract exactly.
    pub fn drain_since(&self, cursor: u64, out: &mut Vec<LogLine>) -> u64 {
        let inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        out.extend(
            inner
                .lines
                .iter()
                .filter(|line| line.seq >= cursor)
                .cloned(),
        );
        inner.next_seq
    }
}

/// The connection the `QObject`s read from and send actions through.
pub struct GuiMirror {
    pub snapshot: Arc<ArcSwap<Snapshot>>,
    pub meta: Arc<ArcSwap<ServerMeta>>,
    pub theme: ThemePreference,
    logs: LocalLogRing,
    outbound: mpsc::UnboundedSender<GuiMessage>,
    next_request_id: AtomicU32,
    pending_completions: Mutex<Option<(RequestId, std::sync::mpsc::Sender<Vec<String>>)>>,
    connected: AtomicBool,
}

impl GuiMirror {
    #[must_use]
    pub const fn logs(&self) -> &LocalLogRing {
        &self.logs
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    /// Runs a console command, exactly as if it had been typed in the terminal.
    pub fn submit(&self, line: String) {
        let _ = self.outbound.send(GuiMessage::Submit(line));
    }

    /// Begins a graceful shutdown.
    pub fn request_stop(&self) {
        let _ = self.outbound.send(GuiMessage::RequestStop);
    }

    /// Tab-completion candidates for `line` at `cursor`.
    pub fn completions(&self, line: &str, cursor: usize) -> Vec<String> {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = std::sync::mpsc::channel();
        *self
            .pending_completions
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some((id, tx));

        let sent = self.outbound.send(GuiMessage::Complete {
            id,
            line: line.to_owned(),
            cursor,
        });
        if sent.is_err() {
            return Vec::new();
        }

        rx.recv_timeout(COMPLETIONS_TIMEOUT).unwrap_or_default()
    }

    fn resolve_completions(&self, id: RequestId, candidates: Vec<String>) {
        let mut pending = self
            .pending_completions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let matches_id = matches!(pending.as_ref(), Some((pending_id, _)) if *pending_id == id);
        if matches_id && let Some((_, tx)) = pending.take() {
            let _ = tx.send(candidates);
        }
    }
}

/// Connects to `endpoint` (a Unix socket path or Windows named-pipe name) and blocks until the
/// server's initial handshake arrives, returning the resulting mirror.
///
/// # Errors
///
/// Returns an error if the connection or handshake fails.
pub fn connect(endpoint: &str) -> std::io::Result<Arc<GuiMirror>> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<std::io::Result<Arc<GuiMirror>>>();
    let endpoint = endpoint.to_owned();

    std::thread::Builder::new()
        .name("pumpkin-gui-ipc".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            runtime.block_on(run_connection(&endpoint, &ready_tx));
        })
        .map_err(std::io::Error::other)?;

    ready_rx
        .recv()
        .unwrap_or_else(|_| Err(std::io::Error::other("IPC thread exited before connecting")))
}

async fn run_connection(
    endpoint: &str,
    ready_tx: &std::sync::mpsc::Sender<std::io::Result<Arc<GuiMirror>>>,
) {
    let stream = match connect_stream(endpoint).await {
        Ok(stream) => stream,
        Err(err) => {
            let _ = ready_tx.send(Err(err));
            return;
        }
    };

    let (mut read_half, write_half) = tokio::io::split(stream);

    let (meta, theme) = match read_message::<_, ServerMessage>(&mut read_half).await {
        Ok(ServerMessage::Hello { meta, theme }) => (meta, theme),
        Ok(_) => {
            let _ = ready_tx.send(Err(std::io::Error::other(
                "expected Hello as the server's first message",
            )));
            return;
        }
        Err(err) => {
            let _ = ready_tx.send(Err(std::io::Error::other(err.to_string())));
            return;
        }
    };

    let (out_tx, out_rx) = mpsc::unbounded_channel::<GuiMessage>();
    let mirror = Arc::new(GuiMirror {
        snapshot: Arc::new(ArcSwap::from_pointee(Snapshot::default())),
        meta: Arc::new(ArcSwap::from_pointee(meta)),
        theme,
        logs: LocalLogRing::new(),
        outbound: out_tx,
        next_request_id: AtomicU32::new(0),
        pending_completions: Mutex::new(None),
        connected: AtomicBool::new(true),
    });

    if ready_tx.send(Ok(mirror.clone())).is_err() {
        // The caller gave up waiting, nothing left to hand the connection to.
        return;
    }

    let writer = tokio::spawn(writer_loop(write_half, out_rx));
    reader_loop(read_half, &mirror).await;
    mirror.connected.store(false, Ordering::Release);
    writer.abort();
}

#[cfg(unix)]
async fn connect_stream(endpoint: &str) -> std::io::Result<tokio::net::UnixStream> {
    tokio::net::UnixStream::connect(endpoint).await
}

#[cfg(windows)]
async fn connect_stream(
    endpoint: &str,
) -> std::io::Result<tokio::net::windows::named_pipe::NamedPipeClient> {
    tokio::net::windows::named_pipe::ClientOptions::new().open(endpoint)
}

async fn reader_loop<R: AsyncRead + Unpin>(mut read_half: R, mirror: &Arc<GuiMirror>) {
    loop {
        match read_message::<_, ServerMessage>(&mut read_half).await {
            Ok(ServerMessage::Snapshot(snapshot)) => mirror.snapshot.store(Arc::new(snapshot)),
            Ok(ServerMessage::LogLines(lines)) => mirror.logs.extend(lines),
            Ok(ServerMessage::Completions { id, candidates }) => {
                mirror.resolve_completions(id, candidates);
            }
            // `Hello` is only ever sent once, immediately after accept, seeing it again here
            // would be a protocol violation, so it ends the connection just like a shutdown or a
            // read error would.
            Ok(ServerMessage::ShuttingDown | ServerMessage::Hello { .. }) | Err(_) => break,
        }
    }
}

async fn writer_loop<W: AsyncWrite + Unpin>(
    mut write_half: W,
    mut rx: mpsc::UnboundedReceiver<GuiMessage>,
) {
    while let Some(msg) = rx.recv().await {
        if write_message(&mut write_half, &msg).await.is_err() {
            break;
        }
    }
}
