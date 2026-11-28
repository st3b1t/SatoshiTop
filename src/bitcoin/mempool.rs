//! Mempool, from `lib/bitcoin/mempool.js`. The original only sets the title
//! (`[work in progress]`); the usage bar and the details are new.

use super::types::MempoolInfo;
use crate::util::human::{human_bytes, virtual_bytes};

/// `Mempool ( 78.8MB / 300.0MB )`, as in the original.
pub fn title(mempool: &MempoolInfo) -> String {
    format!(
        "Mempool ( {} / {} )",
        human_bytes(mempool.bytes as f64),
        human_bytes(mempool.maxmempool as f64)
    )
}

/// Size of the mempool against its limit, in percent (0–100).
pub fn usage_percent(mempool: &MempoolInfo) -> f64 {
    if mempool.maxmempool == 0 {
        return 0.0;
    }
    (100.0 * mempool.bytes as f64 / mempool.maxmempool as f64).clamp(0.0, 100.0)
}

/// Text lines under the bar.
pub fn lines(mempool: &MempoolInfo) -> Vec<String> {
    let mut lines = vec![
        format!("Usage: {:.0}%", usage_percent(mempool)),
        format!("Transactions: {}", mempool.size),
        format!("Min fee: {} SvB", virtual_bytes(mempool.mempoolminfee)),
    ];
    if let Some(total) = mempool.total_fee {
        lines.push(format!("Total fees: {total:.8} BTC"));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fake node's mempool.
    fn fake() -> MempoolInfo {
        MempoolInfo {
            size: 70_853,
            bytes: 78_808_980,
            maxmempool: 300_000_000,
            mempoolminfee: 0.00007758,
            total_fee: Some(7.99827001),
        }
    }

    #[test]
    fn title_and_usage() {
        assert_eq!(title(&fake()), "Mempool ( 78.8MB / 300.0MB )");
        assert!((usage_percent(&fake()) - 26.27).abs() < 0.01);
    }

    #[test]
    fn detail_lines() {
        assert_eq!(
            lines(&fake()),
            [
                "Usage: 26%",
                "Transactions: 70853",
                "Min fee: 7 SvB",
                "Total fees: 7.99827001 BTC"
            ]
        );
        let older = MempoolInfo {
            total_fee: None,
            ..fake()
        };
        assert_eq!(lines(&older).len(), 3);
    }

    #[test]
    fn empty_limit() {
        let mempool = MempoolInfo {
            maxmempool: 0,
            ..fake()
        };
        assert_eq!(usage_percent(&mempool), 0.0);
    }
}
