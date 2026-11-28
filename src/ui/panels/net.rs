//! Network panel, ported from `lib/system/net.js`: received and transmitted
//! rates of the default interface, each with a label and a sparkline.

use malevich::{Bars, Color, Plot};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;

use crate::system::history::{HISTORY_LEN, SysHistory};
use crate::ui::panel_block;
use crate::util::human::human_bytes;

pub const TITLE: &str = "Network";
/// Sparkline color, as in the original (`style.fg: 'blue'`).
const COLOR: Color = Color::Blue;

/// Panel title, with the interface when known: `Network (enp2s0)`.
pub fn title(history: &SysHistory) -> String {
    match history.net.as_ref().and_then(|n| n.iface.as_deref()) {
        Some(iface) => format!("{TITLE} ({iface})"),
        None => TITLE.to_owned(),
    }
}

/// The two labels of the original: `In: …/s Tot: …` and `Out: …/s Tot: …`.
pub fn labels(history: &SysHistory) -> (String, String) {
    let (rx, tx) = history.net_rates();
    let (rx_total, tx_total) = history
        .net
        .as_ref()
        .map_or((0, 0), |n| (n.rx_total, n.tx_total));
    (
        format!(
            "In: {}/s Tot: {}",
            human_bytes(rx),
            human_bytes(rx_total as f64)
        ),
        format!(
            "Out: {}/s Tot: {}",
            human_bytes(tx),
            human_bytes(tx_total as f64)
        ),
    )
}

/// A sparkline (bars without axes, like `malevich::sparkline`) in the panel
/// color. The x axis always spans the whole window, newest sample on the
/// right, so the line does not stretch while the history fills up.
pub fn sparkline(values: Vec<f64>) -> Plot<'static> {
    let start = HISTORY_LEN.saturating_sub(values.len()) as f64;
    Plot::new()
        .layer(Bars::spans(start, 1.0, values).color(COLOR))
        .axes(false)
        .x_domain(0.0, HISTORY_LEN as f64)
}

pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory) {
    let block = panel_block(title(history));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let (rx_label, tx_label) = labels(history);

    if inner.height < 4 {
        // No room for the sparklines: labels only.
        frame.render_widget(Paragraph::new(format!("{rx_label}\n{tx_label}")), inner);
        return;
    }
    let [rx_text, rx_chart, tx_text, tx_chart] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    frame.render_widget(Paragraph::new(rx_label), rx_text);
    frame.render_widget(Paragraph::new(tx_label), tx_text);
    for (ring, chart) in [(&history.rx, rx_chart), (&history.tx, tx_chart)] {
        let values = ring.snapshot();
        if !values.is_empty() && !chart.is_empty() {
            frame.render_widget(sparkline(values).widget(), chart);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::NetSample;

    fn history() -> SysHistory {
        let mut history = SysHistory::new();
        for (i, (rx, tx)) in [(1_000, 500), (2_500, 1_000), (5_000, 1_200)]
            .into_iter()
            .enumerate()
        {
            history.push_net(
                NetSample {
                    iface: Some("eth0".into()),
                    rx_total: rx,
                    tx_total: tx,
                },
                if i == 0 { 0.0 } else { 1.0 },
            );
        }
        history
    }

    #[test]
    fn labels_match_the_original_format() {
        let (rx, tx) = labels(&history());
        assert_eq!(rx, "In: 2.5KB/s Tot: 5.0KB");
        assert_eq!(tx, "Out: 200.0 B/s Tot: 1.2KB");
    }

    #[test]
    fn labels_before_any_sample() {
        let (rx, tx) = labels(&SysHistory::new());
        assert_eq!(rx, "In: 0.00 B/s Tot: 0.00 B");
        assert_eq!(tx, "Out: 0.00 B/s Tot: 0.00 B");
    }

    #[test]
    fn title_shows_the_interface() {
        assert_eq!(title(&history()), "Network (eth0)");
        assert_eq!(title(&SysHistory::new()), "Network");
    }

    #[test]
    fn sparklines_render_on_the_right() {
        let rendered = sparkline(vec![1.0, 5.0, 3.0]).render(&malevich::Frame::plain(61, 3));
        let first_line_with_ink = rendered.lines().find(|l| !l.trim().is_empty()).unwrap();
        assert!(first_line_with_ink.trim_end().len() > 50, "{rendered:?}");
    }
}
