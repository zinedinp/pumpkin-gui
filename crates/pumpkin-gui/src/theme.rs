//! Every colour and measurement in the UI comes from here, so switching themes is one value
//! change rather than a sweep through the widgets.

use iced::{Color, Theme};

/// Shorthand for the hex literals the palette is written in.
const fn rgb(hex: u32) -> Color {
    Color::from_rgb8(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

/// The full set of colour roles the UI draws with.
///
/// Wider than iced's own six-colour `Palette`, which has nothing for table rows or hairlines.
/// `Copy` so style closures can capture it without borrowing the application state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,

    // Surfaces
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub border: Color,
    /// Hairline between chrome segments and table rows.
    pub rule: Color,
    pub row_even: Color,
    pub row_odd: Color,
    pub row_hover: Color,

    // Text
    pub fg: Color,
    pub fg_muted: Color,

    // Accents. `accent` is Pumpkin's orange in both themes, lightened for dark backgrounds.
    pub accent: Color,
    pub good: Color,
    pub warn: Color,
    pub danger: Color,
}

impl Palette {
    pub const DARK: Self = Self {
        dark: true,
        background: rgb(0x16181d),
        surface: rgb(0x1e2128),
        surface_alt: rgb(0x262a33),
        border: rgb(0x333844),
        rule: rgb(0x4a5160),
        row_even: rgb(0x1e2128),
        row_odd: rgb(0x252a33),
        row_hover: rgb(0x30363f),
        fg: rgb(0xffffff),
        fg_muted: rgb(0x9aa1ae),
        accent: rgb(0xff9d4d),
        good: rgb(0x5ac37f),
        warn: rgb(0xe8c05a),
        danger: rgb(0xe8705f),
    };

    pub const LIGHT: Self = Self {
        dark: false,
        background: rgb(0xf6f7f9),
        surface: rgb(0xffffff),
        surface_alt: rgb(0xeef0f4),
        border: rgb(0xd8dce4),
        rule: rgb(0xb8c0cc),
        row_even: rgb(0xffffff),
        row_odd: rgb(0xeef1f5),
        row_hover: rgb(0xe2e7ee),
        fg: rgb(0x000000),
        fg_muted: rgb(0x5c6472),
        accent: rgb(0xe0701a),
        good: rgb(0x2f8f52),
        warn: rgb(0xa97a10),
        danger: rgb(0xc2412e),
    };

    #[must_use]
    pub const fn of(dark: bool) -> Self {
        if dark { Self::DARK } else { Self::LIGHT }
    }

    /// The iced `Theme` the built-in widgets fall back on where nothing overrides them.
    #[must_use]
    pub fn iced(self) -> Theme {
        Theme::custom(
            if self.dark { "Pumpkin Dark" } else { "Pumpkin Light" }.to_owned(),
            iced::theme::Palette {
                background: self.background,
                text: self.fg,
                primary: self.accent,
                success: self.good,
                warning: self.warn,
                danger: self.danger,
            },
        )
    }

    #[must_use]
    pub const fn with_alpha(color: Color, alpha: f32) -> Color {
        Color { a: alpha, ..color }
    }

    /// Tint used behind pills and other lightly filled chips.
    #[must_use]
    pub const fn tint(self, accent: Color) -> Color {
        Self::with_alpha(accent, if self.dark { 0.22 } else { 0.16 })
    }

    #[must_use]
    pub const fn tint_border(self, accent: Color) -> Color {
        Self::with_alpha(accent, if self.dark { 0.7 } else { 0.55 })
    }

    /// Shared threshold so the TPS badge, the tick graph and the core bars agree on what
    /// "healthy" looks like.
    #[must_use]
    pub const fn load_color(self, fraction: f32) -> Color {
        if fraction >= 0.9 {
            self.danger
        } else if fraction >= 0.6 {
            self.warn
        } else {
            self.good
        }
    }

    /// TPS is healthy at 20 and bad as it drops, so the scale runs the other way.
    #[must_use]
    pub const fn tps_color(self, tps: f64) -> Color {
        if tps >= 19.0 {
            self.good
        } else if tps >= 15.0 {
            self.warn
        } else {
            self.danger
        }
    }
}

/// Radii, gaps and text sizes, kept together so tables and cards stay aligned.
pub mod metric {
    pub const RADIUS: f32 = 8.0;
    pub const GAP: f32 = 12.0;
    /// Inset for the first and last table cells, so labels are not flush with the card edge.
    pub const TABLE_EDGE: f32 = 12.0;
    /// Horizontal padding on either side of a column.
    pub const TABLE_CELL_PAD: f32 = 8.0;
    pub const TABLE_HEADER_SIZE: f32 = 12.0;
    pub const TABLE_CELL_SIZE: f32 = 13.0;
    /// Shared control height, so a search field lines up with the buttons beside it.
    pub const CONTROL_HEIGHT: f32 = 32.0;
    pub const ROW_HEIGHT: f32 = 34.0;
}
