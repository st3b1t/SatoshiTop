//! Configuration of `satop`, ported from `lib/utils/conf-parser.js` and
//! `lib/utils/conf-gen.js`.
//!
//! Precedence, lowest to highest: defaults < configuration file < environment
//! variables < command line. Environment and command line are merged by clap
//! (see [`cli`]); this module adds the file and the defaults, validates the
//! values and generates the configuration file.

pub mod auth;
pub mod cli;
pub mod file;

use std::fmt;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Serialize, Serializer};

pub use cli::Cli;
pub use file::FileValues;

/// Default configuration file, relative to `$HOME`.
pub const DEFAULT_CONF: &str = ".satoprc";
/// Default address of the Bitcoin node.
pub const DEFAULT_RPCCONNECT: &str = "127.0.0.1";
/// Default JSON-RPC port of the Bitcoin node (mainnet).
pub const DEFAULT_RPCPORT: u16 = 8332;
/// Default HTTP timeout, in milliseconds.
pub const DEFAULT_RPCCLIENTTIMEOUT: u64 = 5000;
/// Default refresh interval of the system panels, in milliseconds.
pub const DEFAULT_INTERVALSYS: u64 = 3000;
/// Default refresh interval of the Bitcoin panels, in milliseconds.
pub const DEFAULT_INTERVALRPC: u64 = 5000;

/// Example configuration written by `--confgen`, copied from the original
/// `dot.satoprc.example`.
pub const EXAMPLE_CONF: &str = include_str!("dot.satoprc.example");

/// Final configuration, after merging every source.
///
/// Field names match the original options; it serializes to the JSON printed
/// by `--verbose`, with the password masked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Config {
    /// Absolute path of the configuration file (it may not exist).
    pub conf: PathBuf,
    pub rpcconnect: String,
    pub rpcport: u16,
    /// HTTP timeout in milliseconds; 0 means no timeout.
    pub rpcclienttimeout: u64,
    pub rpccookiefile: Option<PathBuf>,
    pub rpcuser: Option<String>,
    #[serde(serialize_with = "mask_secret")]
    pub rpcpassword: Option<String>,
    /// Refresh interval of the system panels, in milliseconds (never 0).
    pub intervalsys: u64,
    /// Refresh interval of the Bitcoin panels, in milliseconds (never 0).
    pub intervalrpc: u64,
    pub verbose: bool,
}

impl Config {
    /// Builds the configuration from the parsed command line (which already
    /// includes environment variables) and the configuration file, if any.
    pub fn merge(cli: &Cli, file: Option<&FileValues>, conf: PathBuf) -> Self {
        let file = file.cloned().unwrap_or_default();
        Self {
            conf,
            rpcconnect: cli
                .rpcconnect
                .clone()
                .or(file.rpcconnect)
                .unwrap_or_else(|| DEFAULT_RPCCONNECT.to_owned()),
            rpcport: cli.rpcport.or(file.rpcport).unwrap_or(DEFAULT_RPCPORT),
            rpcclienttimeout: cli
                .rpcclienttimeout
                .or(file.rpcclienttimeout)
                .unwrap_or(DEFAULT_RPCCLIENTTIMEOUT),
            rpccookiefile: cli.rpccookiefile.clone().or(file.rpccookiefile),
            rpcuser: cli.rpcuser.clone().or(file.rpcuser),
            rpcpassword: cli.rpcpassword.clone().or(file.rpcpassword),
            intervalsys: cli
                .intervalsys
                .or(file.intervalsys)
                .unwrap_or(DEFAULT_INTERVALSYS),
            intervalrpc: cli
                .intervalrpc
                .or(file.intervalrpc)
                .unwrap_or(DEFAULT_INTERVALRPC),
            verbose: cli.verbose.or(file.verbose).unwrap_or(false),
        }
    }

    /// Resolves the configuration file against `home`, reads it if it exists
    /// and merges it with the command line.
    pub fn load(cli: &Cli, home: &Path) -> Result<Self, ConfigError> {
        let conf = conf_path(cli, home);
        let file = file::load(&conf)?;
        Ok(Self::merge(cli, file.as_ref(), conf))
    }

    /// HTTP timeout, `None` when disabled.
    pub fn rpc_timeout(&self) -> Option<Duration> {
        (self.rpcclienttimeout > 0).then(|| Duration::from_millis(self.rpcclienttimeout))
    }

    /// Refresh interval of the system panels.
    pub fn sys_interval(&self) -> Duration {
        Duration::from_millis(self.intervalsys)
    }

    /// Refresh interval of the Bitcoin panels.
    pub fn rpc_interval(&self) -> Duration {
        Duration::from_millis(self.intervalrpc)
    }

    /// JSON printed on stderr by `--verbose`, shaped like the original
    /// (`{"initOpts": {...}}`).
    pub fn verbose_json(&self) -> String {
        // A struct, not `json!`, so fields keep their declaration order.
        #[derive(Serialize)]
        struct Wrapped<'a> {
            #[serde(rename = "initOpts")]
            init_opts: &'a Config,
        }
        serde_json::to_string_pretty(&Wrapped { init_opts: self })
            .expect("config is always serializable")
    }
}

/// Path of the configuration file: `--conf`/`CONF` or the default, relative
/// to `home` unless absolute.
pub fn conf_path(cli: &Cli, home: &Path) -> PathBuf {
    let conf = cli
        .conf
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONF));
    home.join(conf)
}

/// Home directory, from `$HOME`.
pub fn home_dir() -> Result<PathBuf, ConfigError> {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| ConfigError::new("cannot find the home directory: $HOME is not set"))
}

/// Reads a boolean flag from an environment variable value: unset, empty,
/// `0`, `false`, `no` and `off` (any case) are false, anything else is true.
///
/// Used for `FAKERPC` and `FAKESYS`; the original treats any non-empty value,
/// even `0`, as true.
pub fn env_flag(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "" | "0" | "false" | "no" | "off"
        )
    })
}

/// [`env_flag`] applied to the process environment variable `name`.
pub fn env_flag_var(name: &str) -> bool {
    env_flag(std::env::var(name).ok().as_deref())
}

/// Writes the example configuration to `path`, as `--confgen` does.
///
/// If the file exists, asks on `out` whether to overwrite it and reads the
/// answer from `input`; only `y` overwrites. The file is created with mode
/// 0600. Returns whether the file was written.
pub fn confgen(
    path: &Path,
    input: &mut impl BufRead,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<bool, ConfigError> {
    use std::os::unix::fs::PermissionsExt;

    let shown = path.display();
    let io_error =
        |what: &str, e: std::io::Error| ConfigError::new(format!("{what} '{shown}': {e}"));

    if path.exists() {
        // Messages as in the original.
        let _ = writeln!(err, "File '{shown}' just exists!");
        let _ = writeln!(out, "Do you want to overwrite it? [y/n]");
        let _ = out.flush();
        let mut answer = String::new();
        input
            .read_line(&mut answer)
            .map_err(|e| io_error("Error reading the answer for", e))?;
        if answer.trim_end_matches(['\r', '\n']) != "y" {
            return Ok(false);
        }
    }

    std::fs::write(path, EXAMPLE_CONF).map_err(|e| io_error("Error in config generation", e))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| io_error("Error in chmod config", e))?;
    let _ = writeln!(out, "Config file generated in '{shown}'");
    Ok(true)
}

/// A configuration problem: unreadable file or invalid value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(String);

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

fn invalid(key: &str, value: &str, source: &str, expected: &str) -> ConfigError {
    ConfigError::new(format!("invalid {key} '{value}' {source}: {expected}"))
}

/// Parses a port number (1-65535); `source` says where the value comes from.
pub(crate) fn parse_port(value: &str, source: &str) -> Result<u16, ConfigError> {
    value
        .parse::<u16>()
        .ok()
        .filter(|&p| p > 0)
        .ok_or_else(|| invalid("rpcport", value, source, "expected a port number (1-65535)"))
}

/// Parses a timeout in milliseconds (0 disables it).
pub(crate) fn parse_timeout(value: &str, source: &str) -> Result<u64, ConfigError> {
    value.parse::<u64>().map_err(|_| {
        invalid(
            "rpcclienttimeout",
            value,
            source,
            "expected milliseconds (0 for no timeout)",
        )
    })
}

/// Parses a refresh interval in milliseconds (at least 1).
pub(crate) fn parse_interval(key: &str, value: &str, source: &str) -> Result<u64, ConfigError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|&ms| ms > 0)
        .ok_or_else(|| invalid(key, value, source, "expected milliseconds greater than 0"))
}

/// Parses a boolean with the same words accepted by clap for `--verbose`.
pub(crate) fn parse_bool(key: &str, value: &str, source: &str) -> Result<bool, ConfigError> {
    match value.to_ascii_lowercase().as_str() {
        "y" | "yes" | "t" | "true" | "on" | "1" => Ok(true),
        "n" | "no" | "f" | "false" | "off" | "0" => Ok(false),
        _ => Err(invalid(key, value, source, "expected true or false")),
    }
}

fn mask_secret<S: Serializer>(secret: &Option<String>, s: S) -> Result<S::Ok, S::Error> {
    match secret {
        Some(_) => s.serialize_str("***"),
        None => s.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    fn home() -> PathBuf {
        PathBuf::from("/home/u")
    }

    fn defaults() -> Config {
        Config::merge(&Cli::default(), None, home().join(DEFAULT_CONF))
    }

    #[test]
    fn defaults_match_the_original() {
        let config = defaults();
        assert_eq!(config.conf, PathBuf::from("/home/u/.satoprc"));
        assert_eq!(config.rpcconnect, "127.0.0.1");
        assert_eq!(config.rpcport, 8332);
        assert_eq!(config.rpcclienttimeout, 5000);
        assert_eq!(config.intervalsys, 3000);
        assert_eq!(config.intervalrpc, 5000);
        assert_eq!(config.rpccookiefile, None);
        assert_eq!(config.rpcuser, None);
        assert_eq!(config.rpcpassword, None);
        assert!(!config.verbose);
    }

    #[test]
    fn file_overrides_defaults() {
        let file = FileValues {
            rpcconnect: Some("10.0.0.1".into()),
            rpcport: Some(18443),
            intervalsys: Some(1000),
            verbose: Some(true),
            ..FileValues::default()
        };
        let config = Config::merge(&Cli::default(), Some(&file), home());
        assert_eq!(config.rpcconnect, "10.0.0.1");
        assert_eq!(config.rpcport, 18443);
        assert_eq!(config.intervalsys, 1000);
        assert_eq!(config.intervalrpc, DEFAULT_INTERVALRPC);
        assert!(config.verbose);
    }

    #[test]
    fn command_line_and_environment_override_the_file() {
        // Clap has already merged the environment into `Cli`.
        let file = FileValues {
            rpcport: Some(18443),
            rpcuser: Some("file-user".into()),
            verbose: Some(true),
            ..FileValues::default()
        };
        let cli = Cli {
            rpcport: Some(8333),
            rpcuser: Some("cli-user".into()),
            verbose: Some(false),
            ..Cli::default()
        };
        let config = Config::merge(&cli, Some(&file), home());
        assert_eq!(config.rpcport, 8333);
        assert_eq!(config.rpcuser.as_deref(), Some("cli-user"));
        assert!(!config.verbose);
    }

    #[test]
    fn conf_path_is_relative_to_home_unless_absolute() {
        assert_eq!(
            conf_path(&Cli::default(), &home()),
            PathBuf::from("/home/u/.satoprc")
        );
        let relative = Cli {
            conf: Some("cfg/satop.ini".into()),
            ..Cli::default()
        };
        assert_eq!(
            conf_path(&relative, &home()),
            PathBuf::from("/home/u/cfg/satop.ini")
        );
        let absolute = Cli {
            conf: Some("/etc/satoprc".into()),
            ..Cli::default()
        };
        assert_eq!(conf_path(&absolute, &home()), PathBuf::from("/etc/satoprc"));
    }

    #[test]
    fn load_reads_the_file_from_home() {
        let dir = TempDir::new("load");
        std::fs::write(dir.path().join(".satoprc"), "rpcport = 18332\n").unwrap();
        let config = Config::load(&Cli::default(), dir.path()).unwrap();
        assert_eq!(config.rpcport, 18332);
        assert_eq!(config.conf, dir.path().join(".satoprc"));
    }

    #[test]
    fn load_without_file_gives_defaults() {
        let dir = TempDir::new("nofile");
        let config = Config::load(&Cli::default(), dir.path()).unwrap();
        assert_eq!(config.rpcport, DEFAULT_RPCPORT);
    }

    #[test]
    fn durations() {
        let mut config = defaults();
        assert_eq!(config.rpc_timeout(), Some(Duration::from_secs(5)));
        assert_eq!(config.sys_interval(), Duration::from_secs(3));
        assert_eq!(config.rpc_interval(), Duration::from_secs(5));
        config.rpcclienttimeout = 0;
        assert_eq!(config.rpc_timeout(), None);
    }

    #[test]
    fn verbose_json_masks_the_password() {
        let mut config = defaults();
        config.rpcpassword = Some("secret".into());
        let json = config.verbose_json();
        assert!(json.contains("\"initOpts\""), "{json}");
        assert!(json.contains("\"rpcpassword\": \"***\""), "{json}");
        assert!(!json.contains("secret"), "{json}");
        assert!(defaults().verbose_json().contains("\"rpcpassword\": null"));
    }

    #[test]
    fn env_flag_values() {
        assert!(!env_flag(None));
        for off in ["", "0", "false", "FALSE", "no", "off", " 0 "] {
            assert!(!env_flag(Some(off)), "{off:?}");
        }
        for on in ["1", "true", "yes", "on", "anything"] {
            assert!(env_flag(Some(on)), "{on:?}");
        }
    }

    #[test]
    fn parsers_reject_invalid_values() {
        assert_eq!(parse_port("8332", "x"), Ok(8332));
        assert!(parse_port("0", "x").is_err());
        assert!(parse_port("65536", "x").is_err());
        assert_eq!(parse_timeout("0", "x"), Ok(0));
        assert!(parse_timeout("1.5", "x").is_err());
        assert!(parse_interval("intervalsys", "0", "x").is_err());
        assert_eq!(parse_bool("verbose", "On", "x"), Ok(true));
    }

    #[test]
    fn confgen_writes_the_example_with_mode_0600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new("confgen");
        let path = dir.path().join(".satoprc");
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let written = confgen(&path, &mut &b""[..], &mut out, &mut err).unwrap();
        assert!(written);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), EXAMPLE_CONF);
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        let out = String::from_utf8(out).unwrap();
        assert_eq!(
            out,
            format!("Config file generated in '{}'\n", path.display())
        );
        assert!(err.is_empty());
    }

    #[test]
    fn confgen_asks_before_overwriting() {
        let dir = TempDir::new("overwrite");
        let path = dir.path().join(".satoprc");
        std::fs::write(&path, "rpcport=1\n").unwrap();

        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert!(!confgen(&path, &mut &b"n\n"[..], &mut out, &mut err).unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "rpcport=1\n");
        let err = String::from_utf8(err).unwrap();
        assert_eq!(err, format!("File '{}' just exists!\n", path.display()));
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "Do you want to overwrite it? [y/n]\n"
        );

        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert!(confgen(&path, &mut &b"y\n"[..], &mut out, &mut err).unwrap());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), EXAMPLE_CONF);
    }
}
