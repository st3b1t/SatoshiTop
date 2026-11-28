//! Disks panel, ported from `lib/system/disk.js`. The original donut becomes
//! one horizontal bar per mount point on a 0–100% axis.

use malevich::{Bars, Plot};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::system::DiskSample;
use crate::system::history::SysHistory;
use crate::ui::panel_block;
use crate::util::COLORS;

pub const TITLE: &str = "Disks";

/// The disks shown: boot partitions are hidden, as in the original.
pub fn visible(disks: &[DiskSample]) -> Vec<&DiskSample> {
    disks
        .iter()
        .filter(|d| !d.mount.contains("/boot"))
        .collect()
}

/// Bar label: percent first, so it survives when malevich shortens long
/// mount points (`98% /media/Block…`).
pub fn label(disk: &DiskSample) -> String {
    format!("{:.0}% {}", disk.use_percent(), disk.mount)
}

/// One bar per disk, each in its palette color (one layer per bar: `color_by`
/// would use another palette and add a legend). `None` without disks.
pub fn disks_plot(disks: &[DiskSample]) -> Option<Plot<'static>> {
    let disks = visible(disks);
    if disks.is_empty() {
        return None;
    }
    let labels: Vec<String> = disks.iter().map(|d| label(d)).collect();
    let plot = disks
        .iter()
        .enumerate()
        .fold(Plot::new(), |plot, (i, disk)| {
            let mut values = vec![f64::NAN; disks.len()];
            values[i] = disk.use_percent();
            plot.layer(
                Bars::new(labels.clone(), values)
                    .horizontal()
                    .color(COLORS[i % COLORS.len()]),
            )
        });
    Some(plot.x_domain(0.0, 100.0))
}

pub fn render(frame: &mut Frame, area: Rect, history: &SysHistory) {
    let block = panel_block(TITLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if let Some(plot) = disks_plot(&history.disks)
        && !inner.is_empty()
    {
        frame.render_widget(plot.widget(), inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk(mount: &str, percent: u64) -> DiskSample {
        DiskSample {
            fs: format!("/dev/{}", mount.len()),
            mount: mount.into(),
            total: 100,
            used: percent,
        }
    }

    #[test]
    fn boot_partitions_are_hidden() {
        let disks = [
            disk("/", 98),
            disk("/boot/efi", 17),
            disk("/boot", 30),
            disk("/data", 26),
        ];
        let mounts: Vec<&str> = visible(&disks).iter().map(|d| d.mount.as_str()).collect();
        assert_eq!(mounts, ["/", "/data"]);
    }

    #[test]
    fn labels_put_the_percent_first() {
        assert_eq!(
            label(&disk("/media/Blockchain", 26)),
            "26% /media/Blockchain"
        );
    }

    #[test]
    fn every_visible_disk_gets_a_bar() {
        let disks = [disk("/", 98), disk("/boot/efi", 17), disk("/data", 26)];
        let rendered = disks_plot(&disks)
            .unwrap()
            .render(&malevich::Frame::plain(50, 8));
        assert!(rendered.contains("98% /"), "{rendered}");
        assert!(rendered.contains("26% /data"), "{rendered}");
        assert!(!rendered.contains("efi"), "{rendered}");
        assert!(rendered.contains("100"), "{rendered}");
    }

    #[test]
    fn no_disks_no_plot() {
        assert!(disks_plot(&[]).is_none());
        assert!(disks_plot(&[disk("/boot", 10)]).is_none());
    }
}
