//! The window: what it knows, what can happen to it, and how it is drawn.
//!
//! State lives here rather than behind shared handles: the connection pushes everything in as a
//! [`Message`], so nothing has to be polled and no value is stored twice.

use std::collections::VecDeque;

use iced::futures::{SinkExt, Stream};
use iced::widget::{Row, button, column, container, row, text};
use iced::{Background, Element, Length, Padding, Subscription, Task, Theme};
use pumpkin_gui_api::{
    LogLine, PluginRow, ServerMessage, ServerMeta, Snapshot, ThemePreference, WorldRow,
};

use crate::client::{self, Outbound};
use crate::launcher::{self, Status};
use crate::tabs;
use crate::theme::{Palette, metric};
use crate::widgets as w;

/// How much scrollback the window keeps, independent of the server's own ring size.
const LOG_CAPACITY: usize = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Performance,
    Worlds,
    Players,
    Plugins,
}

impl Tab {
    pub const ALL: [Self; 5] = [
        Self::Overview,
        Self::Performance,
        Self::Worlds,
        Self::Players,
        Self::Plugins,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Performance => "Performance",
            Self::Worlds => "Worlds",
            Self::Players => "Players",
            Self::Plugins => "Plugins",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Backend(client::Event),
    TabSelected(Tab),
    ToggleTheme,
    StopServer,
    CloseRequested,
    /// Nothing to do; lets a view hand back a no-op instead of an `Option<Message>`.
    Ignore,

    // Setup screen
    BrowseBinary,
    AttachEndpointChanged(String),
    UseAttachEndpoint,

    Console(tabs::console::Message),
    Players(tabs::players::Message),
    Plugins(tabs::plugins::Message),
}

pub struct App {
    attach: Option<String>,

    status: Status,
    outbound: Option<Outbound>,
    stopping: bool,

    meta: ServerMeta,
    snapshot: Snapshot,
    plugin_rows: Vec<PluginRow>,
    hot_reload: bool,
    log: VecDeque<LogLine>,

    dark: bool,
    tab: Tab,
    setup_endpoint: String,

    console: tabs::console::State,
    players: tabs::players::State,
    plugins: tabs::plugins::State,
}

impl App {
    pub fn new(attach: Option<String>) -> (Self, Task<Message>) {
        (
            Self {
                attach,
                status: Status::Resolving,
                outbound: None,
                stopping: false,
                meta: ServerMeta::default(),
                snapshot: Snapshot::default(),
                plugin_rows: Vec::new(),
                hot_reload: false,
                log: VecDeque::new(),
                dark: true,
                tab: Tab::Overview,
                setup_endpoint: String::new(),
                console: tabs::console::State::default(),
                players: tabs::players::State::default(),
                plugins: tabs::plugins::State::default(),
            },
            Task::none(),
        )
    }

    pub const fn palette(&self) -> Palette {
        Palette::of(self.dark)
    }

    pub fn theme(&self) -> Theme {
        self.palette().iced()
    }

    /// True once the server is accepting commands.
    const fn ready(&self) -> bool {
        self.outbound.is_some() && !self.stopping
    }

    /// Runs a console command, exactly as if it had been typed in the terminal.
    pub fn submit(&self, line: String) {
        if let Some(outbound) = &self.outbound {
            outbound.submit(line);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Backend(event) => self.on_backend(event),
            Message::TabSelected(tab) => {
                self.tab = tab;
                Task::none()
            }
            Message::ToggleTheme => {
                self.dark = !self.dark;
                // Level colours are baked into the log's colour table, so it has to be rebuilt.
                tabs::console::on_theme_changed(self);
                Task::none()
            }
            Message::StopServer => {
                if let Some(outbound) = &self.outbound {
                    outbound.request_stop();
                }
                Task::none()
            }
            Message::CloseRequested => {
                // Only stop the server on close when this window spawned it: attaching to a
                // server must leave that server running.
                if launcher::is_managed()
                    && let Some(outbound) = &self.outbound
                {
                    outbound.request_stop();
                }
                iced::exit()
            }
            Message::Ignore => Task::none(),

            Message::BrowseBinary => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Choose the Pumpkin server executable")
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                },
                |picked| {
                    if let Some(path) = picked {
                        launcher::choose_binary_and_launch(path);
                    }
                    Message::Ignore
                },
            ),
            Message::AttachEndpointChanged(value) => {
                self.setup_endpoint = value;
                Task::none()
            }
            Message::UseAttachEndpoint => {
                let endpoint = self.setup_endpoint.trim().to_owned();
                if !endpoint.is_empty() {
                    launcher::use_attach_endpoint(endpoint);
                }
                Task::none()
            }

            Message::Console(message) => tabs::console::update(self, message),
            Message::Players(message) => tabs::players::update(self, message),
            Message::Plugins(message) => tabs::plugins::update(self, message),
        }
    }

    fn on_backend(&mut self, event: client::Event) -> Task<Message> {
        match event {
            client::Event::Status(status) => {
                self.status = status;
            }
            client::Event::Connected {
                meta,
                theme,
                outbound,
            } => {
                self.meta = *meta;
                self.outbound = Some(outbound);
                match theme {
                    ThemePreference::Dark => self.dark = true,
                    ThemePreference::Light => self.dark = false,
                    ThemePreference::System => {}
                }
            }
            client::Event::Server(message) => return self.on_server(*message),
            client::Event::Disconnected => {
                self.outbound = None;
                self.stopping = true;
                // The server is gone; there is nothing left for the window to show.
                return iced::exit();
            }
        }
        Task::none()
    }

    fn on_server(&mut self, message: ServerMessage) -> Task<Message> {
        match message {
            ServerMessage::Snapshot(snapshot) => self.snapshot = snapshot,
            ServerMessage::LogLines(lines) => {
                self.log.extend(lines);
                while self.log.len() > LOG_CAPACITY {
                    self.log.pop_front();
                }
                return tabs::console::on_new_lines(self);
            }
            ServerMessage::Completions { id, candidates } => {
                return tabs::console::on_completions(self, id, &candidates);
            }
            ServerMessage::Plugins { rows, hot_reload } => {
                self.plugin_rows = rows;
                self.hot_reload = hot_reload;
            }
            // `Config`/`ConfigWritten` are not surfaced yet (the editor is a separate step), and
            // `Hello`/`ShuttingDown` never reach here: the connection handles both itself.
            ServerMessage::Config { .. }
            | ServerMessage::ConfigWritten { .. }
            | ServerMessage::Hello { .. }
            | ServerMessage::ShuttingDown => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let typing = tabs::console::command_line_has_focus(&self.console) && self.tab == Tab::Overview;

        Subscription::batch([
            Subscription::run_with(Attach(self.attach.clone()), backend).map(Message::Backend),
            iced::window::close_requests().map(|_| Message::CloseRequested),
            // History and completion are keys the command line cannot see itself: a `text_input`
            // reports typing and submits, not arrows or tab.
            // `with` rather than a captured flag: a subscription closure has to be non-capturing.
            iced::keyboard::listen().with(typing).filter_map(|(typing, event)| {
                use iced::keyboard::{Event, Key, key::Named};

                let Event::KeyPressed { key, modifiers, .. } = event else {
                    return None;
                };
                if !typing || !modifiers.is_empty() {
                    return None;
                }
                let console = match key {
                    Key::Named(Named::ArrowUp) => tabs::console::Message::HistoryPrev,
                    Key::Named(Named::ArrowDown) => tabs::console::Message::HistoryNext,
                    Key::Named(Named::Tab) => tabs::console::Message::Complete,
                    _ => return None,
                };
                Some(Message::Console(console))
            }),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        let palette = self.palette();

        // The setup screen replaces everything until a server is reachable, rather than floating
        // over a dashboard that has nothing to show yet.
        let body: Element<'_, Message> = if self.outbound.is_none() {
            tabs::setup::view(self)
        } else {
            column![
                self.header(),
                w::row_rule(palette),
                self.tab_bar(),
                w::row_rule(palette),
                container(self.page())
                    .padding(metric::GAP)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .into()
        };

        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(palette.background)),
                ..container::Style::default()
            })
            .into()
    }

    fn header(&self) -> Element<'_, Message> {
        let palette = self.palette();
        let ready = self.snapshot.server_ready;

        let mut status = Row::new().spacing(16).align_y(iced::Alignment::Center).push(
            text(if ready {
                format!("Pumpkin {}", self.meta.pumpkin_version)
            } else {
                "Starting…".to_owned()
            })
            .size(15)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::default()
            })
            .color(palette.fg),
        );

        if ready {
            let online = self.snapshot.players.iter().filter(|p| p.online).count();
            status = status
                .push(w::badge(
                    palette,
                    "TPS",
                    format!("{:.1}", self.snapshot.tps),
                    palette.tps_color(self.snapshot.tps),
                ))
                .push(w::badge(
                    palette,
                    "PLAYERS",
                    online.to_string(),
                    palette.accent,
                ))
                .push(w::badge(
                    palette,
                    "UPTIME",
                    crate::format::duration(self.snapshot.uptime_secs),
                    palette.fg_muted,
                ));
        }

        let stop = {
            let stop = w::action(palette, "Stop Server", palette.danger);
            if self.ready() {
                stop.on_press(Message::StopServer)
            } else {
                stop
            }
        };

        let toggle = button(
            text(if self.dark { "☀" } else { "☾" })
                .size(16)
                .color(palette.fg),
        )
        .padding(6)
        .style(move |_: &Theme, _| button::Style {
            background: None,
            text_color: palette.fg,
            ..button::Style::default()
        })
        .on_press(Message::ToggleTheme);

        container(
            row![
                status,
                w::filler(),
                stop,
                w::filler(),
                toggle
            ]
            .align_y(iced::Alignment::Center)
            .spacing(12),
        )
        .height(52.0)
        .padding(Padding::new(0.0).left(16.0).right(12.0))
        .align_y(iced::Alignment::Center)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.surface)),
            ..container::Style::default()
        })
        .into()
    }

    fn tab_bar(&self) -> Element<'_, Message> {
        let palette = self.palette();
        let tabs = Tab::ALL.iter().fold(Row::new(), |bar, &tab| {
            bar.push(
                button(w::tab(palette, tab.label(), self.tab == tab))
                    .padding(0)
                    .style(move |_: &Theme, status| button::Style {
                        background: Some(Background::Color(
                            if status == button::Status::Hovered && self.tab != tab {
                                palette.surface_alt
                            } else if self.tab == tab {
                                palette.background
                            } else {
                                palette.surface
                            },
                        )),
                        text_color: palette.fg,
                        ..button::Style::default()
                    })
                    .on_press(Message::TabSelected(tab)),
            )
        });

        container(tabs.push(w::filler()))
            .width(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(palette.surface)),
                ..container::Style::default()
            })
            .into()
    }

    fn page(&self) -> Element<'_, Message> {
        match self.tab {
            Tab::Overview => tabs::overview::view(self),
            Tab::Performance => tabs::performance::view(self),
            Tab::Worlds => tabs::worlds::view(self),
            Tab::Players => tabs::players::view(self),
            Tab::Plugins => tabs::plugins::view(self),
        }
    }

    /// Splits the borrow so the console can rebuild itself from the log it renders.
    pub const fn console_and_log(&mut self) -> (&mut tabs::console::State, &VecDeque<LogLine>) {
        (&mut self.console, &self.log)
    }

    // Read-only access for the tab modules, which are views over this state rather than
    // components with state of their own.
    pub const fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub const fn meta(&self) -> &ServerMeta {
        &self.meta
    }
    pub fn worlds(&self) -> &[WorldRow] {
        &self.snapshot.worlds
    }
    pub fn plugin_rows(&self) -> &[PluginRow] {
        &self.plugin_rows
    }
    pub const fn hot_reload(&self) -> bool {
        self.hot_reload
    }
    pub const fn status(&self) -> &Status {
        &self.status
    }
    pub fn setup_endpoint(&self) -> &str {
        &self.setup_endpoint
    }
    pub const fn is_ready(&self) -> bool {
        self.outbound.is_some() && !self.stopping
    }
    pub const fn console_state(&self) -> &tabs::console::State {
        &self.console
    }
    pub const fn console_state_mut(&mut self) -> &mut tabs::console::State {
        &mut self.console
    }
    pub const fn players_state(&self) -> &tabs::players::State {
        &self.players
    }
    pub const fn players_state_mut(&mut self) -> &mut tabs::players::State {
        &mut self.players
    }
    pub const fn plugins_state(&self) -> &tabs::plugins::State {
        &self.plugins
    }
    pub const fn plugins_state_mut(&mut self) -> &mut tabs::plugins::State {
        &mut self.plugins
    }
    pub const fn outbound(&self) -> Option<&Outbound> {
        self.outbound.as_ref()
    }
}

/// The `--attach` endpoint, in a named type so it can identify the subscription.
#[derive(Hash, Clone)]
pub struct Attach(pub Option<String>);

/// Owns the launcher and, once it resolves, the connection; everything they learn arrives here.
fn backend(attach: &Attach) -> impl Stream<Item = client::Event> + use<> {
    let attach = attach.0.clone();

    iced::stream::channel(256, async move |mut output| {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        launcher::start(attach, tx);

        while let Some(event) = rx.recv().await {
            if output.send(event).await.is_err() {
                break;
            }
        }
    })
}

/// Starts the window and returns once it closes.
///
/// # Errors
///
/// Returns an error if the window cannot be created at all, usually because there is no display.
pub fn run(attach: Option<String>) -> iced::Result {
    iced::application(move || App::new(attach.clone()), App::update, App::view)
        .title("Pumpkin")
        .theme(App::theme)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1180.0, 780.0),
            min_size: Some(iced::Size::new(760.0, 520.0)),
            ..iced::window::Settings::default()
        })
        // The window decides what closing means: a managed server is stopped first.
        .exit_on_close_request(false)
        .run()
}
