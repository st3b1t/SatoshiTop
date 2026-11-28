//! Fees panel, ported from `lib/bitcoin/fees.js`: four yellow columns,
//! taller for higher fees (the original draws them all at the same height).

use malevich::Color;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color as TextColor, Style};
use ratatui::widgets::Paragraph;

use super::columns::{Column, ColumnStyle, column_plot};
use crate::bitcoin::fees::fee_columns;
use crate::bitcoin::snapshot::NodeSnapshot;
use crate::ui::panel_block;

/// Title before the first data; then [`DATA_TITLE`], as in the original.
pub const TITLE: &str = "Fees";
pub const DATA_TITLE: &str = "Transaction Fees";

const STYLE: ColumnStyle = ColumnStyle {
    bar: Color::Yellow,
    text: Color::Black,
    title: Color::Yellow,
};

pub fn render(frame: &mut Frame, area: Rect, snapshot: Option<&NodeSnapshot>) {
    let Some(snapshot) = snapshot.filter(|s| s.connection_error().is_none()) else {
        frame.render_widget(panel_block(TITLE), area);
        return;
    };
    let fees = match &snapshot.fees {
        Ok(fees) => fees,
        Err(e) => {
            let error = Paragraph::new(e.to_string())
                .style(Style::default().fg(TextColor::Red))
                .block(panel_block(TITLE));
            frame.render_widget(error, area);
            return;
        }
    };
    let block = panel_block(DATA_TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let columns: Vec<Column> = fee_columns(snapshot.mempool.as_ref().ok(), fees)
        .into_iter()
        .map(|c| Column {
            value: c.sat_vb.map(|fee| fee as f64),
            lines: vec![c.label()],
            title: c.title,
        })
        .collect();
    let y_max = columns.iter().filter_map(|c| c.value).fold(1.0, f64::max);
    let plot = column_plot(&columns, y_max, inner.height, STYLE);
    frame.render_widget(plot.widget(), inner);
}
