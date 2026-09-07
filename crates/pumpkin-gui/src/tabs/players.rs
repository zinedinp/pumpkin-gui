//! The player table, with the moderation actions each view offers.

use std::collections::BTreeMap;

use iced::widget::{Row, column, container, pick_list, row, scrollable, text};
use iced::{Element, Length, Padding};
use pumpkin_gui_api::PlayerRow;

use crate::app::{App, Message as AppMessage};
use crate::command;
use crate::format;
use crate::theme::{Palette, metric};
use crate::widgets as w;

const NAME: f32 = 186.0;
const EDITION: f32 = 78.0;
const UUID: f32 = 276.0;
const PING: f32 = 60.0;
const DIMENSION: f32 = 120.0;
const MODE: f32 = 80.0;
const ONLINE: f32 = 70.0;

/// Which slice of the player list is shown. Each one offers different actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Online,
    Offline,
    Operator,
    Banned,
    Whitelisted,
}

impl View {
    pub const ALL: [Self; 5] = [
        Self::Online,
        Self::Offline,
        Self::Operator,
        Self::Banned,
        Self::Whitelisted,
    ];

    const fn empty_message(self) -> &'static str {
        match self {
            Self::Online => "No players online.",
            Self::Offline => "No offline players.",
            Self::Operator => "No operators.",
            Self::Banned => "No banned players.",
            Self::Whitelisted => "Whitelist is empty.",
        }
    }

    const fn contains(self, player: &PlayerRow) -> bool {
        match self {
            Self::Online => player.online,
            Self::Offline => !player.online,
            Self::Operator => player.operator,
            Self::Banned => player.banned,
            Self::Whitelisted => player.whitelisted,
        }
    }
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "Online",
            Self::Offline => "Offline",
            Self::Operator => "Operator",
            Self::Banned => "Banned",
            Self::Whitelisted => "Whitelisted",
        })
    }
}

/// A kick or a ban, which are confirmed rather than fired straight off a hovered row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    Kick,
    Ban,
}

impl Reason {
    const fn command(self) -> &'static str {
        match self {
            Self::Kick => "kick",
            Self::Ban => "ban",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewChanged(View),
    SearchChanged(String),
    WhitelistFieldChanged(String),
    AddWhitelist,
    Op(String),
    Deop(String),
    Pardon(String),
    Unwhitelist(String),
    /// Opens the confirmation dialog.
    AskReason(Reason, String),
    ReasonChanged(String),
    ConfirmReason,
    CancelReason,
}

#[derive(Default)]
pub struct State {
    pub view: View,
    pub search: String,
    pub whitelist_name: String,
    /// The open kick/ban dialog, if any.
    pub asking: Option<(Reason, String)>,
    pub reason: String,
}

impl State {
    fn matches(&self, player: &PlayerRow) -> bool {
        if !self.view.contains(player) {
            return false;
        }
        if self.search.is_empty() {
            return true;
        }
        let needle = self.search.to_lowercase();
        player.name.to_lowercase().contains(&needle) || player.uuid.to_lowercase().contains(&needle)
    }
}

pub fn update(app: &mut App, message: Message) -> iced::Task<AppMessage> {
    match message {
        Message::ViewChanged(view) => app.players_state_mut().view = view,
        Message::SearchChanged(value) => app.players_state_mut().search = value,
        Message::WhitelistFieldChanged(value) => app.players_state_mut().whitelist_name = value,
        Message::AddWhitelist => {
            let name = app.players_state().whitelist_name.trim().to_owned();
            if let Some(line) = command::targeted("whitelist add", &name, None) {
                app.submit(line);
                app.players_state_mut().whitelist_name.clear();
            }
        }
        Message::Op(name) => run(app, "op", &name),
        Message::Deop(name) => run(app, "deop", &name),
        Message::Pardon(name) => run(app, "pardon", &name),
        Message::Unwhitelist(name) => run(app, "whitelist remove", &name),
        Message::AskReason(reason, name) => {
            let state = app.players_state_mut();
            state.asking = Some((reason, name));
            state.reason.clear();
        }
        Message::ReasonChanged(value) => app.players_state_mut().reason = value,
        Message::ConfirmReason => {
            let state = app.players_state();
            if let Some((reason, name)) = state.asking.clone() {
                let text = state.reason.clone();
                if let Some(line) = command::targeted(reason.command(), &name, Some(&text)) {
                    app.submit(line);
                }
            }
            app.players_state_mut().asking = None;
        }
        Message::CancelReason => app.players_state_mut().asking = None,
    }
    iced::Task::none()
}

fn run(app: &App, verb: &str, name: &str) {
    if let Some(line) = command::targeted(verb, name, None) {
        app.submit(line);
    }
}

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let palette = app.palette();
    let state = app.players_state();

    let visible: Vec<&PlayerRow> = app
        .snapshot()
        .players
        .iter()
        .filter(|player| state.matches(player))
        .collect();

    let page = column![
        toolbar(app, state, palette, &visible),
        table(app, state, palette, &visible),
    ]
    .spacing(metric::GAP);

    // The dialog covers the page rather than floating over it, so a stray click cannot act on a
    // row underneath.
    match &state.asking {
        Some((reason, name)) => reason_dialog(palette, *reason, name, &state.reason),
        None => page.into(),
    }
}

fn toolbar<'a>(
    app: &'a App,
    state: &'a State,
    palette: Palette,
    visible: &[&'a PlayerRow],
) -> Element<'a, AppMessage> {
    // One chip per dimension people are actually in, so an empty world adds no noise.
    let mut per_dimension: BTreeMap<&str, usize> = BTreeMap::new();
    for player in visible {
        let dimension = format::dimension(&player.dimension);
        if !dimension.is_empty() {
            *per_dimension.entry(dimension).or_default() += 1;
        }
    }

    let mut bar = Row::new()
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .push(
            pick_list(View::ALL, Some(state.view), |view| {
                AppMessage::Players(Message::ViewChanged(view))
            })
            .text_size(12)
            .width(150.0),
        )
        .push(
            w::field(palette, "Search players…", &state.search)
                .on_input(|value| AppMessage::Players(Message::SearchChanged(value)))
                .width(220.0),
        );

    for (dimension, count) in per_dimension {
        bar = bar.push(w::count_chip(palette, count, dimension));
    }

    let ready = app.is_ready();
    let mut field = w::field(palette, "Player name…", &state.whitelist_name).width(220.0);
    let mut add = w::action(palette, "Whitelist", palette.accent);
    if ready {
        field = field
            .on_input(|value| AppMessage::Players(Message::WhitelistFieldChanged(value)))
            .on_submit(AppMessage::Players(Message::AddWhitelist));
        if !state.whitelist_name.trim().is_empty() {
            add = add.on_press(AppMessage::Players(Message::AddWhitelist));
        }
    }

    bar.push(w::filler()).push(field).push(add).into()
}

fn table<'a>(
    app: &'a App,
    state: &'a State,
    palette: Palette,
    visible: &[&'a PlayerRow],
) -> Element<'a, AppMessage> {
    let header = Row::new()
        .push(w::header_cell(palette, "NAME", Length::Fixed(NAME)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "EDITION", Length::Fixed(EDITION)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "UUID", Length::Fixed(UUID)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "PING", Length::Fixed(PING)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "DIMENSION", Length::Fixed(DIMENSION)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "MODE", Length::Fixed(MODE)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "ONLINE", Length::Fixed(ONLINE)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "ACTIONS", Length::Fill))
        .height(24.0);

    let body: Element<'_, AppMessage> = if visible.is_empty() {
        let message = if state.search.is_empty() {
            state.view.empty_message().to_owned()
        } else {
            format!("No player matches “{}”.", state.search)
        };
        container(text(message).size(12).color(palette.fg_muted))
            .width(Length::Fill)
            .padding(12)
            .align_x(iced::Alignment::Center)
            .into()
    } else {
        let mut rows = column![];
        for (index, player) in visible.iter().enumerate() {
            rows = rows.push(row_of(app, palette, state.view, index, player));
        }
        scrollable(rows)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    w::plain_card(
        palette,
        column![
            container(header).padding(Padding::new(0.0).bottom(6.0)),
            w::row_rule(palette),
            body,
        ],
    )
}

fn row_of<'a>(
    app: &'a App,
    palette: Palette,
    view: View,
    index: usize,
    player: &'a PlayerRow,
) -> Element<'a, AppMessage> {
    let edition_accent = if player.edition == "bedrock" {
        palette.good
    } else {
        palette.accent
    };
    let edition: Element<'_, AppMessage> = match player.edition.as_str() {
        "bedrock" => w::pill(palette, "Bedrock", edition_accent),
        "java" => w::pill(palette, "Java", edition_accent),
        _ => text("–").size(12).color(palette.fg_muted).into(),
    };

    let ping = if player.online {
        text(format!("{} ms", player.ping_ms))
            .size(metric::TABLE_CELL_SIZE)
            .font(iced::Font::MONOSPACE)
            .color(palette.load_color(player.ping_ms as f32 / 300.0))
    } else {
        text("–")
            .size(metric::TABLE_CELL_SIZE)
            .font(iced::Font::MONOSPACE)
            .color(palette.fg_muted)
    };

    let cells = Row::new()
        .push(w::cell(
            palette,
            player.name.as_str(),
            Length::Fixed(NAME),
            false,
            false,
        ))
        .push(w::column_rule(palette))
        .push(
            container(edition)
                .width(Length::Fixed(EDITION))
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD))
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        )
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            player.uuid.as_str(),
            Length::Fixed(UUID),
            true,
            true,
        ))
        .push(w::column_rule(palette))
        .push(
            container(ping)
                .width(Length::Fixed(PING))
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD))
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        )
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::or_dash(format::dimension(&player.dimension)),
            Length::Fixed(DIMENSION),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::or_dash(&player.gamemode),
            Length::Fixed(MODE),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            if player.online {
                format::duration(player.online_secs)
            } else {
                "–".to_owned()
            },
            Length::Fixed(ONLINE),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(
            container(actions(app, palette, view, player))
                .width(Length::Fill)
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD).right(metric::TABLE_EDGE))
                .align_x(iced::Alignment::End)
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        );

    w::row_background(palette, index, cells)
        .height(48.0)
        .width(Length::Fill)
        .into()
}

fn actions<'a>(
    app: &'a App,
    palette: Palette,
    view: View,
    player: &'a PlayerRow,
) -> Element<'a, AppMessage> {
    let ready = app.is_ready();
    let name = player.name.clone();

    // What each view offers, in the order the Qt row showed them.
    let buttons: Vec<(&'static str, iced::Color, Message)> = match view {
        View::Online => vec![
            ("Op", palette.accent, Message::Op(name.clone())),
            (
                "Kick",
                palette.warn,
                Message::AskReason(Reason::Kick, name.clone()),
            ),
            ("Ban", palette.danger, Message::AskReason(Reason::Ban, name)),
        ],
        View::Offline => vec![
            ("Op", palette.accent, Message::Op(name.clone())),
            ("Ban", palette.danger, Message::AskReason(Reason::Ban, name)),
        ],
        View::Operator => vec![("Revoke op", palette.danger, Message::Deop(name))],
        View::Banned => vec![("Pardon", palette.fg, Message::Pardon(name))],
        View::Whitelisted => vec![("Remove", palette.danger, Message::Unwhitelist(name))],
    };

    buttons
        .into_iter()
        .fold(Row::new().spacing(6), |row, (label, accent, message)| {
            let mut button = w::action(palette, label, accent);
            if ready {
                button = button.on_press(AppMessage::Players(message));
            }
            row.push(button)
        })
        .into()
}

/// Confirms a kick or ban and collects an optional reason.
fn reason_dialog<'a>(
    palette: Palette,
    reason: Reason,
    name: &'a str,
    text_value: &'a str,
) -> Element<'a, AppMessage> {
    let (title, explanation, confirm) = match reason {
        Reason::Kick => (
            format!("Kick {name}"),
            "The player is disconnected and may rejoin.",
            "Kick",
        ),
        Reason::Ban => (
            format!("Ban {name}"),
            "The player is disconnected and added to the ban list.",
            "Ban",
        ),
    };

    let body = column![
        text(explanation).size(12).color(palette.fg_muted),
        w::field(palette, "Reason (optional)", text_value)
            .on_input(|value| AppMessage::Players(Message::ReasonChanged(value)))
            .on_submit(AppMessage::Players(Message::ConfirmReason))
            .width(Length::Fill),
        row![
            w::filler(),
            w::action(palette, "Cancel", palette.fg)
                .on_press(AppMessage::Players(Message::CancelReason)),
            w::action(palette, confirm, palette.danger)
                .on_press(AppMessage::Players(Message::ConfirmReason)),
        ]
        .spacing(8),
    ]
    .spacing(metric::GAP);

    iced::widget::center(container(w::card(palette, title, body)).max_width(380.0)).into()
}
