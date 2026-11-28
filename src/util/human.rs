//! Human-readable formatting, ported from `lib/utils/human.js` and
//! `lib/utils/fees.js`.
//!
//! Output formats match the JavaScript original, quirks included. The one known
//! difference is
//! rounding of exact ties in [`human_bytes`]: Rust rounds half to even, so
//! `1250` is `1.2KB` here and `1.3KB` in JavaScript.

const SECS_PER_YEAR: i64 = 31_536_000;
/// Average month length used by the original (30.44 days).
const SECS_PER_MONTH: i64 = 2_629_744;
const SECS_PER_DAY: i64 = 86_400;
const SECS_PER_HOUR: i64 = 3_600;
const SECS_PER_MIN: i64 = 60;
/// Target block interval of the Bitcoin network, in seconds.
const SECS_PER_BLOCK: i64 = 600;
/// Unit prefixes indexed by exponent; index 0 is a space, as in the original.
const PREFIXES: [char; 6] = [' ', 'K', 'M', 'G', 'T', 'P'];

/// Strips every `/` from a node user agent: `/Satoshi:27.0.0/` → `Satoshi:27.0.0`.
pub fn human_subver(subver: &str) -> String {
    subver.replace('/', "")
}

/// Formats a byte count with decimal prefixes (powers of 1000): `1.5KB`, `123.5MB`.
pub fn human_bytes(bytes: f64) -> String {
    format_bytes(bytes, false)
}

/// Formats a byte count with binary prefixes (powers of 1024): `1.5KiB`, `117.7MiB`.
pub fn human_bytes_binary(bytes: f64) -> String {
    format_bytes(bytes, true)
}

fn format_bytes(bytes: f64, binary: bool) -> String {
    if bytes == 0.0 {
        return "0.00 B".to_owned();
    }
    let base: f64 = if binary { 1024.0 } else { 1000.0 };
    // Same formula as the original, so the unit boundaries round the same way.
    let exp = (bytes.ln() / base.ln()).floor();
    let value = bytes / base.powf(exp);
    // `' KMGTP'.charAt(e)` yields an empty string outside the table.
    let prefix = usize::try_from(exp as i64)
        .ok()
        .and_then(|i| PREFIXES.get(i))
        .map(char::to_string)
        .unwrap_or_default();
    let infix = if binary && exp != 0.0 { "i" } else { "" };
    format!("{value:.1}{prefix}{infix}B")
}

/// Formats a duration in seconds as `1D 2h 3m 4s` (or `1day 2hour 3min 4sec`
/// when `long`). Zero or negative durations give an empty string.
///
/// Like the original, months are not taken modulo the year and days are taken
/// modulo the year only: 400 days is `1Y 13M 35D`.
pub fn human_secs(secs: i64, long: bool) -> String {
    let rem_year = secs % SECS_PER_YEAR;
    let rem_day = rem_year % SECS_PER_DAY;
    let parts = [
        (secs / SECS_PER_YEAR, "Y", "year"),
        (secs / SECS_PER_MONTH, "M", "month"),
        (rem_year / SECS_PER_DAY, "D", "day"),
        (rem_day / SECS_PER_HOUR, "h", "hour"),
        (rem_day % SECS_PER_HOUR / SECS_PER_MIN, "m", "min"),
        (rem_day % SECS_PER_HOUR % SECS_PER_MIN, "s", "sec"),
    ];
    // Truncating division is enough: for negative input every part is <= 0
    // and gets skipped, exactly as with `Math.floor` in the original.
    parts
        .iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|(n, short, long_unit)| format!("{n}{}", if long { long_unit } else { short }))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Formats seconds as `Nm` or `NmSs`: `602` → `10m2s`.
///
/// Like the original, a remainder of exactly one second is hidden (`61` → `1m`).
pub fn human_mins(secs: i64) -> String {
    // `Math.floor` for minutes, truncating `%` for the remainder, as in JS.
    let mins = secs.div_euclid(SECS_PER_MIN);
    let rem = secs % SECS_PER_MIN;
    if rem > 1 {
        format!("{mins}m{rem}s")
    } else {
        format!("{mins}m")
    }
}

/// Formats a number of blocks as the expected time to mine them (10 minutes
/// each): `6` → `1h`.
pub fn human_blocks(blocks: i64, long: bool) -> String {
    human_secs(blocks.saturating_mul(SECS_PER_BLOCK), long)
}

/// Converts a fee rate from BTC/kvB (as returned by the node) to sat/vB,
/// rounding down.
pub fn virtual_bytes(btc_per_kvb: f64) -> i64 {
    (btc_per_kvb * 100_000_000.0 / 1000.0).floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values generated with `node -e` from vendor/SatoshiTop/lib/utils.

    #[test]
    fn human_subver_strips_slashes() {
        assert_eq!(human_subver("/Satoshi:27.0.0/"), "Satoshi:27.0.0");
        assert_eq!(
            human_subver("/Satoshi:0.21.0/Knots:20210130/"),
            "Satoshi:0.21.0Knots:20210130"
        );
        assert_eq!(human_subver("unknown"), "unknown");
    }

    #[test]
    fn human_bytes_matches_js() {
        let cases = [
            (0.0, "0.00 B"),
            (1.0, "1.0 B"),
            (512.0, "512.0 B"),
            (999.0, "999.0 B"),
            (1000.0, "1.0KB"),
            (1023.0, "1.0KB"),
            (1024.0, "1.0KB"),
            (1500.0, "1.5KB"),
            (999_949.0, "999.9KB"),
            (999_950.0, "1000.0KB"),
            (1e6, "1.0MB"),
            (1_048_576.0, "1.0MB"),
            (1_234_567.0, "1.2MB"),
            (123_456_789.0, "123.5MB"),
            (1e9, "1.0GB"),
            (2e9, "2.0GB"),
            (5.5e12, "5.5TB"),
            (1e15, "1.0PB"),
            (3.3e15, "3.3PB"),
            (1e18, "1.0B"),
            (0.5, "500.0B"),
        ];
        for (bytes, expected) in cases {
            assert_eq!(human_bytes(bytes), expected, "human_bytes({bytes})");
        }
    }

    #[test]
    fn human_bytes_binary_matches_js() {
        let cases = [
            (0.0, "0.00 B"),
            (1.0, "1.0 B"),
            (1000.0, "1000.0 B"),
            (1023.0, "1023.0 B"),
            (1024.0, "1.0KiB"),
            (1500.0, "1.5KiB"),
            (999_949.0, "976.5KiB"),
            (1e6, "976.6KiB"),
            (1_048_576.0, "1.0MiB"),
            (123_456_789.0, "117.7MiB"),
            (1e9, "953.7MiB"),
            (2e9, "1.9GiB"),
            (5.5e12, "5.0TiB"),
            (1e15, "909.5TiB"),
            (3.3e15, "2.9PiB"),
            (1e18, "888.2PiB"),
            (0.5, "512.0iB"),
        ];
        for (bytes, expected) in cases {
            assert_eq!(
                human_bytes_binary(bytes),
                expected,
                "human_bytes_binary({bytes})"
            );
        }
    }

    #[test]
    fn human_bytes_rounds_exact_ties_to_even() {
        // Deliberate difference: JavaScript's toFixed gives 1.3KB and 1.3KiB.
        assert_eq!(human_bytes(1250.0), "1.2KB");
        assert_eq!(human_bytes_binary(1280.0), "1.2KiB");
    }

    #[test]
    fn human_secs_matches_js() {
        let cases = [
            (0, "", ""),
            (-5, "", ""),
            (1, "1s", "1sec"),
            (59, "59s", "59sec"),
            (60, "1m", "1min"),
            (61, "1m 1s", "1min 1sec"),
            (3599, "59m 59s", "59min 59sec"),
            (3600, "1h", "1hour"),
            (3661, "1h 1m 1s", "1hour 1min 1sec"),
            (86_399, "23h 59m 59s", "23hour 59min 59sec"),
            (86_400, "1D", "1day"),
            (90_061, "1D 1h 1m 1s", "1day 1hour 1min 1sec"),
            (
                2_629_744,
                "1M 30D 10h 29m 4s",
                "1month 30day 10hour 29min 4sec",
            ),
            (2_629_743, "30D 10h 29m 3s", "30day 10hour 29min 3sec"),
            (
                31_535_999,
                "11M 364D 23h 59m 59s",
                "11month 364day 23hour 59min 59sec",
            ),
            (31_536_000, "1Y 11M", "1year 11month"),
            (34_560_000, "1Y 13M 35D", "1year 13month 35day"),
            (
                63_075_661,
                "2Y 23M 1h 1m 1s",
                "2year 23month 1hour 1min 1sec",
            ),
            (
                1_000_000_000,
                "31Y 380M 259D 1h 46m 40s",
                "31year 380month 259day 1hour 46min 40sec",
            ),
        ];
        for (secs, short, long) in cases {
            assert_eq!(human_secs(secs, false), short, "human_secs({secs})");
            assert_eq!(human_secs(secs, true), long, "human_secs({secs}, long)");
        }
    }

    #[test]
    fn human_mins_matches_js() {
        let cases = [
            (0, "0m"),
            (1, "0m"),
            (2, "0m2s"),
            (59, "0m59s"),
            (60, "1m"),
            (61, "1m"),
            (62, "1m2s"),
            (600, "10m"),
            (601, "10m"),
            (602, "10m2s"),
            (3600, "60m"),
            (-30, "-1m"),
            (-61, "-2m"),
            (-90, "-2m"),
        ];
        for (secs, expected) in cases {
            assert_eq!(human_mins(secs), expected, "human_mins({secs})");
        }
    }

    #[test]
    fn human_blocks_matches_js() {
        let cases = [
            (1, "10m", "10min"),
            (2, "20m", "20min"),
            (3, "30m", "30min"),
            (6, "1h", "1hour"),
            (25, "4h 10m", "4hour 10min"),
            (144, "1D", "1day"),
            (1008, "7D", "7day"),
        ];
        for (blocks, short, long) in cases {
            assert_eq!(human_blocks(blocks, false), short, "human_blocks({blocks})");
            assert_eq!(
                human_blocks(blocks, true),
                long,
                "human_blocks({blocks}, long)"
            );
        }
    }

    #[test]
    fn virtual_bytes_matches_js() {
        let cases = [
            (0.0, 0),
            (0.00001, 1),
            (0.0001234, 12),
            (0.00002876, 2),
            (0.001, 100),
            (0.0000099, 0),
            (0.00010001, 10),
            (1.0, 100_000),
        ];
        for (rate, expected) in cases {
            assert_eq!(virtual_bytes(rate), expected, "virtual_bytes({rate})");
        }
    }
}
