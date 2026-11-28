//! Blocks panel, ported from `lib/bitcoin/blocks.js` and the custom `block`
//! widget: the last four blocks as blue columns as tall as their size.

use malevich::Color;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color as TextColor, Style};
use ratatui::widgets::Paragraph;

use super::columns::{Column, ColumnStyle, column_plot};
use crate::bitcoin::blocks::{block_columns, title, y_max};
use crate::bitcoin::snapshot::NodeSnapshot;
use crate::ui::panel_block;

pub const TITLE: &str = "Blocks";
/// Narrowest column that fits the full labels (`840004`, `2405SvB`) with a
/// margin; narrower columns get the compact ones.
const FULL_LABELS_WIDTH: u16 = 9;

/// Colors of the original: blue bars, white text, blue heights. The heights
/// sit under the bars, so they stay readable.
const STYLE: ColumnStyle = ColumnStyle {
    bar: Color::Blue,
    text: Color::White,
    title: Color::Blue,
};

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>) {
    let Some(snapshot) = snapshot.filter(|s| s.connection_error().is_none()) else {
        frame.render_widget(panel_block(TITLE), area);
        return;
    };
    let size_on_disk = snapshot.blockchain.as_ref().ok().map(|b| b.size_on_disk);
    let blocks = match &snapshot.blocks {
        Ok(blocks) => blocks,
        Err(e) => {
            let error = Paragraph::new(e.to_string())
                .style(Style::default().fg(TextColor::Red))
                .block(panel_block(title(size_on_disk)));
            frame.render_widget(error, area);
            return;
        }
    };
    let block = panel_block(title(size_on_disk));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let block_columns = block_columns(blocks);
    let top = y_max(&block_columns);
    let band = inner.width / (block_columns.len().max(1) as u16);
    let compact = band < FULL_LABELS_WIDTH;
    let columns: Vec<Column> = block_columns
        .into_iter()
        .map(|c| Column {
            value: Some(c.size as f64),
            lines: c.lines(compact).to_vec(),
            title: c.label(compact),
        })
        .collect();
    let plot = column_plot(&columns, top, inner.height, STYLE);
    frame.render_widget(plot.widget(), inner);
}
