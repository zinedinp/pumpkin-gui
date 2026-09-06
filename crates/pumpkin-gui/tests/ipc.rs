//! End-to-end check of the IPC wire protocol against a hand-rolled fake server: exercises the
//! same `pumpkin_gui_api` framing and `pumpkin_gui::client` code the real server/GUI use, without
//! needing a Qt display or a running Minecraft server.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use pumpkin_gui_api::{GuiMessage, LogLevel, LogLine, ServerMessage, ServerMeta, ThemePreference};
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

    let server = runtime.spawn(async move {
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
                html: "hello".to_owned(),
            }]),
        )
        .await
        .expect("write log line");

        // Answers whatever the client sends, so `submit`/`completions` can be checked too.
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
                GuiMessage::RequestStop => panic!("unexpected RequestStop"),
            }
        }
    });

    let mirror = pumpkin_gui::client::connect(&path.to_string_lossy()).expect("client connect");

    assert_eq!(mirror.theme, ThemePreference::Dark);
    assert_eq!(mirror.meta.load().pumpkin_version, "test");

    // The reader task races the assertions below; give it a moment to catch up.
    std::thread::sleep(Duration::from_millis(200));
    assert!((mirror.snapshot.load().tps - 20.0).abs() < f64::EPSILON);

    let mut lines = Vec::new();
    let cursor = mirror.logs().drain_since(0, &mut lines);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].message, "hello");
    assert_eq!(cursor, 1);

    let candidates = mirror.completions("sto", 3);
    assert_eq!(candidates, vec!["stop".to_owned()]);

    mirror.submit("list".to_owned());

    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .expect("server task timed out")
            .expect("server task panicked");
    });

    let _ = std::fs::remove_file(&path);
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

    let text = match pumpkin_gui::client::connect(&path.to_string_lossy()) {
        Ok(_) => panic!("a skewed server must not be accepted"),
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
