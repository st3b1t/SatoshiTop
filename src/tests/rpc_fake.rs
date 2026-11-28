//! Simulated Bitcoin node, ported from `test/rpc-fake.js`.
//!
//! Differences from the original: the chain tip is 840004, so the five blocks
//! shown by the Blocks panel are all in the fixtures; an unknown height or
//! method returns a JSON-RPC error instead of `{}` or `undefined`.

use std::sync::Mutex;

use serde_json::{Value, json};

use super::Rng;
use crate::bitcoin::rpc::{RpcBackend, RpcError};

const BLOCKCHAININFO: &str = include_str!("fixtures/getblockchaininfo.json");
const NETWORKINFO: &str = include_str!("fixtures/getnetworkinfo.json");
const MEMPOOLINFO: &str = include_str!("fixtures/getmempoolinfo.json");
const PEERINFO: &str = include_str!("fixtures/getpeerinfo.json");
const BLOCKSTATS: &str = include_str!("fixtures/getblockstats.json");

/// Uptime added at every `uptime` call, in seconds.
const UPTIME_STEP: u64 = 17;
/// Sync progress added at every `getblockchaininfo` call.
const SYNC_STEP: f64 = 0.037;
/// Chance of starting a new sync animation once synced.
const RESYNC_CHANCE: f64 = 0.1;

/// `estimatesmartfee` answers: three per target (1, 3, 6), picked with a
/// random shift as in the original.
const FEES: [(f64, u32); 9] = [
    (0.000443831, 2),
    (0.000344821, 3),
    (0.000244821, 6),
    (0.000543831, 2),
    (0.000444821, 3),
    (0.000383801, 6),
    (0.000553831, 2),
    (0.000454821, 3),
    (0.000413801, 6),
];

struct State {
    rng: Rng,
    uptime: u64,
    /// Sync progress; `>= 1` means synced.
    progress: f64,
}

/// A node that answers from fixtures, with a little randomness.
pub struct FakeRpc {
    state: Mutex<State>,
}

impl FakeRpc {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let uptime = rng.below(1_000_000);
        Self {
            state: Mutex::new(State {
                rng,
                uptime,
                progress: 0.0,
            }),
        }
    }

    fn answer(&self, method: &str, params: &[Value]) -> Result<Value, RpcError> {
        let mut state = self.state.lock().expect("fake rpc lock");
        match method {
            "uptime" => {
                state.uptime += UPTIME_STEP;
                Ok(json!(state.uptime))
            }
            "getblockchaininfo" => Ok(state.blockchaininfo()),
            "getnetworkinfo" => Ok(fixture(NETWORKINFO)),
            "getmempoolinfo" => Ok(fixture(MEMPOOLINFO)),
            "getpeerinfo" => Ok(state.peerinfo()),
            "estimatesmartfee" => {
                let target = params.first().and_then(as_u64).unwrap_or(1);
                Ok(state.smartfee(target))
            }
            "getblockstats" => {
                let height = params.first().and_then(as_u64);
                let stats = height.and_then(|h| fixture(BLOCKSTATS).get(h.to_string()).cloned());
                stats.ok_or_else(|| RpcError::Rpc {
                    code: -8,
                    message: "Target block height after current tip".into(),
                })
            }
            _ => Err(RpcError::Rpc {
                code: -32601,
                message: "Method not found".into(),
            }),
        }
    }
}

impl State {
    fn blockchaininfo(&mut self) -> Value {
        let mut info = fixture(BLOCKCHAININFO);
        self.progress += SYNC_STEP;
        if self.progress < 1.0 {
            info["verificationprogress"] = json!(self.progress);
            info["initialblockdownload"] = json!(true);
        } else {
            self.progress = if self.rng.next_f64() < RESYNC_CHANCE {
                0.0
            } else {
                1.0
            };
        }
        info
    }

    fn peerinfo(&mut self) -> Value {
        let Value::Array(mut peers) = fixture(PEERINFO) else {
            unreachable!("getpeerinfo fixture is an array");
        };
        // 10 or 11 peers, shuffled.
        peers.truncate(10 + self.rng.below(2) as usize);
        self.rng.shuffle(&mut peers);
        Value::Array(peers)
    }

    fn smartfee(&mut self, target: u64) -> Value {
        let shift = if self.rng.next_f64() > 0.5 { 3 } else { 0 };
        let index = (target + shift).saturating_sub(1) as usize;
        let (feerate, blocks) = FEES[index.min(FEES.len() - 1)];
        json!({ "feerate": feerate, "blocks": blocks })
    }
}

impl RpcBackend for FakeRpc {
    async fn call(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        self.answer(method, &params)
    }
}

fn fixture(text: &str) -> Value {
    serde_json::from_str(text).expect("fixtures are valid JSON")
}

/// A number, also when given as a string (like `satoprpc getblockstats "840000"`).
fn as_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::rpc::call_as;
    use crate::bitcoin::types::*;

    fn rpc() -> FakeRpc {
        FakeRpc::new(7)
    }

    #[tokio::test]
    async fn every_fixture_parses_into_its_type() {
        let rpc = rpc();
        let _: BlockchainInfo = call_as(&rpc, "getblockchaininfo", vec![]).await.unwrap();
        let network: NetworkInfo = call_as(&rpc, "getnetworkinfo", vec![]).await.unwrap();
        assert_eq!(network.subversion, "/Satoshi:22.0.0/");
        assert_eq!(network.localaddresses[0].port, 8333);
        let mempool: MempoolInfo = call_as(&rpc, "getmempoolinfo", vec![]).await.unwrap();
        assert_eq!(mempool.maxmempool, 300_000_000);
        let peers: Vec<PeerInfo> = call_as(&rpc, "getpeerinfo", vec![]).await.unwrap();
        assert!((10..=11).contains(&peers.len()));
        for height in 840_000..=840_004 {
            let stats: BlockStats = call_as(&rpc, "getblockstats", vec![json!(height)])
                .await
                .unwrap();
            assert_eq!(stats.height, height);
        }
    }

    #[tokio::test]
    async fn uptime_grows_at_every_call() {
        let rpc = rpc();
        let a: u64 = call_as(&rpc, "uptime", vec![]).await.unwrap();
        let b: u64 = call_as(&rpc, "uptime", vec![]).await.unwrap();
        assert_eq!(b, a + UPTIME_STEP);
    }

    #[tokio::test]
    async fn sync_is_animated_then_completes() {
        let rpc = rpc();
        let first: BlockchainInfo = call_as(&rpc, "getblockchaininfo", vec![]).await.unwrap();
        assert!(first.initialblockdownload);
        assert!((first.verificationprogress - SYNC_STEP).abs() < 1e-9);
        let mut synced = false;
        for _ in 0..30 {
            let info: BlockchainInfo = call_as(&rpc, "getblockchaininfo", vec![]).await.unwrap();
            synced |= !info.initialblockdownload;
        }
        assert!(synced);
    }

    #[tokio::test]
    async fn fees_come_from_the_table() {
        let rpc = rpc();
        for target in [1, 3, 6] {
            let fee: SmartFee = call_as(&rpc, "estimatesmartfee", vec![json!(target)])
                .await
                .unwrap();
            let feerate = fee.feerate.unwrap();
            assert!(FEES.iter().any(|&(f, b)| f == feerate && b == fee.blocks));
        }
    }

    #[tokio::test]
    async fn unknown_heights_and_methods_are_errors() {
        let rpc = rpc();
        let err = rpc.call("getblockstats", vec![json!(839_999)]).await;
        assert!(matches!(err, Err(RpcError::Rpc { code: -8, .. })));
        let err = rpc.call("stop", vec![]).await;
        assert!(matches!(err, Err(RpcError::Rpc { code: -32601, .. })));
    }

    #[tokio::test]
    async fn same_seed_same_answers() {
        let (a, b) = (FakeRpc::new(3), FakeRpc::new(3));
        for _ in 0..5 {
            assert_eq!(
                a.call("getpeerinfo", vec![]).await,
                b.call("getpeerinfo", vec![]).await
            );
        }
    }
}
