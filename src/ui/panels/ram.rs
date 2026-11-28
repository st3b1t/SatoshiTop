//! Ram panel, ported from `lib/system/ram.js`. The original donut becomes a
//! vertical bar 0–100%, which suits the narrow, tall panel better.

use malevich::{Bars, Plot};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::system::RamSample;
use crate::system::history::SysHistory;
use crate::ui::panel_block;
use crate::util::COLORS;
use crate::util::human::human_bytes;

pub const TITLE: &str = "Ram";

/// `Ram used/total`, e.g. `Ram 7.9GB/17.2GB` (the donut label of the
/// original).
pub fn title(ram: Option<&RamSample>) -> String {
    match ram {
        Some(ram) => format!(
            "{TITLE} {}/{}",
            human_bytes(ram.used() as f64),
            human_bytes(ram.total as f64)
        ),
        None => TITLE.to_owned(),
    }
}

/// Used memory in whole percent, rounded as in the original.
pub fn percent(ram: &RamSample) -> f64 {
    ram.used_percent().round()
}

/// One bar on a 0–100 axis, labeled with its percent.
pub fn ram_plot(ram: &RamSample) -> Plot<'static> {
    let percent = percent(ram);
    Plot::new()
        .layer(Bars::new([format!("{percent:.0}%")], vec![percent]).color(COLORS[3]))
        .y_domain(0.0, 100.0)
}

pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory) {
    let block = panel_block(title(history.ram.as_ref()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if let Some(ram) = &history.ram
        && !inner.is_empty()
    {
        frame.render_widget(ram_plot(ram).widget(), inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAM: RamSample = RamSample {
        total: 16_000_000_000,
        available: 8_600_000_000,
    };

    #[test]
    fn title_shows_used_and_total() {
        assert_eq!(title(Some(&RAM)), "Ram 7.4GB/16.0GB");
        assert_eq!(title(None), "Ram");
    }

    #[test]
    fn percent_is_rounded() {
        assert_eq!(percent(&RAM), 46.0);
    }

    #[test]
    fn the_bar_is_labeled_and_on_a_0_100_axis() {
        let rendered = ram_plot(&RAM).render(&malevich::Frame::plain(14, 9));
        assert!(rendered.contains("46%"), "{rendered}");
        assert!(rendered.contains("100"), "{rendered}");
    }
}
