//! Temperature panel, ported from `lib/system/temp.js`: a red seven-segment
//! display and the session maximum in the title.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::system::history::SysHistory;
use crate::ui::panel_block;
use crate::ui::widgets::lcd::Lcd;

pub const TITLE: &str = "Temperature";

/// `Temperature` before the first reading, then `Temp (max 64C)` as in the
/// original; `--` without sensors.
pub fn title(history: &SysHistory) -> String {
    match history.temperature {
        None => TITLE.to_owned(),
        Some(_) => format!("Temp (max {}C)", degrees(history.max_temperature)),
    }
}

/// The display: `45C`, or `--C` without sensors.
pub fn display(history: &SysHistory) -> String {
    format!("{}C", degrees(history.temperature.and_then(|t| t.main)))
}

fn degrees(value: Option<f64>) -> String {
    value.map_or_else(|| "--".to_owned(), |t| format!("{t:.0}"))
}

pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory) {
    let block = panel_block(title(history));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if history.temperature.is_some() {
        let text = display(history);
        frame.render_widget(
            Lcd::new(&text).style(Style::default().fg(Color::Red)),
            inner,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::TempSample;

    #[test]
    fn before_the_first_reading() {
        let history = SysHistory::new();
        assert_eq!(title(&history), "Temperature");
    }

    #[test]
    fn with_sensors() {
        let mut history = SysHistory::new();
        history.push_temperature(TempSample {
            main: Some(64.4),
            max: Some(70.0),
        });
        history.push_temperature(TempSample {
            main: Some(45.2),
            max: Some(50.0),
        });
        assert_eq!(title(&history), "Temp (max 70C)");
        assert_eq!(display(&history), "45C");
    }

    #[test]
    fn without_sensors() {
        let mut history = SysHistory::new();
        history.push_temperature(TempSample::default());
        assert_eq!(title(&history), "Temp (max --C)");
        assert_eq!(display(&history), "--C");
    }
}
