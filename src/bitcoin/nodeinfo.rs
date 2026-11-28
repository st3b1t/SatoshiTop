//! Node info, ported from `lib/bitcoin/nodeinfo.js` and the `status` helper
//! of `lib/bitcoin/rpc.js`.

use super::snapshot::NodeSnapshot;
use super::types::BlockchainInfo;
use crate::util::human::{human_secs, human_subver};

/// Synchronization state of the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Still downloading or verifying blocks; progress in whole percent.
    Syncing(u8),
    Synched,
}

impl NodeStatus {
    /// `Syncing` while in initial block download or behind the known headers.
    pub fn of(info: &BlockchainInfo) -> Self {
        if info.initialblockdownload || info.headers > info.blocks {
            let percent = (info.verificationprogress * 100.0)
                .floor()
                .clamp(0.0, 100.0);
            Self::Syncing(percent as u8)
        } else {
            Self::Synched
        }
    }
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Texts of the original.
            Self::Syncing(percent) => write!(f, "Syncing {percent}%"),
            Self::Synched => f.write_str("Synched"),
        }
    }
}

/// One line of the panel: label and value, `None` when unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLine {
    pub label: &'static str,
    pub value: Option<String>,
    /// Set on the status line only, to color it.
    pub status: Option<NodeStatus>,
}

/// The four lines of the original: Blocks, Status, UpTime, UserAgent. A field
/// that failed, or is zero, is unknown, as in the original.
pub fn node_lines(snapshot: &NodeSnapshot) -> [NodeLine; 4] {
    let blockchain = snapshot.blockchain.as_ref().ok();
    let status = blockchain.map(NodeStatus::of);
    let line = |label, value: Option<String>| NodeLine {
        label,
        value,
        status: None,
    };
    [
        line(
            "Blocks",
            blockchain
                .filter(|b| b.blocks > 0)
                .map(|b| b.blocks.to_string()),
        ),
        NodeLine {
            label: "Status",
            value: status.map(|s| s.to_string()),
            status,
        },
        line(
            "UpTime",
            snapshot
                .uptime
                .as_ref()
                .ok()
                .map(|&secs| human_secs(i64::try_from(secs).unwrap_or(i64::MAX), false))
                .filter(|s| !s.is_empty()),
        ),
        line(
            "UserAgent",
            snapshot
                .network
                .as_ref()
                .ok()
                .map(|n| human_subver(&n.subversion))
                .filter(|s| !s.is_empty()),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::rpc::RpcError;
    use crate::bitcoin::types::NetworkInfo;

    fn info(blocks: u64, headers: u64, ibd: bool, progress: f64) -> BlockchainInfo {
        BlockchainInfo {
            chain: "main".into(),
            blocks,
            headers,
            verificationprogress: progress,
            initialblockdownload: ibd,
            size_on_disk: 0,
            mediantime: 0,
        }
    }

    #[test]
    fn status_follows_the_original() {
        assert_eq!(
            NodeStatus::of(&info(10, 10, true, 0.037)),
            NodeStatus::Syncing(3)
        );
        assert_eq!(
            NodeStatus::of(&info(10, 12, false, 0.999)),
            NodeStatus::Syncing(99)
        );
        assert_eq!(
            NodeStatus::of(&info(10, 10, false, 0.9999)),
            NodeStatus::Synched
        );
        assert_eq!(NodeStatus::Syncing(3).to_string(), "Syncing 3%");
        assert_eq!(NodeStatus::Synched.to_string(), "Synched");
    }

    fn snapshot() -> NodeSnapshot {
        let mut snapshot = NodeSnapshot::failed(RpcError::Timeout);
        snapshot.blockchain = Ok(info(840_004, 840_004, false, 1.0));
        snapshot.uptime = Ok(90_061);
        snapshot.network = Ok(NetworkInfo {
            subversion: "/Satoshi:27.0.0/".into(),
            localaddresses: vec![],
        });
        snapshot
    }

    #[test]
    fn lines_with_every_value() {
        let lines = node_lines(&snapshot());
        let values: Vec<Option<&str>> = lines.iter().map(|l| l.value.as_deref()).collect();
        assert_eq!(
            values,
            [
                Some("840004"),
                Some("Synched"),
                Some("1D 1h 1m 1s"),
                Some("Satoshi:27.0.0")
            ]
        );
        assert_eq!(lines[1].status, Some(NodeStatus::Synched));
        let labels: Vec<&str> = lines.iter().map(|l| l.label).collect();
        assert_eq!(labels, ["Blocks", "Status", "UpTime", "UserAgent"]);
    }

    #[test]
    fn failed_or_zero_fields_are_unknown() {
        let mut snapshot = snapshot();
        snapshot.uptime = Err(RpcError::Timeout);
        snapshot.blockchain = Ok(info(0, 0, false, 0.0));
        let lines = node_lines(&snapshot);
        assert_eq!(lines[0].value, None, "zero blocks");
        assert_eq!(lines[2].value, None, "failed uptime");
        assert!(
            node_lines(&NodeSnapshot::failed(RpcError::Timeout))
                .iter()
                .all(|l| l.value.is_none())
        );
    }
}
