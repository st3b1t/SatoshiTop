//! Processes panel, ported from `lib/system/proc.js`: a table sorted by CPU.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Row, Table};

use crate::system::ProcSample;
use crate::system::history::SysHistory;
use crate::ui::panel_block;

pub const TITLE: &str = "Processes";
/// Column headers, as in the original.
pub const HEADERS: [&str; 4] = ["%CPU", "%Ram", "User", "Command"];
/// Column widths and spacing of the original (`columnWidth: [4, 4, 6, 20]`,
/// `columnSpacing: 2`); the command takes the remaining width.
const WIDTHS: [Constraint; 4] = [
    Constraint::Length(4),
    Constraint::Length(4),
    Constraint::Length(6),
    Constraint::Fill(1),
];
const SPACING: u16 = 2;

/// Table rows, busiest process first, numbers with one decimal.
pub fn rows(processes: &[ProcSample]) -> Vec<[String; 4]> {
    let mut sorted: Vec<&ProcSample> = processes.iter().collect();
    sorted.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
    sorted
        .into_iter()
        .map(|p| {
            [
                format!("{:.1}", p.cpu),
                format!("{:.1}", p.mem),
                p.user.clone(),
                p.command.clone(),
            ]
        })
        .collect()
}

pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory) {
    // Only as many rows as fit: no need to build the others.
    let visible = area.height.saturating_sub(3) as usize;
    let rows = rows(&history.processes)
        .into_iter()
        .take(visible)
        .map(Row::new);
    let table = Table::new(rows, WIDTHS)
        .header(Row::new(HEADERS).style(Style::default().add_modifier(Modifier::BOLD)))
        .column_spacing(SPACING)
        // Rows are green in the original table widget.
        .style(Style::default().fg(Color::Green))
        .block(panel_block(TITLE));
    frame.render_widget(table, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(cpu: f64, command: &str) -> ProcSample {
        ProcSample {
            pid: 1,
            cpu,
            mem: 0.44,
            user: "bitcoin".into(),
            command: command.into(),
        }
    }

    #[test]
    fn rows_are_sorted_by_cpu_and_formatted() {
        let rows = rows(&[
            process(0.04, "init"),
            process(12.345, "bitcoind"),
            process(3.0, "tor"),
        ]);
        let commands: Vec<&str> = rows.iter().map(|r| r[3].as_str()).collect();
        assert_eq!(commands, ["bitcoind", "tor", "init"]);
        assert_eq!(rows[0], ["12.3", "0.4", "bitcoin", "bitcoind"]);
        assert_eq!(rows[2][0], "0.0");
    }
}
