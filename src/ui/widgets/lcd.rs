//! Seven-segment display, a replacement for the `lcd` widget of
//! blessed-contrib (malevich has no such mark).
//!
//! Digits scale to the area, as in the original. Supported characters:
//! `0`–`9`, `C`, `-` and space; anything else is blank.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};

/// Glyph used to draw the segments.
const SEGMENT: &str = "█";
/// Columns between two digits.
const GAP: u16 = 1;
/// Smallest digit that still reads as seven segments.
const MIN_WIDTH: u16 = 3;
const MIN_HEIGHT: u16 = 3;

/// Segments of a character, in the usual order: a (top), b (top right),
/// c (bottom right), d (bottom), e (bottom left), f (top left), g (middle).
fn segments(c: char) -> [bool; 7] {
    match c {
        '0' => [true, true, true, true, true, true, false],
        '1' => [false, true, true, false, false, false, false],
        '2' => [true, true, false, true, true, false, true],
        '3' => [true, true, true, true, false, false, true],
        '4' => [false, true, true, false, false, true, true],
        '5' => [true, false, true, true, false, true, true],
        '6' => [true, false, true, true, true, true, true],
        '7' => [true, true, true, false, false, false, false],
        '8' => [true; 7],
        '9' => [true, true, true, true, false, true, true],
        'C' => [true, false, false, true, true, true, false],
        '-' => [false, false, false, false, false, false, true],
        _ => [false; 7],
    }
}

/// A seven-segment rendering of `text`.
pub struct Lcd<'a> {
    text: &'a str,
    style: Style,
}

impl<'a> Lcd<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Lcd<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let count = self.text.chars().count() as u16;
        if count == 0 || area.is_empty() {
            return;
        }
        let width = ((area.width + GAP) / count).saturating_sub(GAP);
        let height = area.height;
        if width < MIN_WIDTH || height < MIN_HEIGHT {
            // Too small for segments: plain text.
            Paragraph::new(self.text)
                .style(self.style)
                .alignment(Alignment::Center)
                .render(area, buf);
            return;
        }
        // Keep digits from getting wider than tall-ish, and center them.
        let width = width.min(height.saturating_mul(2));
        let total = count * width + (count - 1) * GAP;
        let left = area.x + (area.width - total) / 2;

        for (i, c) in self.text.chars().enumerate() {
            let x0 = left + i as u16 * (width + GAP);
            draw_digit(buf, Rect::new(x0, area.y, width, height), c, self.style);
        }
    }
}

fn draw_digit(buf: &mut Buffer, cell: Rect, c: char, style: Style) {
    let [a, b, c_, d, e, f, g] = segments(c);
    let (x0, x1) = (cell.x, cell.x + cell.width - 1);
    let (y0, y1) = (cell.y, cell.y + cell.height - 1);
    let ym = y0 + (cell.height - 1) / 2;
    let mut set = |x: u16, y: u16| {
        buf[(x, y)].set_symbol(SEGMENT).set_style(style);
    };
    for (on, y) in [(a, y0), (g, ym), (d, y1)] {
        if on {
            (x0..=x1).for_each(|x| set(x, y));
        }
    }
    for (on, x, from, to) in [
        (f, x0, y0, ym),
        (b, x1, y0, ym),
        (e, x0, ym, y1),
        (c_, x1, ym, y1),
    ] {
        if on {
            (from..=to).for_each(|y| set(x, y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(text: &str, width: u16, height: u16) -> Vec<String> {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        Lcd::new(text).render(area, &mut buf);
        (0..height)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect()
    }

    #[test]
    fn digits_are_drawn_with_segments() {
        assert_eq!(draw("8", 3, 5), ["███", "█ █", "███", "█ █", "███"]);
        assert_eq!(draw("1", 3, 5), ["  █", "  █", "  █", "  █", "  █"]);
        assert_eq!(draw("C", 3, 3), ["███", "█  ", "███"]);
        assert_eq!(draw("-", 3, 3), ["   ", "███", "   "]);
    }

    #[test]
    fn several_characters_are_spaced_and_centered() {
        let rows = draw("45C", 13, 5);
        assert_eq!(rows[0], " █ █ ███ ███ ");
        assert_eq!(rows[2], " ███ ███ █   ");
        assert_eq!(rows[4], "   █ ███ ███ ");
    }

    #[test]
    fn small_areas_fall_back_to_text() {
        let rows = draw("45C", 8, 2);
        assert_eq!(rows[0].trim(), "45C");
        assert_eq!(rows[1].trim(), "");
    }

    #[test]
    fn empty_text_and_area_do_not_panic() {
        draw("", 5, 5);
        draw("45C", 1, 1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Lcd::new("8").render(Rect::new(0, 0, 0, 0), &mut buf);
    }
}
