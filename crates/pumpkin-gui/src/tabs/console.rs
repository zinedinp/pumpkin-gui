//! Log output plus a command line, backed by the same dispatcher as the terminal console.
//!
//! The log is one `text_editor` over the whole scrollback rather than a widget per line: only a
//! single text object can carry a selection that spans lines, and cosmic-text's editor appends
//! without reshaping what is already there. Colours ride in through a [`Highlighter`] fed from
//! `LogLine::runs`, which is why nothing here parses text to decide how to paint it.

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use iced::advanced::text::highlighter::{self, Highlighter};
use iced::widget::{
    Row, column, container, pick_list, row, text, text_editor,
};
use iced::{Background, Color, Element, Font, Length, Theme};
use pumpkin_gui_api::{LogLevel, LogLine};

use crate::app::{App, Message as AppMessage};
use crate::theme::{Palette, metric};
use crate::widgets as w;

/// Which levels are shown. Mirrors the Qt combo box, which offered only these four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    #[default]
    All,
    Info,
    Warn,
    Error,
}

impl Level {
    pub const ALL: [Self; 4] = [Self::All, Self::Info, Self::Warn, Self::Error];

    const fn rank(self) -> u8 {
        match self {
            Self::All => 0,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::All => "all",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        })
    }
}

const fn level_rank(level: LogLevel) -> u8 {
    match level {
        LogLevel::Trace => 0,
        LogLevel::Debug => 1,
        LogLevel::Info => 2,
        LogLevel::Warn => 3,
        LogLevel::Error => 4,
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    LevelChanged(Level),
    SearchChanged(String),
    JumpToEnd,
    Copy,
    SaveLog,
    InputChanged(String),
    Submit,
    HistoryPrev,
    HistoryNext,
    Complete,
    /// Nothing changed, but a task finished.
    Done,
}

/// The colour ranges of one line, in byte offsets into that line's text.
type LineSpans = Vec<(Range<usize>, Color)>;

/// Handed to the highlighter by revision: comparing the table itself would cost more than the
/// highlighting it saves.
#[derive(Debug, Clone)]
pub struct HighlightSettings {
    revision: u64,
    lines: Arc<[LineSpans]>,
}

impl PartialEq for HighlightSettings {
    fn eq(&self, other: &Self) -> bool {
        self.revision == other.revision
    }
}

/// Colours lines from a table the server already gave us, rather than by parsing them again.
pub struct AnsiHighlighter {
    settings: HighlightSettings,
    current: usize,
}

impl Highlighter for AnsiHighlighter {
    type Settings = HighlightSettings;
    type Highlight = Color;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Color)>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            settings: settings.clone(),
            current: 0,
        }
    }

    // Deliberately does not rewind: lines appended at the end must not re-colour the rest.
    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
    }

    fn change_line(&mut self, line: usize) {
        self.current = self.current.min(line);
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let spans = self
            .settings
            .lines
            .get(self.current)
            .cloned()
            .unwrap_or_default();
        self.current += 1;
        spans.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current
    }
}

pub struct State {
    content: text_editor::Content,
    spans: Vec<LineSpans>,
    shared: Arc<[LineSpans]>,
    revision: u64,

    /// Whether the view sticks to the newest line.
    follow: bool,
    /// Lines the log has that the editor does not, because the user is reading back.
    pending: usize,
    /// The sequence number after the last line folded into the editor.
    next_seq: u64,

    level: Level,
    search: String,

    input: String,
    history: Vec<String>,
    /// Index into `history`; equal to its length means "editing a new line".
    history_index: usize,
    draft: String,
    hint: String,

    next_request: u32,
    pending_completion: Option<u32>,
    copied: bool,
    /// Whether the log, rather than the command line, is what the keyboard is aimed at. Up/Down
    /// move a cursor in one and step through history in the other.
    log_has_focus: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            content: text_editor::Content::new(),
            spans: Vec::new(),
            shared: Arc::from(Vec::new()),
            revision: 0,
            follow: true,
            pending: 0,
            next_seq: 0,
            level: Level::All,
            search: String::new(),
            input: String::new(),
            history: Vec::new(),
            history_index: 0,
            draft: String::new(),
            hint: String::new(),
            next_request: 0,
            pending_completion: None,
            copied: false,
            log_has_focus: false,
        }
    }
}

impl State {
    fn shows(&self, line: &LogLine) -> bool {
        if level_rank(line.level) < self.level.rank() {
            return false;
        }
        self.search.is_empty()
            || line
                .message
                .to_lowercase()
                .contains(&self.search.to_lowercase())
    }

    fn publish(&mut self) {
        self.revision += 1;
        self.shared = Arc::from(self.spans.clone());
    }

    /// Rebuilds the whole editor from the log. Used when the filter or the theme changes, which
    /// alters lines that are already there.
    fn rebuild(&mut self, log: &VecDeque<LogLine>, palette: Palette) {
        let visible: Vec<&LogLine> = log.iter().filter(|line| self.shows(line)).collect();

        let mut text = String::new();
        self.spans.clear();
        for (index, line) in visible.iter().enumerate() {
            if index > 0 {
                text.push('\n');
            }
            text.push_str(&line.message);
            self.spans.push(spans_of(line, palette));
        }

        self.content = text_editor::Content::with_text(&text);
        self.next_seq = log.back().map_or(0, |line| line.seq + 1);
        self.pending = 0;
        self.publish();
    }

    /// Folds every line newer than the cursor into the editor.
    fn append_new(&mut self, log: &VecDeque<LogLine>, palette: Palette) {
        let fresh: Vec<&LogLine> = log
            .iter()
            .filter(|line| line.seq >= self.next_seq && self.shows(line))
            .collect();
        self.next_seq = log.back().map_or(self.next_seq, |line| line.seq + 1);

        if fresh.is_empty() {
            return;
        }

        let empty = self.spans.is_empty();
        let mut text = String::new();
        for (index, line) in fresh.iter().enumerate() {
            // The first line of an empty editor must not start with a blank line.
            if index > 0 || !empty {
                text.push('\n');
            }
            text.push_str(&line.message);
            self.spans.push(spans_of(line, palette));
        }

        self.content
            .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
        self.content
            .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                Arc::new(text),
            )));
        self.publish();
    }
}

/// The colour of every run in a line, with runs that carry no ANSI colour of their own falling
/// back to the level's.
fn spans_of(line: &LogLine, palette: Palette) -> LineSpans {
    let fallback = match line.level {
        LogLevel::Error => palette.danger,
        LogLevel::Warn => palette.warn,
        LogLevel::Debug | LogLevel::Trace => palette.fg_muted,
        LogLevel::Info => palette.fg,
    };

    let mut offset = 0;
    line.runs
        .iter()
        .map(|run| {
            let start = offset;
            offset += run.text.len();
            let color = run
                .color
                .map_or(fallback, |(r, g, b)| Color::from_rgb8(r, g, b));
            (start..offset, color)
        })
        .collect()
}

/// New log lines arrived. While the user is reading back, hold them rather than dragging the view
/// to the bottom and wiping their selection.
pub fn on_new_lines(app: &mut App) -> iced::Task<AppMessage> {
    let palette = app.palette();
    let (state, log) = app.console_and_log();

    if state.follow {
        state.append_new(log, palette);
    } else {
        state.pending = log
            .iter()
            .filter(|line| line.seq >= state.next_seq && state.shows(line))
            .count();
    }
    iced::Task::none()
}

/// The theme changed, so every colour already baked into the table is wrong.
pub fn on_theme_changed(app: &mut App) {
    let palette = app.palette();
    let (state, log) = app.console_and_log();
    state.rebuild(log, palette);
}

pub fn on_completions(
    app: &mut App,
    id: u32,
    candidates: &[String],
) -> iced::Task<AppMessage> {
    let state = app.console_state_mut();
    if state.pending_completion != Some(id) {
        return iced::Task::none();
    }
    state.pending_completion = None;

    match candidates.len() {
        0 => {}
        1 => {
            // A single match completes the word in place.
            let candidate = candidates.first().cloned().unwrap_or_default();
            let head = state.input.rfind(' ').map_or(0, |at| at + 1);
            state.input.truncate(head);
            state.input.push_str(&candidate);
            state.hint.clear();
        }
        _ => state.hint = candidates.join("  "),
    }
    iced::Task::none()
}

pub fn update(app: &mut App, message: Message) -> iced::Task<AppMessage> {
    let palette = app.palette();

    match message {
        Message::Edit(action) => {
            let state = app.console_state_mut();
            // Anything the user does inside the log means they want it to hold still.
            let takes_over = match &action {
                text_editor::Action::Scroll { lines } => *lines < 0,
                text_editor::Action::Click(_)
                | text_editor::Action::Drag(_)
                | text_editor::Action::Select(_)
                | text_editor::Action::SelectWord
                | text_editor::Action::SelectLine
                | text_editor::Action::SelectAll => true,
                _ => false,
            };
            if takes_over {
                // TEMP PROBE
                #[allow(clippy::print_stderr)]
                {
                    eprintln!("PROBE takes_over via {action:?}");
                }
                state.follow = false;
                state.log_has_focus = true;
            }
            // Read-only: scrolling, clicking and selecting are allowed, edits are not.
            if !action.is_edit() {
                state.content.perform(action);
            }
        }
        Message::LevelChanged(level) => {
            let (state, log) = app.console_and_log();
            state.level = level;
            state.rebuild(log, palette);
        }
        Message::SearchChanged(search) => {
            let (state, log) = app.console_and_log();
            state.search = search;
            state.rebuild(log, palette);
        }
        Message::JumpToEnd => {
            let (state, log) = app.console_and_log();
            state.follow = true;
            state.append_new(log, palette);
        }
        Message::Copy => {
            let state = app.console_state_mut();
            state.copied = true;
            let text = state
                .content
                .selection()
                .unwrap_or_else(|| state.content.text());
            return iced::clipboard::write(format!("```\n{text}\n```"))
                .map(|()| AppMessage::Console(Message::Done));
        }
        Message::SaveLog => {
            let contents = app.console_state().content.text();
            return iced::Task::perform(save_log(contents), |()| {
                AppMessage::Console(Message::Done)
            });
        }
        Message::InputChanged(value) => {
            let state = app.console_state_mut();
            state.input = value;
            state.hint.clear();
            state.log_has_focus = false;
        }
        Message::Submit => {
            let state = app.console_state_mut();
            let line = state.input.trim().trim_start_matches('/').trim().to_owned();
            if line.is_empty() {
                return iced::Task::none();
            }
            // Same rule as a shell: repeating the previous command adds no duplicate entry.
            if state.history.last() != Some(&line) {
                state.history.push(line.clone());
            }
            state.history_index = state.history.len();
            state.draft.clear();
            state.input.clear();
            state.hint.clear();
            app.submit(line);
        }
        Message::HistoryPrev => recall(app.console_state_mut(), -1),
        Message::HistoryNext => recall(app.console_state_mut(), 1),
        Message::Complete => {
            let state = app.console_state_mut();
            let id = state.next_request;
            state.next_request = state.next_request.wrapping_add(1);
            state.pending_completion = Some(id);
            let line = state.input.clone();
            let cursor = line.len();
            if let Some(outbound) = app.outbound() {
                outbound.send(pumpkin_gui_api::GuiMessage::Complete { id, line, cursor });
            }
        }
        Message::Done => {}
    }

    iced::Task::none()
}

/// True while Up/Down/Tab should drive the command line rather than the log.
pub const fn command_line_has_focus(state: &State) -> bool {
    !state.log_has_focus
}

/// Steps through the command history; stepping off the end restores the half-typed line.
fn recall(state: &mut State, offset: isize) {
    if state.history.is_empty() {
        return;
    }
    if state.history_index == state.history.len() {
        state.draft = state.input.clone();
    }

    let next = (state.history_index as isize + offset).clamp(0, state.history.len() as isize);
    state.history_index = next as usize;
    state.input = state
        .history
        .get(state.history_index)
        .cloned()
        .unwrap_or_else(|| state.draft.clone());
}

async fn save_log(contents: String) {
    let name = "pumpkin.log";
    let Some(handle) = rfd::AsyncFileDialog::new()
        .set_title("Save log")
        .add_filter("Log files", &["log"])
        .set_file_name(name)
        .save_file()
        .await
    else {
        return;
    };
    let _ = std::fs::write(handle.path(), contents);
}

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let palette = app.palette();
    let state = app.console_state();

    let toolbar = toolbar(app, state, palette);

    let log = container(
        text_editor(&state.content)
            .size(12)
            .font(Font::MONOSPACE)
            .height(Length::Fill)
            .padding(8)
            .on_action(|action| AppMessage::Console(Message::Edit(action)))
            .highlight_with::<AnsiHighlighter>(
                HighlightSettings {
                    revision: state.revision,
                    lines: state.shared.clone(),
                },
                |color, _theme| highlighter::Format {
                    color: Some(*color),
                    font: None,
                },
            )
            .style(move |_: &Theme, _status| text_editor::Style {
                background: Background::Color(palette.surface),
                border: iced::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: metric::RADIUS.into(),
                },
                placeholder: palette.fg_muted,
                value: palette.fg,
                selection: Palette::with_alpha(palette.accent, 0.35),
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    column![toolbar, log, command_input(app, state, palette)]
        .spacing(8)
        .into()
}

fn toolbar<'a>(app: &'a App, state: &'a State, palette: Palette) -> Element<'a, AppMessage> {
    let mut bar = Row::new()
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .push(
            pick_list(Level::ALL, Some(state.level), |level| {
                AppMessage::Console(Message::LevelChanged(level))
            })
            .text_size(12)
            .width(130.0),
        )
        .push(
            w::field(palette, "Filter output…", &state.search)
                .on_input(|value| AppMessage::Console(Message::SearchChanged(value)))
                .width(260.0),
        )
        .push(w::filler());

    // Reading back pauses the log rather than letting it scroll out from under the cursor, so say
    // how much is waiting and offer the way back.
    if !state.follow {
        let label = if state.pending == 0 {
            "Jump to end".to_owned()
        } else {
            format!("{} new — jump to end", state.pending)
        };
        bar = bar.push(
            w::action(palette, label, palette.accent)
                .on_press(AppMessage::Console(Message::JumpToEnd)),
        );
    }

    let has_text = !state.content.is_empty();
    let mut copy = w::action(palette, if state.copied { "Copied" } else { "Copy" }, palette.fg);
    let mut save = w::action(palette, "Save log", palette.fg);
    if has_text {
        copy = copy.on_press(AppMessage::Console(Message::Copy));
        save = save.on_press(AppMessage::Console(Message::SaveLog));
    }

    let _ = app;
    bar.push(copy).push(save).into()
}

/// Command line with history and tab completion, matching what rustyline offers in the terminal.
fn command_input<'a>(app: &'a App, state: &'a State, palette: Palette) -> Element<'a, AppMessage> {
    let ready = app.is_ready();

    let mut field = w::field(
        palette,
        if ready {
            "Type a command…"
        } else {
            "Waiting for the server…"
        },
        &state.input,
    )
    .font(Font::MONOSPACE)
    .size(13)
    .width(Length::Fill);

    if ready {
        field = field
            .on_input(|value| AppMessage::Console(Message::InputChanged(value)))
            .on_submit(AppMessage::Console(Message::Submit));
    }

    let mut run = w::action(palette, "Run", palette.fg);
    if ready && !state.input.trim().is_empty() {
        run = run.on_press(AppMessage::Console(Message::Submit));
    }

    let mut stack = column![field].spacing(2);
    if !state.hint.is_empty() {
        stack = stack.push(
            text(state.hint.clone())
                .size(11)
                .font(Font::MONOSPACE)
                .color(palette.fg_muted),
        );
    }

    row![
        text("$")
            .size(13)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::MONOSPACE
            })
            .color(palette.accent),
        stack,
        run,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
