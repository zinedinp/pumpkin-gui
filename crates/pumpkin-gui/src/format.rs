//! Shared value formatting, so a byte count reads the same on every page.

/// `None` and negatives render as an en dash: the samplers use -1 for "not measured yet".
const UNKNOWN: &str = "–";

const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

#[must_use]
pub fn bytes(value: Option<u64>) -> String {
    let Some(value) = value else {
        return UNKNOWN.to_owned();
    };

    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    // Bytes and KiB have no meaningful fraction; larger units do.
    let decimals = usize::from(unit >= 2);
    format!("{size:.decimals$} {}", UNITS[unit])
}

#[must_use]
pub fn rate(bytes_per_second: u64) -> String {
    format!("{}/s", bytes(Some(bytes_per_second)))
}

#[must_use]
pub fn duration(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

/// In-world clock: 24 000 ticks to a day, and tick 0 is 06:00 rather than midnight.
#[must_use]
pub fn game_time(time_of_day: i64) -> String {
    let day = time_of_day.div_euclid(24_000) + 1;
    let ticks = time_of_day.rem_euclid(24_000);
    let total_minutes = (ticks * 60 / 1000 + 360) % 1440;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("Day {day} · {hours:02}:{minutes:02}")
}

/// A dimension without its namespace, which is the same on every row and only costs width.
#[must_use]
pub fn dimension(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// Joins a list for a single-line cell, or the en dash when it is empty.
#[must_use]
pub fn list(values: &[String]) -> String {
    if values.is_empty() {
        UNKNOWN.to_owned()
    } else {
        values.join(", ")
    }
}

/// Falls back to the en dash so an empty cell still reads as "nothing here".
#[must_use]
pub const fn or_dash(value: &str) -> &str {
    if value.is_empty() { UNKNOWN } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_switches_to_fractions_above_kib() {
        assert_eq!(bytes(Some(512)), "512 B");
        assert_eq!(bytes(Some(2048)), "2 KiB");
        assert_eq!(bytes(Some(3 * 1024 * 1024)), "3.0 MiB");
        assert_eq!(bytes(None), "–");
    }

    #[test]
    fn duration_drops_the_units_it_does_not_need() {
        assert_eq!(duration(45), "45s");
        assert_eq!(duration(3661), "1h 1m");
        assert_eq!(duration(90_061), "1d 1h");
    }

    #[test]
    fn game_time_starts_the_day_at_six() {
        assert_eq!(game_time(0), "Day 1 · 06:00");
        // Ticks run negative on a world whose time has been set backwards.
        assert_eq!(game_time(-1), "Day 0 · 05:59");
    }
}
