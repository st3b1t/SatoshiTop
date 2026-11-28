//! Tor address, ported from `lib/bitcoin/tor.js`.

use super::types::NetworkInfo;

/// The node's reachable address as `address:port`: the onion address if the
/// node announces one (the original takes the first local address, which may
/// be IPv4), `unknown:unknown` when there is none.
pub fn tor_address(network: &NetworkInfo) -> String {
    let addresses = &network.localaddresses;
    addresses
        .iter()
        .find(|a| a.address.ends_with(".onion"))
        .or_else(|| addresses.first())
        .map_or_else(
            || "unknown:unknown".to_owned(),
            |a| format!("{}:{}", a.address, a.port),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::types::LocalAddress;

    fn network(addresses: &[(&str, u16)]) -> NetworkInfo {
        NetworkInfo {
            subversion: String::new(),
            localaddresses: addresses
                .iter()
                .map(|&(address, port)| LocalAddress {
                    address: address.into(),
                    port,
                })
                .collect(),
        }
    }

    #[test]
    fn the_onion_address_is_preferred() {
        let net = network(&[("203.0.113.7", 8333), ("abc.onion", 8333)]);
        assert_eq!(tor_address(&net), "abc.onion:8333");
    }

    #[test]
    fn otherwise_the_first_address() {
        assert_eq!(
            tor_address(&network(&[("203.0.113.7", 8333)])),
            "203.0.113.7:8333"
        );
    }

    #[test]
    fn no_address() {
        assert_eq!(tor_address(&network(&[])), "unknown:unknown");
    }
}
