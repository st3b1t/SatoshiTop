//! Fee estimates, ported from `lib/bitcoin/fees.js`.

use super::snapshot::FEE_TARGETS;
use super::types::{MempoolInfo, SmartFee};
use crate::util::human::{human_blocks, virtual_bytes};

/// One column of the Fees panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeeColumn {
    /// `Slow`, or the time to confirm (`1hour`, `20min`).
    pub title: String,
    /// Fee rate in sat/vB; `None` when the node has no estimate.
    pub sat_vb: Option<i64>,
}

impl FeeColumn {
    /// `15 SvB`, or `-- SvB` without an estimate (`NaN SvB` in the original).
    pub fn label(&self) -> String {
        match self.sat_vb {
            Some(fee) => format!("{fee} SvB"),
            None => "-- SvB".to_owned(),
        }
    }
}

/// The four columns of the original: slow, then the estimates for 6, 3 and
/// 1 blocks. `fees` are the estimates for [`FEE_TARGETS`] (1, 3, 6).
///
/// Slow is `min(2 * mempoolminfee, estimate for 6 blocks)`; without a 6-block
/// estimate it is `2 * mempoolminfee`, without mempool info the estimate.
pub fn fee_columns(mempool: Option<&MempoolInfo>, fees: &[SmartFee; 3]) -> [FeeColumn; 4] {
    let [one, three, six] = fees;
    let min_fee = mempool.map(|m| 2.0 * m.mempoolminfee);
    let slow = match (min_fee, six.feerate) {
        (Some(min), Some(six)) => Some(min.min(six)),
        (min, six) => min.or(six),
    };
    let column = |fee: &SmartFee, target: u64| FeeColumn {
        // The node answers with the target it could estimate for; fall back
        // to the requested one when it does not say.
        title: human_blocks(
            if fee.blocks > 0 {
                i64::from(fee.blocks)
            } else {
                target as i64
            },
            true,
        ),
        sat_vb: fee.feerate.map(virtual_bytes),
    };
    [
        FeeColumn {
            title: "Slow".to_owned(),
            sat_vb: slow.map(virtual_bytes),
        },
        column(six, FEE_TARGETS[2]),
        column(three, FEE_TARGETS[1]),
        column(one, FEE_TARGETS[0]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fee(feerate: Option<f64>, blocks: u32) -> SmartFee {
        SmartFee {
            feerate,
            blocks,
            errors: vec![],
        }
    }

    fn mempool(minfee: f64) -> MempoolInfo {
        MempoolInfo {
            size: 1,
            bytes: 1,
            maxmempool: 1,
            mempoolminfee: minfee,
            total_fee: None,
        }
    }

    #[test]
    fn columns_follow_the_original() {
        let fees = [
            fee(Some(0.000443831), 2),
            fee(Some(0.000344821), 3),
            fee(Some(0.000244821), 6),
        ];
        let columns = fee_columns(Some(&mempool(0.00007758)), &fees);
        let shown: Vec<(String, String)> = columns
            .iter()
            .map(|c| (c.title.clone(), c.label()))
            .collect();
        assert_eq!(
            shown,
            [
                ("Slow".to_owned(), "15 SvB".to_owned()),
                ("1hour".to_owned(), "24 SvB".to_owned()),
                ("30min".to_owned(), "34 SvB".to_owned()),
                ("20min".to_owned(), "44 SvB".to_owned()),
            ]
        );
    }

    #[test]
    fn slow_takes_the_lower_of_the_two() {
        let fees = [
            fee(Some(0.0003), 1),
            fee(Some(0.0002), 3),
            fee(Some(0.00001), 6),
        ];
        let columns = fee_columns(Some(&mempool(0.00007758)), &fees);
        assert_eq!(columns[0].sat_vb, Some(1));
    }

    #[test]
    fn missing_estimates_are_shown_as_dashes() {
        let fees = [fee(None, 0), fee(None, 0), fee(None, 0)];
        let columns = fee_columns(Some(&mempool(0.00001)), &fees);
        assert_eq!(columns[0].sat_vb, Some(2), "slow from the mempool alone");
        assert_eq!(columns[1].label(), "-- SvB");
        assert_eq!(columns[1].title, "1hour", "requested target");
        assert_eq!(columns[3].title, "10min");
        let columns = fee_columns(None, &fees);
        assert_eq!(columns[0].label(), "-- SvB");
    }
}
