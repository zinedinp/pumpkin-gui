//! The landing page: the numbers you check at a glance, with the console right underneath.
//!
//! The console gets the growing half deliberately — it is the part you actually work in, while
//! the stats above only need enough room to be readable.

use iced::widget::{Row, column};
use iced::{Element, Length};

use crate::app::{App, Message};
use crate::format;
use crate::theme::metric;
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

    let memory_fraction = if system.mem_system_total > 0 {
        system.mem_system_used as f32 / system.mem_system_total as f32
    } else {
        0.0
    };
    let disk_fraction = if snapshot.disk.total > 0 {
        1.0 - snapshot.disk.free as f32 / snapshot.disk.total as f32
    } else {
        0.0
    };
    let online = snapshot.players.iter().filter(|player| player.online).count();

    let tiles = Row::new()
        .spacing(metric::GAP)
        .push(w::stat_tile(
            palette,
            "TPS",
            format!("{:.2}", snapshot.tps),
            format!("{:.1} of {budget:.0} ms per tick", snapshot.mspt),
            palette.tps_color(snapshot.tps),
            -1.0,
        ))
        .push(w::stat_tile(
            palette,
            "CPU",
            format!("{:.0} %", system.cpu_total),
            format!("{} cores", system.cpu_per_core.len()),
            palette.load_color(system.cpu_total / 100.0),
            system.cpu_total / 100.0,
        ))
        .push(w::stat_tile(
            palette,
            "MEMORY",
            format::bytes(Some(system.mem_process_rss)),
            format!(
                "system {} / {}",
                format::bytes(Some(system.mem_system_used)),
                format::bytes(Some(system.mem_system_total))
            ),
            palette.accent,
            memory_fraction,
        ))
        .push(w::stat_tile(
            palette,
            "STORAGE",
            snapshot
                .worlds_size_bytes
                .map_or_else(|| "scanning…".to_owned(), |size| format::bytes(Some(size))),
            format!(
                "{} free of {}",
                format::bytes(Some(snapshot.disk.free)),
                format::bytes(Some(snapshot.disk.total))
            ),
            palette.accent,
            disk_fraction,
        ))
        .push(w::stat_tile(
            palette,
            "PLAYERS",
            online.to_string(),
            format!("up {}", format::duration(snapshot.uptime_secs)),
            palette.accent,
            -1.0,
        ));

    column![
        tiles,
        iced::widget::container(super::console::view(app))
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(metric::GAP)
    .into()
}
