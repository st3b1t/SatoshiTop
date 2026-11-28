//! satoshitop: a Rust port of SatoshiTop, a terminal dashboard for a Bitcoin
//! full node, using malevich for charts and ratatui for the UI.
//!
//! Planned layout: `config`, `util`, `bitcoin`, `system`, `tests`, `ui`, `app`.

pub mod app;
pub mod bitcoin;
pub mod config;
pub mod system;
pub mod tests;
pub mod ui;
pub mod util;

#[cfg(test)]
mod test_util;

/// Program name, as in the original npm package.
pub const NAME: &str = "SatoshiTop";

/// Crate version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn version_comes_from_the_manifest() {
        assert_eq!(VERSION, "3.0.0");
        assert_eq!(NAME, "SatoshiTop");
    }
}
