//! The shared building blocks every page is assembled from.
//!
//! One function per former QML component, so spacing, borders and colours stay consistent without
//! each page repeating them.

use iced::widget::{
    Column, Row, Space, button, checkbox, column, container, row, space, text, text_input, tooltip,
};
use iced::{Background, Border, Color, Element, Font, Length, Padding, Shadow, Theme};

use crate::theme::{Palette, metric};

/// Empty space that pushes what follows it to the far side of its row.
pub fn filler() -> Space {
    space().width(Length::Fill)
}

/// Border with no shadow, which is every border in this UI.
fn edge(color: Color, radius: f32) -> Border {
    Border {
        color,
        width: 1.0,
        radius: radius.into(),
    }
}

/// A titled panel. Everything on the dashboard sits in one of these.
pub fn card<'a, Message: 'a>(
    palette: Palette,
    title: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    panel(
        palette,
        Column::new()
            .spacing(8)
            .push(section_title(palette, title))
            .push(content.into()),
    )
}

/// The same panel without a heading, for pages that are one big table.
pub fn plain_card<'a, Message: 'a>(
    palette: Palette,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    panel(palette, content.into())
}

fn panel<'a, Message: 'a>(
    palette: Palette,
    inner: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(inner.into())
        .padding(metric::GAP)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.surface)),
            border: edge(palette.border, metric::RADIUS),
            ..container::Style::default()
        })
        .into()
}

/// The accented heading a card and a stat tile share.
pub fn section_title<'a, Message: 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    text(label)
        .size(metric::TABLE_HEADER_SIZE + 2.0)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::default()
        })
        .color(palette.accent)
        .into()
}

/// A small tinted label: plugin kind, plugin state, a permission, an edition.
pub fn pill<'a, Message: 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
    accent: Color,
) -> Element<'a, Message> {
    container(
        text(label)
            .size(10)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            })
            .color(accent),
    )
    .padding(Padding::new(2.0).left(6.0).right(6.0))
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(palette.tint(accent))),
        border: edge(palette.tint_border(accent), 4.0),
        ..container::Style::default()
    })
    .into()
}

/// A compact label/value pair for the header bar.
pub fn badge<'a, Message: 'a>(
    palette: Palette,
    label: &'a str,
    value: String,
    accent: Color,
) -> Element<'a, Message> {
    column![
        text(value)
            .size(15)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::MONOSPACE
            })
            .color(accent),
        text(label).size(9).color(palette.fg_muted),
    ]
    .spacing(0)
    .into()
}

/// Compact "N label" chip used above the tables.
pub fn count_chip<'a, Message: 'a>(
    palette: Palette,
    count: usize,
    label: &'a str,
) -> Element<'a, Message> {
    container(
        row![
            text(count.to_string())
                .size(13)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::MONOSPACE
                })
                .color(palette.fg),
            text(label).size(12).color(palette.fg),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .height(metric::CONTROL_HEIGHT)
    .padding(Padding::new(0.0).left(8.0).right(8.0))
    .align_y(iced::Alignment::Center)
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(palette.surface_alt)),
        border: edge(palette.border, 6.0),
        ..container::Style::default()
    })
    .into()
}

/// A themed button. The caller adds `.on_press(..)`; without one it renders as disabled.
pub fn action<'a, Message: Clone + 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
    accent: Color,
) -> button::Button<'a, Message> {
    button(
        text(label)
            .size(12)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center),
    )
    .height(metric::CONTROL_HEIGHT)
    .padding(Padding::new(0.0).left(14.0).right(14.0))
    .style(move |_: &Theme, status| {
        // Disabled and idle share the resting surface; only hover and press lift it.
        let background = match status {
            button::Status::Pressed => palette.border,
            button::Status::Hovered => palette.surface_alt,
            button::Status::Active | button::Status::Disabled => palette.surface,
        };
        button::Style {
            background: Some(Background::Color(background)),
            text_color: if status == button::Status::Disabled {
                palette.fg_muted
            } else {
                accent
            },
            border: edge(palette.border, 4.0),
            shadow: Shadow::default(),
            snap: true,
        }
    })
}

/// A themed single-line input.
pub fn field<'a, Message: Clone + 'a>(
    palette: Palette,
    placeholder: &'a str,
    value: &'a str,
) -> text_input::TextInput<'a, Message> {
    text_input(placeholder, value)
        .size(12)
        .padding(Padding::new(0.0).left(10.0).right(10.0))
        .style(move |_: &Theme, status| text_input::Style {
            background: Background::Color(palette.background),
            border: edge(
                if matches!(status, text_input::Status::Focused { .. }) {
                    palette.accent
                } else {
                    palette.border
                },
                4.0,
            ),
            icon: palette.fg_muted,
            placeholder: palette.fg_muted,
            value: palette.fg,
            selection: palette.accent,
        })
}

/// A themed checkbox. The caller adds `.on_toggle(..)`.
pub fn check<'a, Message: Clone + 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
    checked: bool,
) -> checkbox::Checkbox<'a, Message> {
    checkbox(checked)
        .label(label)
        .size(16)
        .text_size(12)
        .style(move |_: &Theme, _status| checkbox::Style {
            background: Background::Color(if checked {
                palette.accent
            } else {
                palette.background
            }),
            icon_color: palette.surface,
            border: edge(
                if checked { palette.accent } else { palette.border },
                3.0,
            ),
            text_color: Some(palette.fg),
        })
}

/// One tab in the top bar. The active one is marked with an accent rule underneath rather than a
/// filled block, so the bar stays quiet next to the content below it.
pub fn tab<'a, Message: Clone + 'a>(
    palette: Palette,
    label: &'a str,
    selected: bool,
) -> Element<'a, Message> {
    let underline = container(filler())
        .height(if selected { 2.0 } else { 0.0 })
        .width(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.accent)),
            ..container::Style::default()
        });

    column![
        container(
            text(label)
                .size(13)
                .font(if selected {
                    Font {
                        weight: iced::font::Weight::Bold,
                        ..Font::default()
                    }
                } else {
                    Font::default()
                })
                .color(if selected { palette.fg } else { palette.fg_muted }),
        )
        .height(32.0)
        .padding(Padding::new(0.0).left(16.0).right(16.0))
        .align_y(iced::Alignment::Center),
        underline,
    ]
    .into()
}

/// A header cell of a table.
pub fn header_cell<'a, Message: 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
    width: Length,
) -> Element<'a, Message> {
    container(
        text(label)
            .size(metric::TABLE_HEADER_SIZE)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            })
            .color(palette.accent),
    )
    .width(width)
    .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD).right(metric::TABLE_CELL_PAD))
    .into()
}

/// A cell of a table row.
pub fn cell<'a, Message: 'a>(
    palette: Palette,
    value: impl text::IntoFragment<'a>,
    width: Length,
    muted: bool,
    mono: bool,
) -> Element<'a, Message> {
    container(
        text(value)
            .size(metric::TABLE_CELL_SIZE)
            .font(if mono { Font::MONOSPACE } else { Font::default() })
            .color(if muted { palette.fg_muted } else { palette.fg })
            .wrapping(text::Wrapping::None),
    )
    .width(width)
    .clip(true)
    .padding(Padding::new(0.0).left(metric::TABLE_CELL_PAD).right(metric::TABLE_CELL_PAD))
    .align_y(iced::Alignment::Center)
    .height(Length::Fill)
    .into()
}

/// The hairline between two table columns.
pub fn column_rule<'a, Message: 'a>(palette: Palette) -> Element<'a, Message> {
    container(filler())
        .width(1.0)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.rule)),
            ..container::Style::default()
        })
        .into()
}

/// The hairline under a table header or between chrome segments.
pub fn row_rule<'a, Message: 'a>(palette: Palette) -> Element<'a, Message> {
    container(filler())
        .width(Length::Fill)
        .height(1.0)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.rule)),
            ..container::Style::default()
        })
        .into()
}

/// Stripes a table row and marks the hovered one.
pub fn row_background<'a, Message: 'a>(
    palette: Palette,
    index: usize,
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    let base = if index.is_multiple_of(2) {
        palette.row_even
    } else {
        palette.row_odd
    };
    container(content).style(move |_: &Theme| container::Style {
        background: Some(Background::Color(base)),
        ..container::Style::default()
    })
}

/// Explains why a control is unavailable, which a disabled control cannot say itself.
pub fn hint<'a, Message: 'a>(
    palette: Palette,
    content: impl Into<Element<'a, Message>>,
    message: impl text::IntoFragment<'a>,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(message).size(11).color(palette.fg))
            .padding(8)
            .max_width(320)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(palette.surface_alt)),
                border: edge(palette.border, 4.0),
                ..container::Style::default()
            }),
        tooltip::Position::Top,
    )
    .delay(std::time::Duration::from_millis(400))
    .into()
}

/// One headline number with a caption and an optional fill bar.
///
/// A negative `fraction` hides the bar; tiles like TPS have no meaningful 0..1 scale.
pub fn stat_tile<'a, Message: 'a>(
    palette: Palette,
    label: impl text::IntoFragment<'a>,
    value: String,
    caption: String,
    accent: Color,
    fraction: f32,
) -> Element<'a, Message> {
    let mut inner = Column::new()
        .spacing(2)
        .push(section_title(palette, label))
        .push(
            text(value)
                .size(22)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::default()
                })
                .color(accent),
        );

    if fraction >= 0.0 {
        inner = inner.push(meter(palette, fraction));
    }

    inner = inner.push(text(caption).size(11).color(palette.fg_muted));

    container(inner)
        .width(Length::Fill)
        .padding(metric::GAP)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.surface)),
            border: edge(palette.border, metric::RADIUS),
            ..container::Style::default()
        })
        .into()
}

/// A thin filled bar, used by the stat tiles and the per-core grid.
pub fn meter<'a, Message: 'a>(palette: Palette, fraction: f32) -> Element<'a, Message> {
    let clamped = fraction.clamp(0.0, 1.0);
    let fill = container(filler())
        .width(Length::FillPortion((clamped * 1000.0) as u16))
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.load_color(fraction))),
            border: edge(Color::TRANSPARENT, 2.0),
            ..container::Style::default()
        });
    let rest = filler().width(Length::FillPortion(((1.0 - clamped) * 1000.0) as u16));

    container(Row::new().push(fill).push(rest))
        .width(Length::Fill)
        .height(4.0)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(palette.surface_alt)),
            border: edge(Color::TRANSPARENT, 2.0),
            ..container::Style::default()
        })
        .into()
}
