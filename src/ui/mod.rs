//! Terminal UI: layout and panels. Views are pure functions of the app state.

pub mod dashboard;
pub mod panels;
pub mod widgets;

use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Block;

/// Border color of every panel: `#06989a`.
pub const BORDER_COLOR: Color = Color::Rgb(0x06, 0x98, 0x9a);

/// The box around a panel: border in [`BORDER_COLOR`], title in the default
/// color.
pub fn panel_block<'a>(title: impl Into<Line<'a>>) -> Block<'a> {
    Block::bordered()
        .border_style(Style::default().fg(BORDER_COLOR))
        // Titles sit on the border line and would take its color.
        .title_style(Style::default().fg(Color::Reset))
        .title(title)
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    use super::*;

    #[test]
    fn borders_use_the_panel_color() {
        let area = Rect::new(0, 0, 10, 3);
        let mut buf = Buffer::empty(area);
        panel_block("Cpu").render(area, &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        assert_eq!(buf[(0, 0)].fg, BORDER_COLOR);
        assert_eq!(buf[(9, 2)].fg, BORDER_COLOR);
        // The title keeps the default color.
        assert_eq!(buf[(1, 0)].symbol(), "C");
        assert_eq!(buf[(1, 0)].fg, Color::Reset);
    }
}
