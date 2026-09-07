//! The background IPC connection.
//!
//! The connection lives on its own thread with its own runtime; everything it learns reaches the
//! window as an [`Event`] rather than through shared state, so the application owns the data and
//! nothing has to be polled.

use pumpkin_gui_api::{
    GuiMessage, PROTOCOL_VERSION, ServerMessage, ServerMeta, ThemePreference, read_message,
    write_message,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

/// What the window learns from the backend, in the order it happens.
#[derive(Debug, Clone)]
pub enum Event {
    /// Progress while resolving and connecting; see [`crate::launcher::Status`].
    Status(crate::launcher::Status),
    /// The handshake succeeded. Carries the only handle for talking back.
    Connected {
        meta: Box<ServerMeta>,
        theme: ThemePreference,
        outbound: Outbound,
    },
    Server(Box<ServerMessage>),
    /// The server shut down, crashed, or the stream broke.
    Disconnected,
}

/// The write end of the connection, cloned into the application state.
#[derive(Debug, Clone)]
pub struct Outbound(mpsc::UnboundedSender<GuiMessage>);

impl Outbound {
    /// Runs a console command, exactly as if it had been typed in the terminal.
    pub fn submit(&self, line: String) {
        self.send(GuiMessage::Submit(line));
    }

    /// Begins a graceful shutdown.
    pub fn request_stop(&self) {
        self.send(GuiMessage::RequestStop);
    }

    pub fn send(&self, message: GuiMessage) {
        // A closed channel means the connection is already gone, which the window learns from
        // `Disconnected` anyway.
        let _ = self.0.send(message);
    }
}

/// Where [`Event`]s are delivered. Sending is non-blocking, so the connection never waits on the
/// window.
pub type Events = mpsc::UnboundedSender<Event>;

/// Connects to `endpoint` (a Unix socket path or Windows named-pipe name) and blocks until the
/// server's initial handshake either succeeds or fails.
///
/// On success the connection keeps running on its own thread, emitting [`Event`]s until it ends.
///
/// # Errors
///
/// Returns an error if the connection or handshake fails, so the caller can retry.
pub fn connect(endpoint: &str, events: Events) -> std::io::Result<()> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<std::io::Result<()>>();
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
            runtime.block_on(run_connection(&endpoint, &ready_tx, &events));
        })
        .map_err(std::io::Error::other)?;

    ready_rx
        .recv()
        .unwrap_or_else(|_| Err(std::io::Error::other("IPC thread exited before connecting")))
}

async fn run_connection(
    endpoint: &str,
    ready_tx: &std::sync::mpsc::Sender<std::io::Result<()>>,
    events: &Events,
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
        Ok(ServerMessage::Hello {
            protocol,
            meta,
            theme,
        }) => {
            // postcard decodes a skewed variant list silently wrong, so stop at the handshake
            // rather than misreading everything after it.
            if protocol != PROTOCOL_VERSION {
                let _ = ready_tx.send(Err(std::io::Error::other(format!(
                    "the server speaks GUI protocol v{protocol}, this window speaks \
                     v{PROTOCOL_VERSION}; rebuild both from the same source"
                ))));
                return;
            }
            (meta, theme)
        }
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
    let connected = events.send(Event::Connected {
        meta: Box::new(meta),
        theme,
        outbound: Outbound(out_tx),
    });

    // Tell the caller only after the window has been handed the connection, so a retry loop
    // cannot see success while the window still has no way to talk back.
    if ready_tx.send(Ok(())).is_err() || connected.is_err() {
        return;
    }

    let writer = tokio::spawn(writer_loop(write_half, out_rx));
    reader_loop(read_half, events).await;
    let _ = events.send(Event::Disconnected);
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

async fn reader_loop<R: AsyncRead + Unpin>(mut read_half: R, events: &Events) {
    loop {
        match read_message::<_, ServerMessage>(&mut read_half).await {
            // `Hello` is only ever sent once, immediately after accept; seeing it again would be
            // a protocol violation, so it ends the connection just like a shutdown or a read
            // error would.
            Ok(ServerMessage::ShuttingDown | ServerMessage::Hello { .. }) | Err(_) => break,
            Ok(message) => {
                if events.send(Event::Server(Box::new(message))).is_err() {
                    break;
                }
            }
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
