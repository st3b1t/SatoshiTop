//! Cpu panel, ported from `lib/system/cpu.js` and `lib/widget/line.js`: one
//! line per core, axis pinned to 0–100%, no legend.

use malevich::{Charset, Color, Line, Plot, Viewport};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::system::history::{HISTORY_LEN, SysHistory};
use crate::ui::panel_block;
use crate::util::COLORS;

/// Panel title, as in the original.
pub const TITLE: &str = "Cpu";

/// Line color of core `index`: the palette, cycled.
pub fn core_color(index: usize) -> Color {
    COLORS[index % COLORS.len()]
}

/// "Seconds ago" x positions for `len` samples taken every `interval`
/// seconds: the newest sample is at 0.
fn seconds_ago(len: usize, interval: f64) -> Vec<f64> {
    (0..len)
        .map(|i| (i as f64 - len.saturating_sub(1) as f64) * interval)
        .collect()
}

/// One line per core, in core order.
fn core_lines(history: &SysHistory, interval: f64) -> Vec<Line<'static>> {
    history
        .cores
        .iter()
        .enumerate()
        .map(|(i, ring)| {
            let load = ring.snapshot();
            let x = seconds_ago(load.len(), interval);
            Line::xy(x, load).color(core_color(i))
        })
        .collect()
}

/// The chart: one line per core. The x axis is pinned to the whole window
/// (61 samples) from the first sample on, so it does not rescale while the
/// history fills up.
pub fn cpu_plot(history: &SysHistory, interval: f64) -> Plot<'static> {
    let window = Viewport::auto().tail(0.0, (HISTORY_LEN - 1) as f64 * interval);
    core_lines(history, interval)
        .into_iter()
        .fold(Plot::new(), Plot::layer)
        .y_domain(0.0, 100.0)
        .viewport(window)
}

/// Draws the panel: a ratatui border with the title, the chart inside.
pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory, interval: f64) {
    let block = panel_block(TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 2 || inner.height < 2 {
        return;
    }
    let plot = cpu_plot(history, interval);
    frame.render_widget(plot.widget().charset(Charset::Braille), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::CpuSample;

    fn history(samples: usize, cores: usize) -> SysHistory {
        let mut history = SysHistory::new();
        for step in 0..samples {
            history.push_cpu(&CpuSample {
                per_core: (0..cores).map(|c| (10 * c + step) as f64 % 100.0).collect(),
            });
        }
        history
    }

    #[test]
    fn the_axis_is_pinned_to_one_hundred() {
        let rendered = cpu_plot(&history(20, 4), 1.0).render(&malevich::Frame::plain(60, 12));
        assert!(rendered.contains("100"), "{rendered}");
    }

    #[test]
    fn the_x_axis_covers_the_whole_window() {
        let rendered = cpu_plot(&history(3, 2), 1.0).render(&malevich::Frame::plain(60, 12));
        assert!(rendered.contains("-60"), "{rendered}");
    }

    #[test]
    fn one_line_per_core() {
        assert_eq!(core_lines(&history(10, 8), 1.0).len(), 8);
    }

    #[test]
    fn an_empty_history_renders() {
        let rendered = cpu_plot(&SysHistory::new(), 1.0).render(&malevich::Frame::plain(40, 10));
        assert!(!rendered.is_empty());
    }

    #[test]
    fn colors_cycle_through_the_palette() {
        assert_eq!(core_color(0), Color::Magenta);
        assert_eq!(core_color(5), Color::Red);
        assert_eq!(core_color(6), Color::Magenta);
        assert_eq!(core_color(7), Color::Cyan);
    }
}
