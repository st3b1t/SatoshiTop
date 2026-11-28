//! Typed JSON-RPC results. Only the fields used by the panels are kept;
//! unknown fields are ignored, and fields missing in some Bitcoin Core
//! versions have defaults.

use serde::Deserialize;

/// `getblockchaininfo`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BlockchainInfo {
    #[serde(default)]
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    #[serde(default)]
    pub verificationprogress: f64,
    #[serde(default)]
    pub initialblockdownload: bool,
    #[serde(default)]
    pub size_on_disk: u64,
    #[serde(default)]
    pub mediantime: i64,
}

/// `getnetworkinfo`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NetworkInfo {
    #[serde(default)]
    pub subversion: String,
    #[serde(default)]
    pub localaddresses: Vec<LocalAddress>,
}

/// One entry of `getnetworkinfo.localaddresses` (e.g. the onion address).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocalAddress {
    pub address: String,
    pub port: u16,
}

/// One entry of `getpeerinfo`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PeerInfo {
    pub addr: String,
    #[serde(default)]
    pub subver: String,
    #[serde(default)]
    pub lastsend: i64,
    #[serde(default)]
    pub lastrecv: i64,
}

/// `getmempoolinfo`. Amounts are in BTC, fee rates in BTC/kvB.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MempoolInfo {
    pub size: u64,
    pub bytes: u64,
    pub maxmempool: u64,
    #[serde(default)]
    pub mempoolminfee: f64,
    /// Added in Bitcoin Core 0.21.
    #[serde(default)]
    pub total_fee: Option<f64>,
}

/// `estimatesmartfee`. The node omits `feerate` (and sets `errors`) when it
/// has not seen enough blocks to estimate.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SmartFee {
    /// Fee rate in BTC/kvB.
    #[serde(default)]
    pub feerate: Option<f64>,
    /// Block target the estimate is valid for.
    #[serde(default)]
    pub blocks: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// `getblockstats`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BlockStats {
    pub height: u64,
    pub time: i64,
    pub total_size: u64,
    /// Average fee rate in sat/vB.
    #[serde(default)]
    pub avgfeerate: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_fee_without_feerate() {
        let fee: SmartFee = serde_json::from_str(
            r#"{"errors":["Insufficient data or no feerate found"],"blocks":2}"#,
        )
        .unwrap();
        assert_eq!(fee.feerate, None);
        assert_eq!(fee.blocks, 2);
        assert_eq!(fee.errors.len(), 1);
    }

    #[test]
    fn mempool_info_from_older_nodes() {
        let info: MempoolInfo =
            serde_json::from_str(r#"{"size":1,"bytes":2,"maxmempool":3,"usage":4}"#).unwrap();
        assert_eq!(info.total_fee, None);
        assert_eq!(info.mempoolminfee, 0.0);
    }
}
