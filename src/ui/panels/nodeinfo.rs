//! Bitcoin Node panel, ported from `lib/bitcoin/nodeinfo.js`, and the
//! connection error of `lib/satop.js`.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::bitcoin::nodeinfo::{NodeLine, NodeStatus, node_lines};
use crate::bitcoin::snapshot::NodeSnapshot;
use crate::ui::panel_block;

pub const TITLE: &str = "Bitcoin Node";

/// `Connection error: host:port message`, as in the original.
pub fn connection_error(snapshot: &NodeSnapshot, address: &str) -> Option<String> {
    snapshot
        .connection_error()
        .map(|e| format!("Connection error: {address} {e}"))
}

/// Colors of the original: yellow labels, green values, white `unknown`,
/// red while syncing.
fn styled(line: &NodeLine) -> Line<'static> {
    let value_color = match line.status {
        Some(NodeStatus::Syncing(_)) => Color::Red,
        _ => Color::Green,
    };
    let value = match &line.value {
        Some(value) => Span::styled(value.clone(), Style::default().fg(value_color)),
        None => Span::styled("unknown", Style::default().fg(Color::White)),
    };
    Line::from(vec![
        Span::styled(
            format!("{}: ", line.label),
            Style::default().fg(Color::Yellow),
        ),
        value,
    ])
}

/// The panel text: nothing before the first snapshot, the error when the
/// node cannot be reached, the four lines otherwise.
pub fn text(snapshot: Option<&NodeSnapshot>, address: &str) -> Vec<Line<'static>> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    if let Some(error) = connection_error(snapshot, address) {
        return vec![Line::styled(error, Style::default().fg(Color::Red))];
    }
    node_lines(snapshot).iter().map(styled).collect()
}

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>, address: &str) {
    let paragraph = Paragraph::new(text(snapshot, address))
        .wrap(Wrap { trim: true })
        .block(panel_block(TITLE));
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::rpc::RpcError;

    fn plain(lines: &[Line]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn empty_before_the_first_snapshot() {
        assert!(text(None, "127.0.0.1:8332").is_empty());
    }

    #[test]
    fn connection_error_as_in_the_original() {
        let snapshot = NodeSnapshot::failed(RpcError::Connection(
            "Connection refused (os error 111)".into(),
        ));
        assert_eq!(
            plain(&text(Some(&snapshot), "127.0.0.1:8332")),
            ["Connection error: 127.0.0.1:8332 Connection refused (os error 111)"]
        );
    }

    #[test]
    fn other_errors_are_not_connection_errors() {
        let snapshot = NodeSnapshot::failed(RpcError::Rpc {
            code: -1,
            message: "x".into(),
        });
        assert_eq!(
            plain(&text(Some(&snapshot), "h:1")),
            [
                "Blocks: unknown",
                "Status: unknown",
                "UpTime: unknown",
                "UserAgent: unknown"
            ]
        );
    }
}
