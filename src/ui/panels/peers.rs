//! Peers panel, ported from `lib/bitcoin/peers.js`.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Row, Table};

use crate::bitcoin::peers::{peer_rows, title};
use crate::bitcoin::snapshot::NodeSnapshot;
use crate::ui::panel_block;

pub const TITLE: &str = "Peers";
pub const HEADERS: [&str; 2] = ["UserAgent", "Address"];
/// Width of the user agent column, as in the original (`columnWidth: [18, 30]`).
const AGENT_WIDTH: u16 = 18;

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>) {
    let peers = match snapshot {
        Some(snapshot) if snapshot.connection_error().is_none() => &snapshot.peers,
        // Nothing yet, or no node: an empty box, as in the original.
        _ => {
            frame.render_widget(panel_block(TITLE), area);
            return;
        }
    };
    let peers = match peers {
        Ok(peers) => peers,
        Err(e) => {
            let error = Paragraph::new(e.to_string())
                .style(Style::default().fg(Color::Red))
                .block(panel_block(TITLE));
            frame.render_widget(error, area);
            return;
        }
    };
    // The address column takes the rest of the width (the original uses a
    // fixed 30 and a TODO to follow the screen width).
    let address_width = area.width.saturating_sub(2 + AGENT_WIDTH) as usize;
    let visible = area.height.saturating_sub(3) as usize;
    let rows = peer_rows(peers, address_width)
        .into_iter()
        .take(visible)
        .map(Row::new);
    let table = Table::new(rows, [Constraint::Length(AGENT_WIDTH), Constraint::Fill(1)])
        .header(Row::new(HEADERS))
        .column_spacing(0)
        // Rows are green in the original table widget.
        .style(Style::default().fg(Color::Green))
        .block(panel_block(title(peers)));
    frame.render_widget(table, area);
}
