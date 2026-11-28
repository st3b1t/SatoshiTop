//! Tor Network panel, ported from `lib/bitcoin/tor.js`.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

use crate::bitcoin::snapshot::NodeSnapshot;
use crate::bitcoin::tor::tor_address;
use crate::ui::panel_block;

pub const TITLE: &str = "Tor Network";

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>) {
    let block = panel_block(TITLE);
    let paragraph = match snapshot {
        Some(snapshot) if snapshot.connection_error().is_none() => match &snapshot.network {
            // Blue, as in the original.
            Ok(network) => {
                Paragraph::new(tor_address(network)).style(Style::default().fg(Color::Blue))
            }
            Err(e) => Paragraph::new(e.to_string()).style(Style::default().fg(Color::Red)),
        },
        _ => Paragraph::default(),
    };
    frame.render_widget(paragraph.block(block), area);
}
