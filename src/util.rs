//! Small shared helpers.

/// Lowercase, hyphen-separated slug for file names; never empty.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Parse a `#rrggbb` color string into `(red, green, blue)` channels. Missing or
/// malformed channels default to `0`.
pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let digits = hex.trim().trim_start_matches('#');
    let channel = |range: std::ops::Range<usize>| {
        digits
            .get(range)
            .and_then(|slice| u8::from_str_radix(slice, 16).ok())
            .unwrap_or(0)
    };
    (channel(0..2), channel(2..4), channel(4..6))
}

/// Compose `(red, green, blue)` channels into a `#rrggbb` string.
pub fn rgb_to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

/// Compose `(red, green, blue)` channels into an `r, g, b` string.
pub fn rgb_to_text(red: u8, green: u8, blue: u8) -> String {
    format!("{red}, {green}, {blue}")
}

/// Parse user-typed color text (`#rrggbb`, `rrggbb`, `r, g, b`, or
/// `rgb(r, g, b)`) into channels. Returns `None` when the text is not a color.
pub fn parse_color_text(text: &str) -> Option<(u8, u8, u8)> {
    let trimmed = text.trim();
    let hex = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if hex.len() == 6 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Some(hex_to_rgb(hex));
    }

    let inner = trimmed
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or(trimmed);
    let channels: Vec<u8> = inner
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .filter(|piece| !piece.is_empty())
        .map(|piece| piece.parse::<u8>())
        .collect::<Result<_, _>>()
        .ok()?;
    match channels[..] {
        [red, green, blue] => Some((red, green, blue)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_to_kebab_case() {
        assert_eq!(slug("Hello World!"), "hello-world");
        assert_eq!(slug("  a..b  "), "a-b");
    }

    #[test]
    fn empty_input_falls_back() {
        assert_eq!(slug("   "), "untitled");
    }

    #[test]
    fn hex_rgb_round_trips() {
        assert_eq!(hex_to_rgb("#f2efe9"), (242, 239, 233));
        assert_eq!(rgb_to_hex(242, 239, 233), "#f2efe9");
        let (r, g, b) = hex_to_rgb("#1b1b1b");
        assert_eq!(rgb_to_hex(r, g, b), "#1b1b1b");
    }

    #[test]
    fn malformed_hex_defaults_to_zero() {
        assert_eq!(hex_to_rgb("nope"), (0, 0, 0));
    }

    #[test]
    fn parses_hex_color_text() {
        assert_eq!(parse_color_text("#6fb6c8"), Some((111, 182, 200)));
        assert_eq!(parse_color_text("6FB6C8"), Some((111, 182, 200)));
        assert_eq!(parse_color_text("  #1b1b1b  "), Some((27, 27, 27)));
    }

    #[test]
    fn parses_rgb_color_text() {
        assert_eq!(parse_color_text("111, 182, 200"), Some((111, 182, 200)));
        assert_eq!(parse_color_text("0 0 255"), Some((0, 0, 255)));
        assert_eq!(parse_color_text("rgb(27, 27, 27)"), Some((27, 27, 27)));
    }

    #[test]
    fn rejects_invalid_color_text() {
        assert_eq!(parse_color_text(""), None);
        assert_eq!(parse_color_text("#12"), None);
        assert_eq!(parse_color_text("300, 0, 0"), None);
        assert_eq!(parse_color_text("1, 2"), None);
        assert_eq!(parse_color_text("blue"), None);
    }

    #[test]
    fn formats_rgb_text() {
        assert_eq!(rgb_to_text(111, 182, 200), "111, 182, 200");
    }
}
