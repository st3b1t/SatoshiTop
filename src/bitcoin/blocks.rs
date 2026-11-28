//! Last blocks, ported from `lib/bitcoin/blocks.js`.

use super::types::BlockStats;
use crate::util::human::{human_bytes, human_mins};

/// Lowest top of the y axis: 2 MiB, as in the original (`maxHeight`).
pub const MIN_Y_MAX: f64 = 2_097_152.0;

/// One column of the Blocks panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockColumn {
    pub height: u64,
    /// Block size in bytes.
    pub size: u64,
    /// Average fee rate in sat/vB.
    pub avgfeerate: u64,
    /// Seconds since the previous block (negative if timestamps disagree).
    pub interval: i64,
}

impl BlockColumn {
    /// The three text lines of the original: size, fee rate, time since the
    /// previous block. `compact` drops the fee rate unit, for narrow columns.
    pub fn lines(&self, compact: bool) -> [String; 3] {
        [
            human_bytes(self.size as f64),
            if compact {
                self.avgfeerate.to_string()
            } else {
                format!("{}SvB", self.avgfeerate)
            },
            human_mins(self.interval),
        ]
    }

    /// The height under the column; `compact` keeps its last four digits,
    /// enough to tell recent blocks apart (`0004` for 840004).
    pub fn label(&self, compact: bool) -> String {
        let height = self.height.to_string();
        if compact && height.len() > 4 {
            height[height.len() - 4..].to_owned()
        } else {
            height
        }
    }
}

/// Columns from blocks given newest first, oldest column first. The oldest
/// block is only used for the time to the next one, as in the original.
pub fn block_columns(newest_first: &[BlockStats]) -> Vec<BlockColumn> {
    let oldest_first: Vec<&BlockStats> = newest_first.iter().rev().collect();
    oldest_first
        .windows(2)
        .map(|pair| BlockColumn {
            height: pair[1].height,
            size: pair[1].total_size,
            avgfeerate: pair[1].avgfeerate,
            interval: pair[1].time - pair[0].time,
        })
        .collect()
}

/// Top of the y axis: the largest block, at least 2 MiB.
pub fn y_max(columns: &[BlockColumn]) -> f64 {
    columns
        .iter()
        .map(|c| c.size as f64)
        .fold(MIN_Y_MAX, f64::max)
}

/// `Blocks (tot 556.7GB)`: the size of the chain on disk.
pub fn title(size_on_disk: Option<u64>) -> String {
    match size_on_disk {
        Some(size) => format!("Blocks (tot {})", human_bytes(size as f64)),
        None => "Blocks".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(height: u64, time: i64, size: u64) -> BlockStats {
        BlockStats {
            height,
            time,
            total_size: size,
            avgfeerate: height % 1000,
        }
    }

    /// The fake node's blocks, newest first.
    fn fake() -> Vec<BlockStats> {
        vec![
            stats(840_004, 1_713_573_202, 1_420_877),
            stats(840_003, 1_713_572_633, 1_406_232),
            stats(840_002, 1_713_572_228, 1_425_969),
            stats(840_001, 1_713_571_854, 2_362_638),
            stats(840_000, 1_713_571_767, 2_325_218),
        ]
    }

    #[test]
    fn columns_go_from_oldest_to_newest() {
        let columns = block_columns(&fake());
        let heights: Vec<u64> = columns.iter().map(|c| c.height).collect();
        assert_eq!(heights, [840_001, 840_002, 840_003, 840_004]);
        let times: Vec<String> = columns.iter().map(|c| c.lines(false)[2].clone()).collect();
        assert_eq!(times, ["1m27s", "6m14s", "6m45s", "9m29s"]);
        assert_eq!(columns[0].lines(false), ["2.4MB", "1SvB", "1m27s"]);
        assert_eq!(columns[0].label(false), "840001");
    }

    #[test]
    fn compact_labels_for_narrow_columns() {
        let column = &block_columns(&fake())[3];
        assert_eq!(column.lines(true), ["1.4MB", "4", "9m29s"]);
        assert_eq!(column.label(true), "0004");
        let low = BlockColumn {
            height: 42,
            ..column.clone()
        };
        assert_eq!(low.label(true), "42");
    }

    #[test]
    fn the_axis_is_at_least_2_mib() {
        let columns = block_columns(&fake());
        assert_eq!(y_max(&columns), 2_362_638.0);
        assert_eq!(y_max(&columns[1..]), MIN_Y_MAX);
        assert_eq!(y_max(&[]), MIN_Y_MAX);
    }

    #[test]
    fn fewer_than_two_blocks_give_no_columns() {
        assert!(block_columns(&fake()[..1]).is_empty());
        assert!(block_columns(&[]).is_empty());
    }

    #[test]
    fn title_shows_the_chain_size() {
        assert_eq!(title(Some(556_672_030_542)), "Blocks (tot 556.7GB)");
        assert_eq!(title(None), "Blocks");
    }
}
