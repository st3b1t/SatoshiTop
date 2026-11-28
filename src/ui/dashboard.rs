//! The dashboard layout, ported from the 12x12 grid of `lib/satop.js`.
//!
//! The Tor Network panel, the last grid row in the original, gets a fixed
//! height at the bottom so that its line of text is always visible; the other
//! panels share the rest on an 11-row by 12-column grid.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::ui::panel_block;
use crate::ui::panels;

/// Columns of the grid.
pub const GRID_COLS: u16 = 12;
/// Rows of the grid above the Tor Network panel.
pub const GRID_ROWS: u16 = 11;
/// Height of the Tor Network panel: border and one line of text.
pub const TOR_HEIGHT: u16 = 3;
/// Smallest terminal the dashboard is drawn in.
pub const MIN_WIDTH: u16 = 40;
pub const MIN_HEIGHT: u16 = 12;

/// A panel and its grid cell: row, column, row span, column span.
///
/// The original uses fractional spans for Network (3.2), Processes (4.2) and
/// Temperature (2.2), which overlap the panels below; they are rounded down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub title: &'static str,
    pub row: u16,
    pub col: u16,
    pub rows: u16,
    pub cols: u16,
}

const fn cell(title: &'static str, row: u16, col: u16, rows: u16, cols: u16) -> Cell {
    Cell {
        title,
        row,
        col,
        rows,
        cols,
    }
}

/// Every panel of the original, in its order. Tor Network keeps its grid row
/// (11) for reference, but is placed by [`panel_rect`] at a fixed height.
pub const PANELS: [Cell; 12] = [
    cell(panels::cpu::TITLE, 0, 0, 3, 5),
    cell(panels::net::TITLE, 3, 0, 3, 5),
    cell(panels::ram::TITLE, 0, 5, 4, 2),
    cell(panels::disks::TITLE, 0, 7, 4, 5),
    cell(panels::procs::TITLE, 4, 8, 4, 4),
    cell(panels::temp::TITLE, 4, 5, 2, 3),
    cell(panels::nodeinfo::TITLE, 6, 0, 2, 3),
    cell(panels::fees::TITLE, 6, 3, 2, 5),
    cell(panels::mempool::TITLE, 8, 0, 3, 4),
    cell(panels::blocks::TITLE, 8, 4, 3, 4),
    cell(panels::peers::TITLE, 8, 8, 3, 4),
    cell(panels::tor::TITLE, 11, 0, 1, 12),
];

/// Whether the dashboard fits in `area`.
pub fn fits(area: Rect) -> bool {
    area.width >= MIN_WIDTH && area.height >= MIN_HEIGHT
}

/// The screen area of a panel. Grid edges are computed from cumulative
/// positions, so adjacent panels never overlap or leave gaps.
pub fn panel_rect(area: Rect, cell: &Cell) -> Rect {
    let tor = TOR_HEIGHT.min(area.height);
    if cell.title == panels::tor::TITLE {
        return Rect::new(area.x, area.bottom() - tor, area.width, tor);
    }
    let grid = Rect::new(area.x, area.y, area.width, area.height - tor);
    let edge = |start: u16, size: u16, index: u16, count: u16| {
        start + (u32::from(size) * u32::from(index.min(count)) / u32::from(count)) as u16
    };
    let x0 = edge(grid.x, grid.width, cell.col, GRID_COLS);
    let x1 = edge(grid.x, grid.width, cell.col + cell.cols, GRID_COLS);
    let y0 = edge(grid.y, grid.height, cell.row, GRID_ROWS);
    let y1 = edge(grid.y, grid.height, cell.row + cell.rows, GRID_ROWS);
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

/// Two short lines, to fit narrow terminals: `Terminal too small` and
/// `35x10 (min 40x12)`.
pub fn too_small_message(area: Rect) -> [String; 2] {
    [
        "Terminal too small".to_owned(),
        format!(
            "{}x{} (min {MIN_WIDTH}x{MIN_HEIGHT})",
            area.width, area.height
        ),
    ]
}

/// Draws the whole dashboard.
pub fn draw(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    if !fits(area) {
        let height = area.height.min(2);
        let y = area.y + (area.height - height) / 2;
        let lines = Rect::new(area.x, y, area.width, height);
        let message =
            Paragraph::new(too_small_message(area).join("\n")).alignment(Alignment::Center);
        frame.render_widget(message, lines);
        return;
    }
    for cell in &PANELS {
        let rect = panel_rect(area, cell);
        if rect.width == 0 || rect.height == 0 {
            continue;
        }
        match cell.title {
            panels::cpu::TITLE => {
                panels::cpu::render(frame, rect, &state.sys, state.cadence.fast.as_secs_f64())
            }
            panels::net::TITLE => panels::net::render(frame, rect, &state.sys),
            panels::ram::TITLE => panels::ram::render(frame, rect, &state.sys),
            panels::disks::TITLE => panels::disks::render(frame, rect, &state.sys),
            panels::procs::TITLE => panels::procs::render(frame, rect, &state.sys),
            panels::temp::TITLE => panels::temp::render(frame, rect, &state.sys),
            panels::nodeinfo::TITLE => {
                panels::nodeinfo::render(frame, rect, state.node.as_ref(), &state.node_address)
            }
            panels::peers::TITLE => panels::peers::render(frame, rect, state.node.as_ref()),
            panels::tor::TITLE => panels::tor::render(frame, rect, state.node.as_ref()),
            panels::fees::TITLE => panels::fees::render(frame, rect, state.node.as_ref()),
            panels::mempool::TITLE => panels::mempool::render(frame, rect, state.node.as_ref()),
            panels::blocks::TITLE => panels::blocks::render(frame, rect, state.node.as_ref()),
            title => frame.render_widget(panel_block(title), rect),
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::system::CpuSample;

    fn rects(width: u16, height: u16) -> Vec<(&'static str, Rect)> {
        let area = Rect::new(0, 0, width, height);
        PANELS
            .iter()
            .map(|c| (c.title, panel_rect(area, c)))
            .collect()
    }

    #[test]
    fn rects_at_80x24() {
        let r = |x, y, w, h| Rect::new(x, y, w, h);
        assert_eq!(
            rects(80, 24),
            [
                ("Cpu", r(0, 0, 33, 5)),
                ("Network", r(0, 5, 33, 6)),
                ("Ram", r(33, 0, 13, 7)),
                ("Disks", r(46, 0, 34, 7)),
                ("Processes", r(53, 7, 27, 8)),
                ("Temperature", r(33, 7, 20, 4)),
                ("Bitcoin Node", r(0, 11, 20, 4)),
                ("Fees", r(20, 11, 33, 4)),
                ("Mempool", r(0, 15, 26, 6)),
                ("Blocks", r(26, 15, 27, 6)),
                ("Peers", r(53, 15, 27, 6)),
                ("Tor Network", r(0, 21, 80, 3)),
            ]
        );
    }

    #[test]
    fn rects_at_40x15() {
        let r = |x, y, w, h| Rect::new(x, y, w, h);
        assert_eq!(
            rects(40, 15),
            [
                ("Cpu", r(0, 0, 16, 3)),
                ("Network", r(0, 3, 16, 3)),
                ("Ram", r(16, 0, 7, 4)),
                ("Disks", r(23, 0, 17, 4)),
                ("Processes", r(26, 4, 14, 4)),
                ("Temperature", r(16, 4, 10, 2)),
                ("Bitcoin Node", r(0, 6, 10, 2)),
                ("Fees", r(10, 6, 16, 2)),
                ("Mempool", r(0, 8, 13, 4)),
                ("Blocks", r(13, 8, 13, 4)),
                ("Peers", r(26, 8, 14, 4)),
                ("Tor Network", r(0, 12, 40, 3)),
            ]
        );
    }

    #[test]
    fn panels_tile_the_screen_at_every_size() {
        for width in (MIN_WIDTH..=200).step_by(7) {
            for height in (MIN_HEIGHT..=60).step_by(5) {
                let rects = rects(width, height);
                let covered: u32 = rects
                    .iter()
                    .map(|(_, r)| u32::from(r.width) * u32::from(r.height))
                    .sum();
                assert_eq!(
                    covered,
                    u32::from(width) * u32::from(height),
                    "{width}x{height}"
                );
                for (i, (_, a)) in rects.iter().enumerate() {
                    for (_, b) in &rects[i + 1..] {
                        assert!(!a.intersects(*b), "{width}x{height}: {a:?} overlaps {b:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn tor_always_shows_its_line() {
        for height in [MIN_HEIGHT, 24, 60] {
            let tor = rects(80, height)[11].1;
            assert_eq!(tor.height, TOR_HEIGHT);
            assert_eq!(tor.bottom(), height);
        }
    }

    fn render(width: u16, height: u16, state: &AppState) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, state)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn every_panel_title_is_drawn() {
        let mut state = AppState::for_tests();
        for step in 0..5 {
            state.sys.push_cpu(&CpuSample {
                per_core: vec![10.0 * step as f64, 50.0],
            });
        }
        let screen = render(120, 40, &state);
        for cell in &PANELS {
            assert!(
                screen.contains(cell.title),
                "missing {}:\n{screen}",
                cell.title
            );
        }
        assert!(screen.contains("100"), "cpu axis:\n{screen}");
    }

    async fn fake_snapshot() -> crate::bitcoin::snapshot::NodeSnapshot {
        let mut fetcher =
            crate::bitcoin::snapshot::SnapshotFetcher::new(crate::tests::rpc_fake::FakeRpc::new(1));
        // Twice, so that the fake sync animation has started.
        fetcher.fetch().await;
        fetcher.fetch().await
    }

    #[tokio::test]
    async fn bitcoin_text_panels_show_the_snapshot() {
        let mut state = AppState::for_tests();
        state.node = Some(fake_snapshot().await);
        let screen = render(160, 48, &state);
        for text in [
            "Blocks: 840004",
            "Status: Syncing 7%",
            "UserAgent: Satoshi:22.0.0",
            "connected)",
            "UserAgent",
            "satoshitopm72cepqoa2l54adxtg3qwdd7ci6oddzcco5yjqd7mrdwqk.onion:8333",
            "Transaction Fees",
            "SvB",
            "Mempool ( 78.8MB / 300.0MB )",
            "Transactions: 70853",
            "Blocks (tot 556.7GB)",
            "840004",
            "9m29s",
        ] {
            assert!(screen.contains(text), "missing {text:?}:\n{screen}");
        }
    }

    #[test]
    fn connection_error_in_the_node_panel() {
        use crate::bitcoin::rpc::RpcError;
        let mut state = AppState::for_tests();
        state.node = Some(crate::bitcoin::snapshot::NodeSnapshot::failed(
            RpcError::Connection("refused".into()),
        ));
        let screen = render(160, 48, &state);
        // The message wraps inside the narrow panel.
        assert!(
            screen.contains("Connection error: 127.0.0.1:8332") && screen.contains("refused"),
            "{screen}"
        );
        assert!(!screen.contains("connected)"), "{screen}");
    }

    #[test]
    fn small_terminals_get_a_message() {
        let screen = render(35, 10, &AppState::for_tests());
        assert!(screen.contains("Terminal too small"), "{screen}");
        assert!(screen.contains("35x10 (min 40x12)"), "{screen}");
        assert!(!screen.contains("Cpu"), "{screen}");
        let screen = render(40, 12, &AppState::for_tests());
        assert!(screen.contains("Cpu"), "{screen}");
    }

    /// Every size from nothing to large, with every panel full of data.
    #[tokio::test]
    async fn no_size_panics() {
        let mut state = AppState::for_tests();
        let mut sys = crate::tests::sys_fake::FakeSys::new(1);
        use crate::system::SysSource;
        for _ in 0..3 {
            state.sys.push_cpu(&sys.cpu());
            state.sys.push_net(sys.net(), 1.0);
        }
        state.sys.push_ram(sys.ram());
        state.sys.push_disks(sys.disks());
        state.sys.push_processes(sys.processes());
        state.sys.push_temperature(sys.temperature());
        state.node = Some(fake_snapshot().await);
        for width in (0..=200).step_by(9) {
            for height in (0..=60).step_by(4) {
                render(width, height, &state);
            }
        }
    }
}
