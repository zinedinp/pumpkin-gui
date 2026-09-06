// Build scripts talk to Cargo over stdout and abort by panicking
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

const QML_URI: &str = "org.pumpkin.gui";

const MISSING_QT: &str = "\
Qt6 not found! pumpkin-gui needs Qt 6.5+ with QtDeclarative and QtSvg.

  Arch:          pacman -S qt6-base qt6-declarative qt6-svg
  Debian/Ubuntu: apt install qt6-base-dev qt6-declarative-dev libqt6svg6-dev
  Fedora:        dnf install qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtsvg-devel

Set QMAKE=/path/to/qmake6 to point at a custom Qt installation,
or build without --features gui.";

fn main() {
    preflight();

    let module = QmlModule::new(QML_URI)
        // Singletons must be declared here, not just via `pragma Singleton`, so cxx-qt writes the
        // matching `singleton` entries into the generated qmldir.
        .qml_file(QmlFile::from("qml/Theme.qml").singleton(true))
        .qml_file(QmlFile::from("qml/Format.qml").singleton(true))
        .qml_file(QmlFile::from("qml/Icons.qml").singleton(true))
        .qml_files([
            "qml/Main.qml",
            "qml/Overview.qml",
            "qml/Performance.qml",
            "qml/StatTile.qml",
            "qml/ConsoleView.qml",
            "qml/CommandInput.qml",
            "qml/ThemedField.qml",
            "qml/ThemedButton.qml",
            "qml/ThemedCheckBox.qml",
            "qml/ThemedComboBox.qml",
            "qml/ThemedTabButton.qml",
            "qml/CopyableText.qml",
            "qml/ConfirmDialog.qml",
            "qml/CoreGrid.qml",
            "qml/TickGraph.qml",
            "qml/Card.qml",
            "qml/Meter.qml",
            "qml/Badge.qml",
            "qml/CountChip.qml",
            "qml/Placeholder.qml",
            "qml/Worlds.qml",
            "qml/Players.qml",
            "qml/PlayerRowDelegate.qml",
            "qml/EditionTag.qml",
            "qml/TableVRule.qml",
            "qml/IconButton.qml",
            "qml/ReasonDialog.qml",
        ]);

    // SAFETY: include paths or sources changes.
    unsafe {
        CxxQtBuilder::new_qml_module(module)
            .files([
                "src/qobjects/server_stats.rs",
                "src/qobjects/players.rs",
                "src/qobjects/dev_tools.rs",
                "src/qobjects/console.rs",
            ])
            // Embedded rather than loaded from disk so the binary stays self-contained. The QML
            // module supplies the `/qt/qml/org/pumpkin/gui` prefix.
            .qrc_resources([
                "qml/icons/op.svg",
                "qml/icons/kick.svg",
                "qml/icons/ban.svg",
                "qml/icons/copy.svg",
            ])
            .qt_module("Svg")
            // GCC 16 + Qt headers: QChar is incomplete in a SFINAE check inside libstdc++
            // (`std::ranges::data`). Qt itself silences this (QTBUG-143470)
            .cc_builder(|cc| {
                cc.flag_if_supported("-Wno-sfinae-incomplete");
            })
            .build();
    }
}

/// Fails with an actionable message instead of letting the C++ linker complain about Qt.
///
/// Only meaningful on Linux, where Qt comes from distribution packages. Windows and macOS get Qt
/// from vcpkg, which `cxx-qt-build` locates itself.
fn preflight() {
    println!("cargo:rerun-if-env-changed=QMAKE");

    if std::env::var_os("QMAKE").is_some() {
        // pointed at a specific Qt; cxx-qt-build will validate it.
        return;
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        return;
    }

    let Some(qmake) = ["qmake6", "qmake"].into_iter().find(|bin| {
        Command::new(bin)
            .arg("-query")
            .arg("QT_VERSION")
            .output()
            .is_ok_and(|out| out.status.success())
    }) else {
        panic!("{MISSING_QT}");
    };

    let version = qmake_query(qmake, "QT_VERSION").unwrap_or_default();
    assert!(
        version.starts_with('6'),
        "{MISSING_QT}\n\nFound Qt {version} via `{qmake}`, but Qt 6.5+ is required."
    );

    // QtQuick and QtSvg checked separately
    let Some(lib_dir) = qmake_query(qmake, "QT_INSTALL_LIBS").map(PathBuf::from) else {
        return;
    };

    let missing: Vec<&str> = ["Quick", "Qml", "Svg"]
        .into_iter()
        .filter(|module| !lib_dir.join(format!("libQt6{module}.so")).exists())
        .collect();

    assert!(
        missing.is_empty(),
        "{MISSING_QT}\n\nFound Qt {version} in {}, but these modules are missing: {}",
        lib_dir.display(),
        missing.join(", ")
    );
}

fn qmake_query(qmake: &str, key: &str) -> Option<String> {
    let output = Command::new(qmake).arg("-query").arg(key).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
