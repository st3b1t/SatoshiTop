//! Column chart with text, the shared shape of the Fees and Blocks panels
//! (the custom `block` widget of the original): one bar per column, text
//! lines inside the bar from its foot, and a title under it.
//!
//! malevich draws it: bars on a bands axis without axes, `Text` marks
//! centered in each band. Text over a bar takes the bar's color as its
//! background. The y domain reserves one row below zero for the titles, and
//! one row of data per text line is computed from the height of the area, so
//! lines land on exact rows.

use malevich::{Align, Bars, Color, Plot, Text};

/// Colors of a column chart.
#[derive(Debug, Clone, Copy)]
pub struct ColumnStyle {
    pub bar: Color,
    pub text: Color,
    pub title: Color,
}

/// One column: bar height in data units, text lines (top first), title.
#[derive(Debug, Clone)]
pub struct Column {
    /// Bar height; `None` draws no bar (the text is still shown).
    pub value: Option<f64>,
    pub lines: Vec<String>,
    pub title: String,
}

/// The chart for an area `rows` tall, with the y axis from 0 to `y_max`.
pub fn column_plot(columns: &[Column], y_max: f64, rows: u16, style: ColumnStyle) -> Plot<'static> {
    // One row for the titles below zero; the others span 0..y_max.
    let data_rows = f64::from(rows.max(2) - 1);
    let unit = y_max.max(f64::MIN_POSITIVE) / data_rows;
    let row = |index: f64| unit * (index + 0.5);

    // Bands need distinct names; they are not shown (no axes).
    let bands: Vec<String> = (0..columns.len()).map(|i| i.to_string()).collect();
    let values: Vec<f64> = columns
        .iter()
        .map(|c| c.value.unwrap_or(f64::NAN))
        .collect();
    let mut plot = Plot::new().layer(Bars::new(bands, values).color(style.bar));
    for (i, column) in columns.iter().enumerate() {
        let x = i as f64;
        let count = column.lines.len();
        for (j, line) in column.lines.iter().enumerate() {
            let text = Text::at(x, row((count - 1 - j) as f64), line.clone());
            plot = plot.layer(text.color(style.text).align(Align::Center));
        }
        let title = Text::at(x, row(-1.0), column.title.clone());
        plot = plot.layer(title.color(style.title).align(Align::Center));
    }
    plot.y_domain(-unit, y_max).axes(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STYLE: ColumnStyle = ColumnStyle {
        bar: Color::Blue,
        text: Color::White,
        title: Color::Cyan,
    };

    fn column(value: f64, lines: &[&str], title: &str) -> Column {
        Column {
            value: Some(value),
            lines: lines.iter().map(|l| l.to_string()).collect(),
            title: title.into(),
        }
    }

    #[test]
    fn text_sits_on_exact_rows() {
        let columns = [
            column(10.0, &["a1", "b1"], "t1"),
            column(5.0, &["a2", "b2"], "t2"),
        ];
        let rendered = column_plot(&columns, 10.0, 6, STYLE).render(&malevich::Frame::plain(20, 6));
        let rows: Vec<&str> = rendered.lines().collect();
        assert_eq!(rows.len(), 6, "{rendered}");
        assert!(
            rows[5].contains("t1") && rows[5].contains("t2"),
            "{rendered}"
        );
        assert!(
            rows[4].contains("b1") && rows[4].contains("b2"),
            "{rendered}"
        );
        assert!(
            rows[3].contains("a1") && rows[3].contains("a2"),
            "{rendered}"
        );
    }

    #[test]
    fn missing_values_and_tiny_areas() {
        let columns = [Column {
            value: None,
            lines: vec!["-- SvB".into()],
            title: "Slow".into(),
        }];
        for rows in [1, 2, 3] {
            column_plot(&columns, 1.0, rows, STYLE)
                .render(&malevich::Frame::plain(10, rows.into()));
        }
        column_plot(&[], 1.0, 5, STYLE).render(&malevich::Frame::plain(10, 5));
    }
}
