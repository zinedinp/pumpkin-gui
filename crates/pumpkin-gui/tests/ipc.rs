//! End-to-end check of the IPC wire protocol against a hand-rolled fake server: exercises the
//! same `pumpkin_gui_api` framing and `pumpkin_gui::client` code the real server/GUI use, without
//! needing a display or a running Minecraft server.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use pumpkin_gui::client::{self, Event};
use pumpkin_gui_api::{
    GuiMessage, LogLevel, LogLine, ServerMessage, ServerMeta, StyledRun, ThemePreference,
};
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};

fn socket_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("pumpkin-gui-test-{}.sock", std::process::id()))
}

async fn accept_one(listener: UnixListener) -> UnixStream {
    let (stream, _addr) = listener.accept().await.expect("accept");
    stream
}

#[test]
fn connect_receives_hello_snapshot_and_logs() {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);

    // Multi-threaded so the spawned fake-server task actually runs while the main test thread
    // blocks in `pumpkin_gui::client::connect` below.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("runtime");
    let _guard = runtime.enter();
    let listener = UnixListener::bind(&path).expect("bind");

    let server = runtime.spawn(serve_one_session(listener));

    let (events_tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    client::connect(&path.to_string_lossy(), events_tx).expect("client connect");

    // The connection reports itself before anything else, and hands over the only way to talk
    // back at the same time.
    let outbound = match events.blocking_recv().expect("connected event") {
        Event::Connected {
            meta,
            theme,
            outbound,
        } => {
            assert_eq!(theme, ThemePreference::Dark);
            assert_eq!(meta.pumpkin_version, "test");
            outbound
        }
        other => panic!("expected Connected, got {other:?}"),
    };

    let mut saw_snapshot = false;
    let mut saw_log = false;
    let mut saw_completions = false;

    outbound.send(GuiMessage::Complete {
        id: 7,
        line: "sto".to_owned(),
        cursor: 3,
    });

    while !(saw_snapshot && saw_log && saw_completions) {
        match events.blocking_recv().expect("event") {
            Event::Server(message) => match *message {
                ServerMessage::Snapshot(snapshot) => {
                    assert!((snapshot.tps - 20.0).abs() < f64::EPSILON);
                    saw_snapshot = true;
                }
                ServerMessage::LogLines(lines) => {
                    assert_eq!(lines.len(), 1);
                    assert_eq!(lines[0].message, "hello");
                    // The colour runs survive the round trip, not just the plain text.
                    assert_eq!(lines[0].runs.len(), 1);
                    assert_eq!(lines[0].runs[0].text, "hello");
                    saw_log = true;
                }
                ServerMessage::Completions { id, candidates } => {
                    assert_eq!(id, 7);
                    assert_eq!(candidates, vec!["stop".to_owned()]);
                    saw_completions = true;
                }
                other => panic!("unexpected server message: {other:?}"),
            },
            other => panic!("unexpected event: {other:?}"),
        }
    }

    outbound.submit("list".to_owned());

    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .expect("server task timed out")
            .expect("server task panicked");
    });

    let _ = std::fs::remove_file(&path);
}

/// The fake server: greets, pushes a snapshot and a log line, then answers what the client sends
/// until it submits a command.
async fn serve_one_session(listener: UnixListener) {
    let mut stream = accept_one(listener).await;

    pumpkin_gui_api::write_message(
        &mut stream,
        &ServerMessage::Hello {
            protocol: pumpkin_gui_api::PROTOCOL_VERSION,
            meta: ServerMeta {
                pumpkin_version: "test".to_owned(),
                ..Default::default()
            },
            theme: ThemePreference::Dark,
        },
    )
    .await
    .expect("write hello");

    pumpkin_gui_api::write_message(
        &mut stream,
        &ServerMessage::Snapshot(pumpkin_gui_api::Snapshot {
            server_ready: true,
            tps: 20.0,
            ..Default::default()
        }),
    )
    .await
    .expect("write snapshot");

    pumpkin_gui_api::write_message(
        &mut stream,
        &ServerMessage::LogLines(vec![LogLine {
            seq: 0,
            level: LogLevel::Info,
            target: "console".to_owned(),
            message: "hello".to_owned(),
            runs: vec![StyledRun {
                text: "hello".to_owned(),
                ..Default::default()
            }],
        }]),
    )
    .await
    .expect("write log line");

    let mut buf = Vec::new();
    loop {
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        buf.resize(len, 0);
        if stream.read_exact(&mut buf).await.is_err() {
            break;
        }

        let msg: GuiMessage = postcard::from_bytes(&buf).expect("decode gui message");
        match msg {
            GuiMessage::Complete { id, .. } => {
                pumpkin_gui_api::write_message(
                    &mut stream,
                    &ServerMessage::Completions {
                        id,
                        candidates: vec!["stop".to_owned()],
                    },
                )
                .await
                .expect("write completions");
            }
            GuiMessage::Submit(line) => {
                assert_eq!(line, "list");
                return;
            }
            // Listed rather than caught by a wildcard, so a new protocol variant fails to compile
            // here instead of silently reaching a generic panic at runtime.
            GuiMessage::RequestStop => panic!("unexpected RequestStop"),
            GuiMessage::ReadConfig(file) => panic!("unexpected ReadConfig({file:?})"),
            GuiMessage::WriteConfig { file, .. } => panic!("unexpected WriteConfig({file:?})"),
        }
    }
}

/// A server built from different sources must be rejected at the handshake, not read as garbage.
#[test]
fn a_protocol_mismatch_is_refused_with_both_versions_named() {
    let path = std::env::temp_dir().join(format!("pumpkin-gui-skew-{}.sock", std::process::id()));
    let _ = std::fs::remove_file(&path);

    let listen_path = path.clone();
    let server = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async move {
            let listener = tokio::net::UnixListener::bind(&listen_path).expect("bind");
            let (mut stream, _) = listener.accept().await.expect("accept");
            pumpkin_gui_api::write_message(
                &mut stream,
                &ServerMessage::Hello {
                    protocol: pumpkin_gui_api::PROTOCOL_VERSION + 1,
                    meta: ServerMeta::default(),
                    theme: ThemePreference::Dark,
                },
            )
            .await
            .expect("write hello");
        });
    });

    // Give the listener a moment to bind before connecting.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let (events_tx, _events) = tokio::sync::mpsc::unbounded_channel();
    let text = match client::connect(&path.to_string_lossy(), events_tx) {
        Ok(()) => panic!("a skewed server must not be accepted"),
        Err(err) => err.to_string(),
    };

    assert!(text.contains("protocol"), "unhelpful message: {text}");
    assert!(
        text.contains(&(pumpkin_gui_api::PROTOCOL_VERSION + 1).to_string()),
        "the server's version must be named: {text}"
    );

    server.join().expect("server thread");
    let _ = std::fs::remove_file(&path);
}
