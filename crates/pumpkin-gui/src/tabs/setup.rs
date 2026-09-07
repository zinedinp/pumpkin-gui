//! The first-run screen: shown whenever [`crate::launcher`] has not resolved a server to talk to.

use iced::widget::{center, column, container, row, text};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::launcher::{HEADLINE_SETUP, Status};
use crate::theme::metric;
use crate::widgets as w;

const NO_SERVER: &str = "No server was found next to pumpkin-gui. Choose the server executable, \
                         or connect to one that's already running.";

/// Splits the current status into what the screen renders: a headline, a message, and whether
/// that message is an error.
fn parts(status: &Status) -> (&str, String, bool) {
    match status {
        Status::Resolving => (HEADLINE_SETUP, "Looking for a server…".to_owned(), false),
        Status::NeedsSetup | Status::Connected => (HEADLINE_SETUP, String::new(), false),
        Status::Validating(message) | Status::Launching(message) | Status::Connecting(message) => {
            (HEADLINE_SETUP, message.clone(), false)
        }
        Status::Failed { headline, detail } => (headline, detail.clone(), true),
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();
    let (headline, message, failed) = parts(app.status());

    let lead = if failed {
        text(message.clone()).size(13).color(palette.danger)
    } else {
        text(NO_SERVER).size(12).color(palette.fg_muted)
    };

    let mut body = column![
        lead,
        w::action(palette, "Choose server executable…", palette.accent)
            .width(Length::Fill)
            .on_press(Message::BrowseBinary),
        w::row_rule(palette),
        text("Or attach to a running server")
            .size(12)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::default()
            })
            .color(palette.fg),
        row![
            w::field(palette, "Socket path or pipe name", app.setup_endpoint())
                .on_input(Message::AttachEndpointChanged)
                .on_submit(Message::UseAttachEndpoint)
                .width(Length::Fill),
            w::action(palette, "Attach", palette.fg).on_press(Message::UseAttachEndpoint),
        ]
        .spacing(8),
    ]
    .spacing(metric::GAP);

    // While something is still being tried, say what: the buttons above stay usable as a way out.
    if !failed && !message.is_empty() {
        body = body.push(text(message).size(12).color(palette.fg_muted));
    }

    center(container(w::card(palette, headline, body)).max_width(460.0)).into()
}
