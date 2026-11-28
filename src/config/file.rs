//! Reading of the `.satoprc` configuration file, ported from the `ini` part of
//! `lib/utils/conf-parser.js`.
//!
//! The file is parsed with `rust-ini`, with quoting and escaping turned off, and
//! each value is then cleaned up the way the npm `ini` package does it: matching
//! quotes are stripped, otherwise the value ends at the first unescaped `;` or
//! `#` (inline comment).

use std::path::{Path, PathBuf};

use ini::{Ini, ParseOption};

use super::{ConfigError, parse_bool, parse_interval, parse_port, parse_timeout};

/// Values read from the configuration file. Every field is optional: missing
/// keys fall back to environment variables, then to defaults.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileValues {
    pub rpcconnect: Option<String>,
    pub rpcport: Option<u16>,
    pub rpcclienttimeout: Option<u64>,
    pub rpccookiefile: Option<PathBuf>,
    pub rpcuser: Option<String>,
    pub rpcpassword: Option<String>,
    pub intervalsys: Option<u64>,
    pub intervalrpc: Option<u64>,
    pub verbose: Option<bool>,
}

/// Loads the configuration file at `path`. A missing file is not an error and
/// gives `None`, as in the original.
pub fn load(path: &Path) -> Result<Option<FileValues>, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text, path).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::new(format!(
            "cannot read {}: {e}",
            path.display()
        ))),
    }
}

/// Parses the text of a configuration file; `path` is only used in error
/// messages. Only keys of the general section (before any `[section]`) are
/// read; unknown keys are ignored.
pub fn parse(text: &str, path: &Path) -> Result<FileValues, ConfigError> {
    let options = ParseOption {
        enabled_quote: false,
        enabled_escape: false,
        ..ParseOption::default()
    };
    let ini = Ini::load_from_str_opt(text, options)
        .map_err(|e| ConfigError::new(format!("invalid {}: {e}", path.display())))?;
    let section = ini.general_section();
    // Like npm `ini`, a repeated key keeps its last value.
    let get = |key: &str| section.get_all(key).next_back().map(clean_value);
    let source = || format!("in {}", path.display());

    Ok(FileValues {
        rpcconnect: get("rpcconnect"),
        rpcport: get("rpcport")
            .map(|v| parse_port(&v, &source()))
            .transpose()?,
        rpcclienttimeout: get("rpcclienttimeout")
            .map(|v| parse_timeout(&v, &source()))
            .transpose()?,
        rpccookiefile: get("rpccookiefile").map(PathBuf::from),
        rpcuser: get("rpcuser"),
        rpcpassword: get("rpcpassword"),
        intervalsys: get("intervalsys")
            .map(|v| parse_interval("intervalsys", &v, &source()))
            .transpose()?,
        intervalrpc: get("intervalrpc")
            .map(|v| parse_interval("intervalrpc", &v, &source()))
            .transpose()?,
        verbose: get("verbose")
            .map(|v| parse_bool("verbose", &v, &source()))
            .transpose()?,
    })
}

/// Cleans a raw value like npm `ini` (`unsafe()` in its source).
fn clean_value(raw: &str) -> String {
    let value = raw.trim();
    let quoted = |q: char| value.len() >= 2 && value.starts_with(q) && value.ends_with(q);
    if quoted('"') {
        // npm `ini` runs JSON.parse on double-quoted values.
        return serde_json::from_str(value)
            .unwrap_or_else(|_| value[1..value.len() - 1].to_owned());
    }
    if quoted('\'') {
        return value[1..value.len() - 1].to_owned();
    }
    let mut out = String::new();
    let mut escaped = false;
    for c in value.chars() {
        if escaped {
            if !matches!(c, '\\' | ';' | '#') {
                out.push('\\');
            }
            out.push(c);
            escaped = false;
        } else if c == ';' || c == '#' {
            break;
        } else if c == '\\' {
            escaped = true;
        } else {
            out.push(c);
        }
    }
    if escaped {
        out.push('\\');
    }
    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("dot.satoprc.example");

    fn parse_ok(text: &str) -> FileValues {
        parse(text, Path::new("test.satoprc")).expect("valid file")
    }

    #[test]
    fn the_shipped_example_is_all_comments() {
        assert_eq!(parse_ok(EXAMPLE), FileValues::default());
    }

    #[test]
    fn the_example_uncommented_gives_its_values() {
        let text: String = EXAMPLE
            .lines()
            .filter(|l| !l.starts_with("##") && l.len() > 1)
            .map(|l| format!("{}\n", l.trim_start_matches('#')))
            .collect();
        let values = parse_ok(&text);
        assert_eq!(values.rpcconnect.as_deref(), Some("127.0.0.1"));
        assert_eq!(values.rpcport, Some(8332));
        assert_eq!(values.rpcclienttimeout, Some(5000));
        assert_eq!(
            values.rpccookiefile,
            Some(PathBuf::from("/home/bitcoin/.bitcoin/.cookie"))
        );
        assert_eq!(values.rpcuser.as_deref(), Some("<username>"));
        assert_eq!(values.rpcpassword.as_deref(), Some("<password>"));
    }

    #[test]
    fn values_are_cleaned_like_npm_ini() {
        // Expected values from `ini.parse` in node.
        let values = parse_ok(
            "rpcuser = alice\n\
             rpcpassword = \"p;a#ss\"\n\
             rpcconnect = 10.0.0.1 ; comment\n\
             rpcport=8333 # c\n\
             verbose = true\n",
        );
        assert_eq!(values.rpcuser.as_deref(), Some("alice"));
        assert_eq!(values.rpcpassword.as_deref(), Some("p;a#ss"));
        assert_eq!(values.rpcconnect.as_deref(), Some("10.0.0.1"));
        assert_eq!(values.rpcport, Some(8333));
        assert_eq!(values.verbose, Some(true));
    }

    #[test]
    fn clean_value_matches_npm_ini() {
        assert_eq!(clean_value("'single'"), "single");
        assert_eq!(clean_value(r"a\b"), r"a\b");
        assert_eq!(clean_value(r"a\;b"), "a;b");
        assert_eq!(clean_value(r#""a\"b""#), "a\"b");
        assert_eq!(clean_value(""), "");
    }

    #[test]
    fn repeated_keys_keep_the_last_value() {
        assert_eq!(parse_ok("rpcport=1\nrpcport=2\n").rpcport, Some(2));
    }

    #[test]
    fn keys_inside_sections_are_ignored() {
        let values = parse_ok("rpcport=8333\n[main]\nintervalsys=1000\n");
        assert_eq!(values.rpcport, Some(8333));
        assert_eq!(values.intervalsys, None);
    }

    #[test]
    fn invalid_values_name_the_file() {
        let err = parse("rpcport=abc\n", Path::new("/home/u/.satoprc")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid rpcport 'abc' in /home/u/.satoprc: expected a port number (1-65535)"
        );
        assert!(parse("intervalsys=0\n", Path::new("f")).is_err());
        assert!(parse("verbose=maybe\n", Path::new("f")).is_err());
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let path = Path::new("/nonexistent/dir/.satoprc");
        assert_eq!(load(path).unwrap(), None);
    }
}
