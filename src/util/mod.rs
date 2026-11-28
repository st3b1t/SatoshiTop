//! Small helpers shared by the panels, ported from `lib/utils/index.js`.

pub mod human;

use malevich::Color;

/// Series palette, in the original order. Panels cycle through it (`i % 6`).
pub const COLORS: [Color; 6] = [
    Color::Magenta,
    Color::Cyan,
    Color::Blue,
    Color::Yellow,
    Color::Green,
    Color::Red,
];

/// Shortens `s` to at most `max` characters, ending with `...` when cut.
///
/// Counts characters rather than UTF-16 units. Unlike the original, which
/// returns a string longer than the input when `max < 3`, the result never
/// exceeds `max` characters: below 3 the text is cut without the ellipsis.
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    if max < 3 {
        return s.chars().take(max).collect();
    }
    let mut out: String = s.chars().take(max - 3).collect();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_follow_the_original_order() {
        assert_eq!(COLORS[0], Color::Magenta);
        assert_eq!(COLORS[5], Color::Red);
    }

    #[test]
    fn truncate_matches_js_for_valid_widths() {
        assert_eq!(truncate("127.0.0.1:8333", 20), "127.0.0.1:8333");
        assert_eq!(truncate("127.0.0.1:8333", 14), "127.0.0.1:8333");
        assert_eq!(truncate("127.0.0.1:8333", 13), "127.0.0.1:...");
        assert_eq!(truncate("abcdef", 3), "...");
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn truncate_never_exceeds_max() {
        // JavaScript gives "abcde..." and "abc..." here.
        assert_eq!(truncate("abcdef", 2), "ab");
        assert_eq!(truncate("abcdef", 0), "");
    }

    #[test]
    fn truncate_counts_characters() {
        assert_eq!(truncate("àèìòùáéíóú", 6), "àèì...");
    }
}
