//! Mempool panel: a usage bar and the details (the original is a work in
//! progress showing a static gauge).

use malevich::{Bars, Color, Plot};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color as TextColor, Style};
use ratatui::widgets::Paragraph;

use crate::bitcoin::mempool::{lines, title, usage_percent};
use crate::bitcoin::snapshot::NodeSnapshot;
use crate::ui::panel_block;

pub const TITLE: &str = "Mempool";

/// Bar color by how full the mempool is: green, then yellow from 50%, red
/// from 80%.
pub fn usage_color(percent: f64) -> Color {
    if percent >= 80.0 {
        Color::Red
    } else if percent >= 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// One horizontal bar on a 0–100 axis, without axes.
pub fn usage_plot(percent: f64) -> Plot<'static> {
    Plot::new()
        .layer(
            Bars::new(["usage"], vec![percent])
                .horizontal()
                .color(usage_color(percent)),
        )
        .x_domain(0.0, 100.0)
        .axes(false)
}

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>) {
    let Some(snapshot) = snapshot.filter(|s| s.connection_error().is_none()) else {
        frame.render_widget(panel_block(TITLE), area);
        return;
    };
    let mempool = match &snapshot.mempool {
        Ok(mempool) => mempool,
        Err(e) => {
            let error = Paragraph::new(e.to_string())
                .style(Style::default().fg(TextColor::Red))
                .block(panel_block(TITLE));
            frame.render_widget(error, area);
            return;
        }
    };
    let block = panel_block(title(mempool));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }
    let [bar, _, text] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    frame.render_widget(usage_plot(usage_percent(mempool)).widget(), bar);
    frame.render_widget(Paragraph::new(lines(mempool).join("\n")), text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_follows_the_fill() {
        assert_eq!(usage_color(26.0), Color::Green);
        assert_eq!(usage_color(50.0), Color::Yellow);
        assert_eq!(usage_color(95.0), Color::Red);
    }

    #[test]
    fn the_bar_is_proportional() {
        let rendered = usage_plot(50.0).render(&malevich::Frame::plain(20, 1));
        let filled = rendered.chars().filter(|c| !c.is_whitespace()).count();
        assert!((8..=12).contains(&filled), "{rendered:?}");
    }
}
