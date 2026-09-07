//! Tick health, per-core CPU load and network throughput.

use iced::widget::canvas::{self, Frame, Geometry, Path, Text};
use iced::widget::{Column, Row, column, container, row, scrollable, text};
use iced::{Color, Element, Font, Length, Point, Rectangle, Renderer, Size, Theme, mouse};

use crate::app::{App, Message};
use crate::format;
use crate::theme::{Palette, metric};
use crate::widgets as w;

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();
    let snapshot = app.snapshot();
    let system = &snapshot.system;
    let budget = if app.meta().tick_budget_ms > 0.0 {
        app.meta().tick_budget_ms
    } else {
        50.0
    };

    // Tick health first: it is the metric that decides whether the server is actually well.
    let ticks: Vec<f32> = snapshot
        .tick_times_nanos
        .iter()
        .map(|nanos| *nanos as f32 / 1_000_000.0)
        .collect();

    let tick_card = w::card(
        palette,
        "TICK PERFORMANCE",
        column![
            row![
                headline(
                    palette,
                    format!("{:.2}", snapshot.tps),
                    "TPS".to_owned(),
                    palette.tps_color(snapshot.tps),
                ),
                headline(
                    palette,
                    format!("{:.2} ms", snapshot.mspt),
                    format!("MSPT (budget {budget:.0} ms)"),
                    palette.load_color((snapshot.mspt / budget) as f32),
                ),
                w::filler(),
            ]
            .spacing(24),
            iced::widget::Canvas::new(TickGraph {
                samples: ticks,
                budget: budget as f32,
                palette,
            })
            .width(Length::Fill)
            .height(90.0),
        ]
        .spacing(8),
    );

    let cpu_title = if system.cpu_temp_c.is_some_and(|temp| temp >= 0.0) {
        format!(
            "CPU — {:.1} %  ·  {:.0} °C",
            system.cpu_total,
            system.cpu_temp_c.unwrap_or_default()
        )
    } else {
        format!("CPU — {:.1} %", system.cpu_total)
    };
    let cpu_card = w::card(palette, cpu_title, core_grid(palette, &system.cpu_per_core));

    let network_card = w::card(
        palette,
        "NETWORK",
        row![
            headline(
                palette,
                format!("↓ {}", format::rate(snapshot.net_in_bps)),
                "inbound".to_owned(),
                palette.fg,
            ),
            headline(
                palette,
                format!("↑ {}", format::rate(snapshot.net_out_bps)),
                "outbound".to_owned(),
                palette.fg,
            ),
            w::filler(),
            addresses(app),
        ]
        .spacing(32)
        .align_y(iced::Alignment::Center),
    );

    scrollable(
        column![tick_card, cpu_card, network_card]
            .spacing(metric::GAP)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// A big number over its caption.
fn headline<'a>(
    palette: Palette,
    value: String,
    caption: String,
    accent: Color,
) -> Element<'a, Message> {
    column![
        text(value)
            .size(30)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            })
            .color(accent),
        text(caption).size(11).color(palette.fg_muted),
    ]
    .spacing(0)
    .into()
}

fn addresses(app: &App) -> Element<'_, Message> {
    let palette = app.palette();
    let mut row = Row::new().spacing(16);

    for (value, label) in [
        (&app.meta().java_address, "Java Edition"),
        (&app.meta().bedrock_address, "Bedrock Edition"),
    ] {
        if !value.is_empty() {
            row = row.push(
                column![
                    text(value.clone())
                        .size(13)
                        .font(Font::MONOSPACE)
                        .color(palette.fg),
                    text(label).size(11).color(palette.fg_muted),
                ]
                .align_x(iced::Alignment::End),
            );
        }
    }

    row.into()
}

/// One bar per CPU core, btop-style: the bar fills with load and shifts green -> yellow -> red.
///
/// Reflows into a grid so a 4-core laptop and a 32-core server both look sensible.
fn core_grid(palette: Palette, usages: &[f32]) -> Element<'_, Message> {
    let columns = usages.len().isqrt().clamp(1, 8);
    let mut grid = Column::new().spacing(4);
    let mut current = Row::new().spacing(10);

    for (index, usage) in usages.iter().enumerate() {
        current = current.push(
            row![
                text(index.to_string())
                    .size(10)
                    .font(Font::MONOSPACE)
                    .color(palette.fg_muted)
                    .width(18.0)
                    .align_x(iced::Alignment::End),
                container(w::meter(palette, usage / 100.0))
                    .width(Length::Fill)
                    .height(12.0),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .width(Length::Fill),
        );

        if (index + 1) % columns == 0 {
            grid = grid.push(current);
            current = Row::new().spacing(10);
        }
    }

    // The last row is usually short; padding it keeps the columns aligned instead of stretching.
    let remainder = usages.len() % columns;
    if remainder != 0 {
        for _ in remainder..columns {
            current = current.push(w::filler());
        }
        grid = grid.push(current);
    }

    grid.into()
}

/// The server's rolling window of the last 100 tick durations, oldest on the left.
struct TickGraph {
    samples: Vec<f32>,
    /// How long a tick may take at the configured tick rate.
    budget: f32,
    palette: Palette,
}

impl canvas::Program<Message> for TickGraph {
    type State = ();

    fn draw(
        &self,
        (): &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill(
            &Path::rounded_rectangle(Point::ORIGIN, bounds.size(), 4.0.into()),
            self.palette.surface_alt,
        );

        if self.samples.is_empty() {
            return vec![frame.into_geometry()];
        }

        let inset = 2.0;
        let plot = Size::new(bounds.width - 2.0 * inset, bounds.height - 2.0 * inset);

        // The scale floors at the budget so an idle server does not stretch 2 ms ticks to the top.
        let peak = self
            .samples
            .iter()
            .copied()
            .fold(0.0f32, f32::max)
            .max(self.budget);

        let count = self.samples.len() as f32;
        let bar_width = (plot.width - (count - 1.0)) / count;

        for (index, sample) in self.samples.iter().enumerate() {
            let height = (plot.height * (sample / peak).min(1.0)).max(1.0);
            frame.fill_rectangle(
                Point::new(
                    inset + index as f32 * (bar_width + 1.0),
                    inset + plot.height - height,
                ),
                Size::new(bar_width, height),
                self.palette.load_color(sample / self.budget),
            );
        }

        // Ticks that cross this line missed the configured tick rate. With nothing overrunning it
        // sits at the top of the scale.
        frame.fill_rectangle(
            Point::new(inset, inset + plot.height * (1.0 - self.budget / peak)),
            Size::new(plot.width, 1.0),
            self.palette.border,
        );

        let sample_peak = self.samples.iter().copied().fold(0.0f32, f32::max);
        frame.fill_text(Text {
            content: format!("peak {sample_peak:.0} ms"),
            position: Point::new(bounds.width - 6.0, 6.0),
            color: self.palette.fg_muted,
            size: 10.0.into(),
            font: Font::MONOSPACE,
            align_x: iced::alignment::Horizontal::Right.into(),
            align_y: iced::alignment::Vertical::Top,
            ..Text::default()
        });

        vec![frame.into_geometry()]
    }
}
