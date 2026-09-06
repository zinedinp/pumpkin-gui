//! The online-player table and the actions on it.

// cxx-qt expands into generated glue that does not follow the workspace's lint profile.
#![allow(
    clippy::used_underscore_binding,
    clippy::unnecessary_box_returns,
    clippy::needless_lifetimes,
    clippy::multiple_unsafe_ops_per_block,
    clippy::undocumented_unsafe_blocks
)]

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qlist.h");
        type QList_QVariant = cxx_qt_lib::QList<QVariant>;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// One entry per online player, each a map QML reads by key.
        #[qproperty(QList_QVariant, rows)]
        /// False while the server is still starting, so QML can disable the action buttons.
        #[qproperty(bool, has_commands)]
        type PlayerList = super::PlayerListRust;

        /// Pulls the newest player list. Driven by a QML `Timer`.
        #[qinvokable]
        fn refresh(self: Pin<&mut Self>);

        /// Grants operator status.
        #[qinvokable]
        fn op(&self, name: &QString);

        /// Revokes operator status.
        #[qinvokable]
        fn deop(&self, name: &QString);

        /// Disconnects a player; an empty `reason` uses the server default.
        #[qinvokable]
        fn kick(&self, name: &QString, reason: &QString);

        /// Bans a player; an empty `reason` uses the server default.
        #[qinvokable]
        fn ban(&self, name: &QString, reason: &QString);

        /// Adds a name to the whitelist (`whitelist add`).
        #[qinvokable]
        fn whitelist(&self, name: &QString);

        /// Removes a name from the whitelist (`whitelist remove`).
        #[qinvokable]
        fn unwhitelist(&self, name: &QString);

        /// Lifts a player ban (`pardon`).
        #[qinvokable]
        fn pardon(&self, name: &QString);
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QMap, QMapPair_QString_QVariant, QString, QVariant};

#[derive(Default)]
pub struct PlayerListRust {
    rows: QList<QVariant>,
    has_commands: bool,
    /// What was last published, so an unchanged list is not rebuilt.
    last: Vec<crate::PlayerRow>,
}

impl qobject::PlayerList {
    pub fn refresh(mut self: Pin<&mut Self>) {
        let Some(side) = crate::gui_side() else {
            return;
        };

        let has_commands = side.commands().is_some();
        if *self.as_ref().has_commands() != has_commands {
            self.as_mut().set_has_commands(has_commands);
        }

        let snapshot = side.snapshot.load();
        if self.as_ref().rust().last == snapshot.players {
            return;
        }

        let rows = snapshot.players.iter().map(player_to_variant).fold(
            QList::<QVariant>::default(),
            |mut list, row| {
                list.append(row);
                list
            },
        );

        self.as_mut().rust_mut().last.clone_from(&snapshot.players);
        self.as_mut().set_rows(rows);
    }

    pub fn op(&self, name: &QString) {
        self.run_targeted("op", name, None);
    }

    pub fn deop(&self, name: &QString) {
        self.run_targeted("deop", name, None);
    }

    pub fn kick(&self, name: &QString, reason: &QString) {
        self.run_targeted("kick", name, Some(reason));
    }

    pub fn ban(&self, name: &QString, reason: &QString) {
        self.run_targeted("ban", name, Some(reason));
    }

    pub fn whitelist(&self, name: &QString) {
        self.run_targeted("whitelist add", name, None);
    }

    pub fn unwhitelist(&self, name: &QString) {
        self.run_targeted("whitelist remove", name, None);
    }

    pub fn pardon(&self, name: &QString) {
        self.run_targeted("pardon", name, None);
    }

    /// Builds `<command> <name> [reason]` and submits it.
    ///
    /// Silently does nothing if the name cannot be expressed as a single argument
    #[allow(clippy::unused_self)]
    fn run_targeted(&self, command: &str, name: &QString, reason: Option<&QString>) {
        let Some(side) = crate::gui_side() else {
            return;
        };
        let Some(commands) = side.commands() else {
            return;
        };
        let Some(target) = quote_argument(&name.to_string()) else {
            return;
        };

        let mut line = format!("{command} {target}");
        if let Some(reason) = reason {
            let reason = sanitize_reason(&reason.to_string());
            if !reason.is_empty() {
                // `kick`/`ban` take the reason as a GreedyPhrase, so it needs no quoting.
                line.push(' ');
                line.push_str(&reason);
            }
        }

        commands.submit(line);
    }
}

/// Renders a player name as one command argument.
///
/// Bedrock gamertags may contain spaces, and Pumpkin's `StringReader::read_string` accepts a
/// quoted argument with backslash escapes, so those still target correctly. Returns `None` for
/// names that could not survive quoting.
fn quote_argument(name: &str) -> Option<String> {
    if name.is_empty() || name.chars().any(char::is_control) {
        return None;
    }

    // Mirrors `StringReader::is_allowed_in_unquoted_string`.
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
    {
        return Some(name.to_owned());
    }

    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    Some(format!("\"{escaped}\""))
}

/// Strips control characters from a free-text reason.
fn sanitize_reason(reason: &str) -> String {
    reason
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_owned()
}

fn player_to_variant(player: &crate::PlayerRow) -> QVariant {
    let mut map = QMap::<QMapPair_QString_QVariant>::default();

    map.insert(
        QString::from("name"),
        QVariant::from(&QString::from(&player.name)),
    );
    map.insert(
        QString::from("uuid"),
        QVariant::from(&QString::from(&player.uuid)),
    );
    map.insert(
        QString::from("edition"),
        QVariant::from(&QString::from(&player.edition)),
    );
    map.insert(QString::from("ping"), QVariant::from(&player.ping_ms));
    map.insert(
        QString::from("dimension"),
        QVariant::from(&QString::from(&player.dimension)),
    );
    map.insert(
        QString::from("gamemode"),
        QVariant::from(&QString::from(&player.gamemode)),
    );
    map.insert(
        QString::from("online"),
        QVariant::from(&(player.online_secs as f64)),
    );
    map.insert(QString::from("isOnline"), QVariant::from(&player.online));
    map.insert(QString::from("operator"), QVariant::from(&player.operator));
    map.insert(QString::from("banned"), QVariant::from(&player.banned));
    map.insert(
        QString::from("whitelisted"),
        QVariant::from(&player.whitelisted),
    );

    QVariant::from(&map)
}
