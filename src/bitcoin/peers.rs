//! Peer list, ported from `lib/bitcoin/peers.js`.

use super::types::PeerInfo;
use crate::util::human::human_subver;
use crate::util::truncate;

/// Panel title: `Peers (N connected)`.
pub fn title(peers: &[PeerInfo]) -> String {
    format!("Peers ({} connected)", peers.len())
}

/// Table rows `[user agent, address]`, most recently active peer first (by
/// the later of last send and last receive), addresses shortened to
/// `address_width` characters.
pub fn peer_rows(peers: &[PeerInfo], address_width: usize) -> Vec<[String; 2]> {
    let mut sorted: Vec<&PeerInfo> = peers.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.lastsend.max(p.lastrecv)));
    sorted
        .into_iter()
        .map(|p| [human_subver(&p.subver), truncate(&p.addr, address_width)])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(addr: &str, subver: &str, lastsend: i64, lastrecv: i64) -> PeerInfo {
        PeerInfo {
            addr: addr.into(),
            subver: subver.into(),
            lastsend,
            lastrecv,
        }
    }

    #[test]
    fn most_recent_activity_first() {
        let peers = [
            peer("1.1.1.1:8333", "/Satoshi:26.0.0/", 100, 50),
            peer("2.2.2.2:8333", "/Satoshi:27.0.0/", 10, 300),
            peer("3.3.3.3:8333", "/btcd:0.24.0/", 200, 200),
        ];
        let rows = peer_rows(&peers, 30);
        let addresses: Vec<&str> = rows.iter().map(|r| r[1].as_str()).collect();
        assert_eq!(addresses, ["2.2.2.2:8333", "3.3.3.3:8333", "1.1.1.1:8333"]);
        assert_eq!(rows[0][0], "Satoshi:27.0.0");
    }

    #[test]
    fn addresses_are_truncated_to_the_column() {
        let onion = "satoshitopm72cepqoa2l54adxtg3qwdd7ci6oddzcco5yjqd7mrdwqk.onion:8333";
        let rows = peer_rows(&[peer(onion, "", 0, 0)], 13);
        assert_eq!(rows[0][1], "satoshitop...");
    }

    #[test]
    fn title_counts_the_peers() {
        assert_eq!(title(&[]), "Peers (0 connected)");
        assert_eq!(
            title(&[peer("a", "", 0, 0), peer("b", "", 0, 0)]),
            "Peers (2 connected)"
        );
    }
}
