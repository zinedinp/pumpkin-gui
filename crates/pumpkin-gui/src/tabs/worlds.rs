//! The world table. Flat rows, so it is the one page the `table` widget fits directly.

use iced::widget::{Row, column, container, scrollable};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::format;
use crate::theme::{Palette, metric};
use crate::widgets as w;

/// Column widths, matched to the Qt table so the two look the same side by side.
const NAME: f32 = 140.0;
const DIMENSION: f32 = 160.0;
const PLAYERS: f32 = 70.0;
const CHUNKS: f32 = 110.0;
const ENTITIES: f32 = 70.0;
const TIME: f32 = 110.0;
const WEATHER: f32 = 80.0;

fn weather_label(weather: &str) -> String {
    match weather {
        "rain" => "Rain".to_owned(),
        "thunder" => "Thunder".to_owned(),
        "clear" => "Clear".to_owned(),
        "none" => "–".to_owned(),
        other => other.to_owned(),
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();

    let header = Row::new()
        .push(w::header_cell(palette, "NAME", Length::Fixed(NAME)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "DIMENSION", Length::Fixed(DIMENSION)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "PLAYERS", Length::Fixed(PLAYERS)))
        .push(w::column_rule(palette))
        .push(w::header_cell(
            palette,
            "LOADED CHUNKS",
            Length::Fixed(CHUNKS),
        ))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "ENTITIES", Length::Fixed(ENTITIES)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "TIME", Length::Fixed(TIME)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "WEATHER", Length::Fixed(WEATHER)))
        .push(w::column_rule(palette))
        .push(w::header_cell(palette, "SIZE", Length::Fill))
        .height(24.0);

    let mut rows = column![];
    for (index, world) in app.worlds().iter().enumerate() {
        rows = rows.push(row_of(palette, index, world));
    }

    let body: Element<'_, Message> = if app.worlds().is_empty() {
        container(
            iced::widget::text("No worlds loaded.")
                .size(12)
                .color(palette.fg_muted),
        )
        .width(Length::Fill)
        .padding(12)
        .align_x(iced::Alignment::Center)
        .into()
    } else {
        scrollable(rows).width(Length::Fill).height(Length::Fill).into()
    };

    w::plain_card(
        palette,
        column![
            container(header).padding(iced::Padding::new(0.0).bottom(6.0)),
            w::row_rule(palette),
            body,
        ]
        .spacing(0),
    )
}

fn row_of(
    palette: Palette,
    index: usize,
    world: &pumpkin_gui_api::WorldRow,
) -> Element<'_, Message> {
    let cells = Row::new()
        .push(w::cell(
            palette,
            world.name.as_str(),
            Length::Fixed(NAME),
            false,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::dimension(&world.dimension),
            Length::Fixed(DIMENSION),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            world.players.to_string(),
            Length::Fixed(PLAYERS),
            false,
            true,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            world.loaded_chunks.to_string(),
            Length::Fixed(CHUNKS),
            false,
            true,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            world.entities.to_string(),
            Length::Fixed(ENTITIES),
            false,
            true,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::game_time(world.time_of_day),
            Length::Fixed(TIME),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            weather_label(&world.weather),
            Length::Fixed(WEATHER),
            true,
            false,
        ))
        .push(w::column_rule(palette))
        .push(w::cell(
            palette,
            format::bytes(world.size_bytes),
            Length::Fill,
            true,
            true,
        ));

    w::row_background(palette, index, cells)
        .height(metric::ROW_HEIGHT)
        .width(Length::Fill)
        .into()
}
