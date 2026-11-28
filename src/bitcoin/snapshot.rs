//! One consistent view of the node per refresh tick.
//!
//! The original lets every panel poll the node on its own timer, so the same
//! call runs several times per tick. Here one [`SnapshotFetcher::fetch`] makes
//! each call once, in parallel, and caches `getblockstats` by height.

use std::collections::BTreeMap;

use futures::future::{join_all, try_join3};
use serde_json::json;

use super::rpc::{RpcBackend, RpcError, call_as};
use super::types::{BlockStats, BlockchainInfo, MempoolInfo, NetworkInfo, PeerInfo, SmartFee};

/// Number of `getblockstats` fetched: the Blocks panel shows 4 blocks and uses
/// the 5th (oldest) one only to compute the time to the next.
pub const LAST_BLOCKS: u64 = 5;
/// Block targets passed to `estimatesmartfee`, in this order in
/// [`NodeSnapshot::fees`].
pub const FEE_TARGETS: [u64; 3] = [1, 3, 6];

/// Everything the Bitcoin panels show. Each field fails on its own, so one
/// failing call only affects its panel.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeSnapshot {
    pub blockchain: Result<BlockchainInfo, RpcError>,
    pub network: Result<NetworkInfo, RpcError>,
    /// Node uptime in seconds.
    pub uptime: Result<u64, RpcError>,
    pub peers: Result<Vec<PeerInfo>, RpcError>,
    pub mempool: Result<MempoolInfo, RpcError>,
    /// Estimates for [`FEE_TARGETS`].
    pub fees: Result<[SmartFee; 3], RpcError>,
    /// The last [`LAST_BLOCKS`] blocks, newest first.
    pub blocks: Result<Vec<BlockStats>, RpcError>,
}

impl NodeSnapshot {
    /// A snapshot where every field failed with `error`.
    pub fn failed(error: RpcError) -> Self {
        Self {
            blockchain: Err(error.clone()),
            network: Err(error.clone()),
            uptime: Err(error.clone()),
            peers: Err(error.clone()),
            mempool: Err(error.clone()),
            fees: Err(error.clone()),
            blocks: Err(error),
        }
    }

    /// The error that made the whole node unusable (connection, timeout,
    /// authentication), if any: shown as "Connection error" by the UI.
    pub fn connection_error(&self) -> Option<&RpcError> {
        self.blockchain
            .as_ref()
            .err()
            .filter(|e| e.is_connection_level())
    }
}

/// Fetches snapshots from a backend, keeping the block cache between ticks.
pub struct SnapshotFetcher<B> {
    backend: B,
    block_cache: BTreeMap<u64, BlockStats>,
}

impl<B: RpcBackend> SnapshotFetcher<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            block_cache: BTreeMap::new(),
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Fetches a full snapshot.
    ///
    /// `getblockchaininfo` goes first, since the block height is needed for
    /// `getblockstats`; if it fails at connection level, nothing else is tried.
    pub async fn fetch(&mut self) -> NodeSnapshot {
        let blockchain: Result<BlockchainInfo, RpcError> =
            call_as(&self.backend, "getblockchaininfo", vec![]).await;
        let tip = match &blockchain {
            Err(e) if e.is_connection_level() => return NodeSnapshot::failed(e.clone()),
            Ok(info) => Ok(info.blocks),
            Err(e) => Err(e.clone()),
        };

        let backend = &self.backend;
        let (network, uptime, peers, mempool, fees, blocks) = futures::join!(
            call_as::<NetworkInfo>(backend, "getnetworkinfo", vec![]),
            call_as::<u64>(backend, "uptime", vec![]),
            call_as::<Vec<PeerInfo>>(backend, "getpeerinfo", vec![]),
            call_as::<MempoolInfo>(backend, "getmempoolinfo", vec![]),
            fetch_fees(backend),
            fetch_blocks(backend, &self.block_cache, tip),
        );

        if let Ok(blocks) = &blocks {
            self.update_cache(blocks);
        }
        NodeSnapshot {
            blockchain,
            network,
            uptime,
            peers,
            mempool,
            fees,
            blocks,
        }
    }

    /// Keeps only the blocks just shown: older heights will not be asked again.
    fn update_cache(&mut self, blocks: &[BlockStats]) {
        self.block_cache = blocks.iter().map(|b| (b.height, b.clone())).collect();
    }
}

async fn fetch_fees(backend: &impl RpcBackend) -> Result<[SmartFee; 3], RpcError> {
    let [a, b, c] = FEE_TARGETS
        .map(|target| call_as::<SmartFee>(backend, "estimatesmartfee", vec![json!(target)]));
    let (a, b, c) = try_join3(a, b, c).await?;
    Ok([a, b, c])
}

/// The last [`LAST_BLOCKS`] blocks below `tip` (included), newest first,
/// asking the node only for heights missing from `cache`.
async fn fetch_blocks(
    backend: &impl RpcBackend,
    cache: &BTreeMap<u64, BlockStats>,
    tip: Result<u64, RpcError>,
) -> Result<Vec<BlockStats>, RpcError> {
    let tip = tip?;
    let heights: Vec<u64> = (0..LAST_BLOCKS)
        .filter_map(|i| tip.checked_sub(i))
        .collect();
    let fetched = join_all(heights.iter().map(|&height| async move {
        match cache.get(&height) {
            Some(stats) => Ok(stats.clone()),
            None => call_as(backend, "getblockstats", vec![json!(height)]).await,
        }
    }))
    .await;
    fetched.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serde_json::Value;

    use super::*;
    use crate::tests::rpc_fake::FakeRpc;

    /// Wraps a backend, recording every call.
    struct Recording<B> {
        inner: B,
        calls: Mutex<Vec<String>>,
        /// When set, every call fails with this error.
        fail: Option<RpcError>,
    }

    impl<B> Recording<B> {
        fn new(inner: B) -> Self {
            Self {
                inner,
                calls: Mutex::new(Vec::new()),
                fail: None,
            }
        }

        fn take_calls(&self) -> Vec<String> {
            let mut calls = std::mem::take(&mut *self.calls.lock().unwrap());
            calls.sort();
            calls
        }
    }

    impl<B: RpcBackend> RpcBackend for Recording<B> {
        async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
            let label = match params.first() {
                Some(p) => format!("{method} {p}"),
                None => method.to_owned(),
            };
            self.calls.lock().unwrap().push(label);
            if let Some(e) = &self.fail {
                return Err(e.clone());
            }
            self.inner.call(method, params).await
        }
    }

    #[tokio::test]
    async fn a_snapshot_from_the_fake_node_is_complete() {
        let mut fetcher = SnapshotFetcher::new(FakeRpc::new(1));
        let snapshot = fetcher.fetch().await;
        assert_eq!(snapshot.connection_error(), None);
        assert_eq!(snapshot.blockchain.unwrap().blocks, 840_004);
        assert!(snapshot.network.is_ok());
        assert!(snapshot.uptime.is_ok());
        assert!(snapshot.peers.is_ok());
        assert!(snapshot.mempool.is_ok());
        assert!(snapshot.fees.is_ok());
        let heights: Vec<u64> = snapshot.blocks.unwrap().iter().map(|b| b.height).collect();
        assert_eq!(heights, [840_004, 840_003, 840_002, 840_001, 840_000]);
    }

    #[tokio::test]
    async fn every_call_runs_once_and_blocks_are_cached() {
        let mut fetcher = SnapshotFetcher::new(Recording::new(FakeRpc::new(1)));
        fetcher.fetch().await;
        assert_eq!(
            fetcher.backend().take_calls(),
            [
                "estimatesmartfee 1",
                "estimatesmartfee 3",
                "estimatesmartfee 6",
                "getblockchaininfo",
                "getblockstats 840000",
                "getblockstats 840001",
                "getblockstats 840002",
                "getblockstats 840003",
                "getblockstats 840004",
                "getmempoolinfo",
                "getnetworkinfo",
                "getpeerinfo",
                "uptime",
            ]
        );

        let second = fetcher.fetch().await;
        assert_eq!(second.blocks.unwrap().len(), LAST_BLOCKS as usize);
        let calls = fetcher.backend().take_calls();
        assert!(
            !calls.iter().any(|c| c.starts_with("getblockstats")),
            "{calls:?}"
        );
    }

    #[tokio::test]
    async fn a_connection_error_stops_the_snapshot() {
        let mut backend = Recording::new(FakeRpc::new(1));
        backend.fail = Some(RpcError::Connection("Connection refused".into()));
        let mut fetcher = SnapshotFetcher::new(backend);
        let snapshot = fetcher.fetch().await;
        assert_eq!(
            snapshot.connection_error(),
            Some(&RpcError::Connection("Connection refused".into()))
        );
        assert!(snapshot.mempool.is_err());
        assert_eq!(fetcher.backend().take_calls(), ["getblockchaininfo"]);
    }

    #[tokio::test]
    async fn other_errors_only_affect_their_field() {
        /// Fails `estimatesmartfee` only.
        struct NoFees(FakeRpc);
        impl RpcBackend for NoFees {
            async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
                if method == "estimatesmartfee" {
                    return Err(RpcError::Rpc {
                        code: -32603,
                        message: "fee estimation disabled".into(),
                    });
                }
                self.0.call(method, params).await
            }
        }
        let snapshot = SnapshotFetcher::new(NoFees(FakeRpc::new(1))).fetch().await;
        assert_eq!(snapshot.connection_error(), None);
        assert!(snapshot.fees.is_err());
        assert!(snapshot.blocks.is_ok());
        assert!(snapshot.mempool.is_ok());
    }

    #[test]
    fn failed_snapshot() {
        let snapshot = NodeSnapshot::failed(RpcError::Timeout);
        assert_eq!(snapshot.connection_error(), Some(&RpcError::Timeout));
        assert_eq!(snapshot.uptime, Err(RpcError::Timeout));
    }
}
