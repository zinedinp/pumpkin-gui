//! Building console command lines.
//!
//! Every action in the window goes through the same dispatcher the terminal console uses, so it
//! has to survive being written out as text and read back by `StringReader`.

/// Renders a name or path as one command argument.
///
/// Bedrock gamertags may contain spaces and a plugin path contains separators, and Pumpkin's
/// `StringReader::read_string` accepts a quoted argument with backslash escapes, so both still
/// target correctly. Returns `None` for values that could not survive quoting.
#[must_use]
pub fn quote(value: &str) -> Option<String> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }

    // Mirrors `StringReader::is_allowed_in_unquoted_string`.
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
    {
        return Some(value.to_owned());
    }

    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    Some(format!("\"{escaped}\""))
}

/// Strips control characters from a free-text reason.
#[must_use]
pub fn sanitize_reason(reason: &str) -> String {
    reason
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_owned()
}

/// `<command> <target> [reason]`, or `None` when the target cannot be expressed as an argument.
#[must_use]
pub fn targeted(command: &str, target: &str, reason: Option<&str>) -> Option<String> {
    let target = quote(target)?;
    let mut line = format!("{command} {target}");

    if let Some(reason) = reason {
        let reason = sanitize_reason(reason);
        if !reason.is_empty() {
            // `kick`/`ban` take the reason as a GreedyPhrase, so it needs no quoting.
            line.push(' ');
            line.push_str(&reason);
        }
    }

    Some(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_names_are_not_quoted() {
        assert_eq!(quote("Steve_1"), Some("Steve_1".to_owned()));
    }

    #[test]
    fn names_with_spaces_are_quoted() {
        assert_eq!(quote("Some Gamertag"), Some("\"Some Gamertag\"".to_owned()));
    }

    #[test]
    fn a_path_survives_quoting() {
        assert_eq!(
            quote("plugins/TinyAC_[BETA].wasm"),
            Some("\"plugins/TinyAC_[BETA].wasm\"".to_owned())
        );
    }

    #[test]
    fn control_characters_are_refused() {
        assert_eq!(quote("bad\nname"), None);
    }

    #[test]
    fn a_reason_is_appended_unquoted() {
        assert_eq!(
            targeted("ban", "Steve", Some("griefing  ")),
            Some("ban Steve griefing".to_owned())
        );
    }

    #[test]
    fn an_empty_reason_is_left_off() {
        assert_eq!(targeted("kick", "Steve", Some("  ")), Some("kick Steve".to_owned()));
    }
}
