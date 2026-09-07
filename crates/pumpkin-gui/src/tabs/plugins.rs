//! The plugin table: what is installed, what is running, and why anything is not.

use iced::widget::{Row, column, container, row, scrollable, text};
use iced::{Element, Length, Padding};
use pumpkin_gui_api::{PluginKind, PluginRow, PluginState};

use crate::app::{App, Message as AppMessage};
use crate::command;
use crate::format;
use crate::theme::{Palette, metric};
use crate::widgets as w;

const NAME: f32 = 214.0;
const VERSION: f32 = 90.0;
const KIND: f32 = 70.0;
const STATE: f32 = 100.0;

/// Why unload and reload can be unavailable, said on hover since a disabled button cannot.
const NOT_READY: &str = "The server is not accepting commands yet.";
const CANNOT_UNLOAD: &str = "This plugin's loader cannot unload at runtime — native plugins on \
                             Windows keep their library locked. Restart the server to change it.";

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    /// Expands or collapses one row, by name.
    Toggle(String),
    Load(String),
    Reload(String),
    AskUnload(String),
    ConfirmUnload,
    CancelUnload,
    SetHotReload(bool),
}

#[derive(Default)]
pub struct State {
    pub search: String,
    /// The one expanded row, by name. Names are unique per plugin manager.
    pub expanded: Option<String>,
    /// The plugin an unload has been asked about but not yet confirmed.
    pub unloading: Option<String>,
}

impl State {
    fn matches(&self, plugin: &PluginRow) -> bool {
        if self.search.is_empty() {
            return true;
        }
        let needle = self.search.to_lowercase();
        plugin.name.to_lowercase().contains(&needle)
            || plugin.description.to_lowercase().contains(&needle)
    }
}

pub fn update(app: &mut App, message: Message) -> iced::Task<AppMessage> {
    match message {
        Message::SearchChanged(value) => app.plugins_state_mut().search = value,
        Message::Toggle(name) => {
            let state = app.plugins_state_mut();
            state.expanded = if state.expanded.as_ref() == Some(&name) {
                None
            } else {
                Some(name)
            };
        }
        Message::Load(path) => {
            if let Some(path) = command::quote(&path) {
                app.submit(format!("plugin load {path}"));
            }
        }
        Message::Reload(name) => {
            if let Some(name) = command::quote(&name) {
                app.submit(format!("plugin reload {name}"));
            }
        }
        Message::AskUnload(name) => app.plugins_state_mut().unloading = Some(name),
        Message::ConfirmUnload => {
            if let Some(name) = app.plugins_state().unloading.clone()
                && let Some(name) = command::quote(&name)
            {
                app.submit(format!("plugin unload {name}"));
            }
            app.plugins_state_mut().unloading = None;
        }
        Message::CancelUnload => app.plugins_state_mut().unloading = None,
        Message::SetHotReload(enabled) => {
            app.submit(format!(
                "plugin hotreload {}",
                if enabled { "enable" } else { "disable" }
            ));
        }
    }
    iced::Task::none()
}

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let palette = app.palette();
    let state = app.plugins_state();

    let visible: Vec<&PluginRow> = app
        .plugin_rows()
        .iter()
        .filter(|plugin| state.matches(plugin))
        .collect();

    let page = column![
        toolbar(app, state, palette),
        table(app, state, palette, &visible),
    ]
    .spacing(metric::GAP);

    state
        .unloading
        .as_ref()
        .map_or_else(|| page.into(), |name| confirm_unload(palette, name))
}

fn toolbar<'a>(app: &'a App, state: &'a State, palette: Palette) -> Element<'a, AppMessage> {
    let running = app
        .plugin_rows()
        .iter()
        .filter(|plugin| plugin.state == PluginState::Loaded)
        .count();
    let stopped = app.plugin_rows().len() - running;

    let mut bar = Row::new()
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .push(
            w::field(palette, "Search plugins…", &state.search)
                .on_input(|value| AppMessage::Plugins(Message::SearchChanged(value)))
                .width(220.0),
        )
        .push(w::count_chip(palette, running, "running"));

    if stopped > 0 {
        bar = bar.push(w::count_chip(palette, stopped, "not running"));
    }

    // Hot reload is global, not per plugin, so it lives here rather than in a row.
    let mut toggle = w::check(palette, "Hot reload", app.hot_reload());
    if app.is_ready() {
        toggle = toggle.on_toggle(|enabled| AppMessage::Plugins(Message::SetHotReload(enabled)));
    }

    bar.push(w::filler())
        .push(w::hint(
            palette,
            toggle,
            "Reloads plugins when their file changes on disk. Costs performance; meant for \
             plugin development.",
        ))
        .into()
}

fn table<'a>(
    app: &'a App,
    state: &'a State,
    palette: Palette,
    visible: &[&'a PluginRow],
) -> Element<'a, AppMessage> {
    let header = Row::new()
        .push(w::header_cell(palette, "NAME", Length::Fixed(NAME)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "VERSION", Length::Fixed(VERSION)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "KIND", Length::Fixed(KIND)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "STATE", Length::Fixed(STATE)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "AUTHORS", Length::Fill))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "ACTIONS", Length::Fixed(200.0)))
        .height(24.0);

    let body: Element<'_, AppMessage> = if visible.is_empty() {
        let message = if state.search.is_empty() {
            "No plugins installed.".to_owned()
        } else {
            format!("No plugin matches “{}”.", state.search)
        };
        container(text(message).size(12).color(palette.fg_muted))
            .width(Length::Fill)
            .padding(12)
            .align_x(iced::Alignment::Center)
            .into()
    } else {
        let mut rows = column![];
        for (index, plugin) in visible.iter().enumerate() {
            rows = rows.push(row_of(app, palette, state, index, plugin));
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

/// The label and colour of a plugin's state pill.
///
/// Anything not running is red: green there would claim the plugin is working.
const fn state_pill(palette: Palette, plugin: &PluginRow) -> (&'static str, iced::Color) {
    match plugin.state {
        PluginState::Loading => ("Loading", palette.warn),
        PluginState::Loaded => ("Active", palette.good),
        PluginState::Failed(_) => ("Failed", palette.danger),
        PluginState::Unloaded => ("Unloaded", palette.danger),
    }
}

fn row_of<'a>(
    app: &'a App,
    palette: Palette,
    state: &'a State,
    index: usize,
    plugin: &'a PluginRow,
) -> Element<'a, AppMessage> {
    let expanded = state.expanded.as_deref() == Some(plugin.name.as_str());
    let (state_label, state_accent) = state_pill(palette, plugin);
    let (kind_label, kind_accent) = match plugin.kind {
        PluginKind::Wasm => ("WASM", palette.good),
        PluginKind::Native => ("Native", palette.accent),
    };

    let header = Row::new()
        .push(
            container(
                text(if expanded { "▼" } else { "▶" })
                    .size(8)
                    .color(palette.fg_muted),
            )
            .width(Length::Fixed(14.0 + metric::TABLE_EDGE))
            .padding(Padding::new(0.0).left(metric::TABLE_EDGE))
            .align_y(iced::Alignment::Center)
            .height(Length::Fill),
        )
        .push(w::cell(
            palette,
            plugin.name.as_str(),
            Length::Fixed(NAME - 14.0 - metric::TABLE_EDGE),
            false,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::or_dash(&plugin.version),
            Length::Fixed(VERSION),
            true,
            true,
        ))
        .push(w::column_rule(palette))
        .push(
            container(w::pill(palette, kind_label, kind_accent))
                .width(Length::Fixed(KIND))
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD))
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        )
        .push(w::column_rule(palette))
        .push(
            container(w::pill(palette, state_label, state_accent))
                .width(Length::Fixed(STATE))
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD))
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        )
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::list(&plugin.authors),
            Length::Fill,
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(
            // Fixed width with the buttons at the right, so the column rule lands at the same x
            // whichever buttons a row shows.
            container(actions(app, palette, plugin))
                .width(Length::Fixed(200.0))
                .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD).right(metric::TABLE_EDGE))
                .align_x(iced::Alignment::End)
                .align_y(iced::Alignment::Center)
                .height(Length::Fill),
        );

    let mut content = column![
        // Only the header toggles: a click in the detail pane must not collapse it away.
        iced::widget::button(container(header).height(40.0))
            .padding(0)
            .style(move |_: &iced::Theme, _| iced::widget::button::Style {
                background: None,
                text_color: palette.fg,
                ..iced::widget::button::Style::default()
            })
            .on_press(AppMessage::Plugins(Message::Toggle(plugin.name.clone()))),
    ];

    if expanded {
        content = content.push(detail(palette, plugin));
    }

    w::row_background(palette, index, content)
        .width(Length::Fill)
        .into()
}

fn detail(palette: Palette, plugin: &PluginRow) -> Element<'_, AppMessage> {
    let mut pane = column![].spacing(6);

    // The error when there is one, otherwise say plainly that it is not running: an unloaded
    // plugin has no error to show, and a blank detail pane looks like a bug.
    match &plugin.state {
        PluginState::Failed(error) => {
            pane = pane.push(text(error.as_str()).size(12).color(palette.danger));
        }
        PluginState::Unloaded => {
            pane = pane.push(
                text("Not running. Load it to start it again.")
                    .size(12)
                    .color(palette.danger),
            );
        }
        PluginState::Loading | PluginState::Loaded => {}
    }

    pane = pane.push(
        text(if plugin.description.is_empty() {
            "No description."
        } else {
            plugin.description.as_str()
        })
        .size(12)
        .color(palette.fg),
    );

    if !plugin.dependencies.is_empty() {
        pane = pane.push(
            text(format!("Depends on: {}", plugin.dependencies.join(", ")))
                .size(11)
                .color(palette.fg_muted),
        );
    }

    if !plugin.permissions.is_empty() {
        let mut permissions = Row::new()
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .push(text("Permissions").size(11).color(palette.fg_muted));
        for permission in &plugin.permissions {
            permissions = permissions.push(w::hint(
                palette,
                w::pill(palette, permission.name.as_str(), palette.warn),
                if permission.description.is_empty() {
                    "Not a permission this server knows about."
                } else {
                    permission.description.as_str()
                },
            ));
        }
        pane = pane.push(permissions);
    }

    pane = pane.push(
        text(format::or_dash(&plugin.path))
            .size(12)
            .font(iced::Font::MONOSPACE)
            .color(palette.fg_muted),
    );

    container(pane)
        .padding(
            Padding::new(0.0)
                .left(metric::TABLE_EDGE + 14.0)
                .right(metric::TABLE_EDGE)
                .top(metric::GAP / 2.0)
                .bottom(metric::GAP),
        )
        .width(Length::Fill)
        .into()
}

fn actions<'a>(app: &'a App, palette: Palette, plugin: &'a PluginRow) -> Element<'a, AppMessage> {
    // Mid-load rows offer nothing: the server is already acting on them.
    if plugin.state == PluginState::Loading {
        return w::filler().into();
    }

    let ready = app.is_ready();
    let stopped = matches!(
        plugin.state,
        PluginState::Failed(_) | PluginState::Unloaded
    );

    if stopped {
        let mut load = w::action(palette, "Load", palette.good);
        if ready && !plugin.path.is_empty() {
            load = load.on_press(AppMessage::Plugins(Message::Load(plugin.path.clone())));
        }
        return load.into();
    }

    let blocked = if ready { CANNOT_UNLOAD } else { NOT_READY };
    let enabled = ready && plugin.can_unload;

    let mut reload = w::action(palette, "Reload", palette.fg);
    let mut unload = w::action(palette, "Unload", palette.danger);
    if enabled {
        reload = reload.on_press(AppMessage::Plugins(Message::Reload(plugin.name.clone())));
        unload = unload.on_press(AppMessage::Plugins(Message::AskUnload(plugin.name.clone())));
    }

    let buttons = row![reload, unload].spacing(6);

    if enabled {
        buttons.into()
    } else {
        w::hint(palette, buttons, blocked)
    }
}

fn confirm_unload(palette: Palette, name: &str) -> Element<'_, AppMessage> {
    let body = column![
        text(format!(
            "Unload “{name}”? Anything it registered stops working immediately."
        ))
        .size(12)
        .color(palette.fg_muted),
        row![
            w::filler(),
            w::action(palette, "Cancel", palette.fg)
                .on_press(AppMessage::Plugins(Message::CancelUnload)),
            w::action(palette, "Unload", palette.danger)
                .on_press(AppMessage::Plugins(Message::ConfirmUnload)),
        ]
        .spacing(8),
    ]
    .spacing(metric::GAP);

    iced::widget::center(container(w::card(palette, "Unload plugin", body)).max_width(400.0)).into()
}
