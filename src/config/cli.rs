//! Command-line options, ported from `lib/utils/options.js`.
//!
//! Every value option is an `Option` without a clap default: clap fills it from
//! the command line or, failing that, from its environment variable, and the
//! merge in the parent module falls back to the file and then to the defaults.
//! Defaults are therefore written by hand in the help text; a test checks that
//! they match the constants.

use std::path::PathBuf;

use clap::builder::BoolishValueParser;
use clap::{ArgAction, Parser};

/// First lines of `--help` and the whole output of `--version`, as in
/// `lib/utils/print-help.js`.
pub const HEADER: &str = concat!(
    "SatoshiTop v",
    env!("CARGO_PKG_VERSION"),
    " - by st3b1t\n",
    "Bitcoin full node monitoring dashboard for terminal\n",
    "https://github.com/st3b1t/SatoshiTop",
);

/// Options of `satop`. Field names match the original options and the keys of
/// the configuration file.
#[derive(Debug, Default, Clone, PartialEq, Eq, Parser)]
#[command(
    name = "satop",
    // The doc comment above is for Rust readers; the header already describes
    // the program.
    about = None,
    long_about = None,
    help_template = "{before-help}{usage-heading} {usage}\n\n{all-args}{after-help}",
    before_help = HEADER,
    disable_help_flag = true,
    disable_version_flag = true,
    hide_possible_values = true
)]
pub struct Cli {
    /// Specify configuration file. Relative path will be prefixed by $HOME
    /// location (default: .satoprc)
    #[arg(long, env = "CONF", value_name = "FILE")]
    pub conf: Option<PathBuf>,

    /// Send commands to node running on <ip> (default: 127.0.0.1)
    #[arg(long, env = "RPCCONNECT", value_name = "IP")]
    pub rpcconnect: Option<String>,

    /// Timeout in milliseconds during HTTP requests, or 0 for no timeout
    /// (default: 5000)
    #[arg(long, env = "RPCCLIENTTIMEOUT", value_name = "MS")]
    pub rpcclienttimeout: Option<u64>,

    /// Location of the auth cookie. Absolute path
    #[arg(
        long,
        env = "RPCCOOKIEFILE",
        value_name = "FILE",
        hide_env_values = true
    )]
    pub rpccookiefile: Option<PathBuf>,

    /// Username for JSON-RPC connections
    #[arg(long, env = "RPCUSER", value_name = "USER", hide_env_values = true)]
    pub rpcuser: Option<String>,

    /// Password for JSON-RPC connections
    #[arg(
        long,
        env = "RPCPASSWORD",
        value_name = "PASSWORD",
        hide_env_values = true
    )]
    pub rpcpassword: Option<String>,

    /// Connect to JSON-RPC on <port> (default: 8332)
    #[arg(long, env = "RPCPORT", value_name = "PORT", value_parser = clap::value_parser!(u16).range(1..))]
    pub rpcport: Option<u16>,

    /// Delay interval for system pick data, value in ms (default: 3000)
    #[arg(long, env = "INTERVALSYS", value_name = "MS", value_parser = clap::value_parser!(u64).range(1..))]
    pub intervalsys: Option<u64>,

    /// Delay interval for rpc pick data, value in ms (default: 5000)
    #[arg(long, env = "INTERVALRPC", value_name = "MS", value_parser = clap::value_parser!(u64).range(1..))]
    pub intervalrpc: Option<u64>,

    /// Generate new configuration file (.satoprc) in your own $HOME directory,
    /// or at the path given by --conf
    #[arg(long)]
    pub confgen: bool,

    /// Verbose mode on; also accepts --verbose=true|false
    #[arg(
        short = 'v',
        long,
        env = "VERBOSE",
        value_name = "BOOL",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new()
    )]
    pub verbose: Option<bool>,

    /// Print help
    #[arg(short = '?', long, action = ArgAction::Help)]
    pub help: Option<bool>,

    /// Print version
    #[arg(short = 'V', long)]
    pub version: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    use super::*;
    use crate::config::{
        DEFAULT_CONF, DEFAULT_INTERVALRPC, DEFAULT_INTERVALSYS, DEFAULT_RPCCLIENTTIMEOUT,
        DEFAULT_RPCCONNECT, DEFAULT_RPCPORT,
    };

    // Environment variables could leak into these tests through clap's `env`
    // support; the options used here are always given on the command line, or
    // are checked only for the error kind.
    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("satop").chain(args.iter().copied()))
    }

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn value_options_are_read() {
        let cli = parse(&[
            "--conf",
            "my.conf",
            "--rpcconnect",
            "10.0.0.1",
            "--rpcport=18443",
            "--rpcclienttimeout",
            "0",
            "--rpccookiefile",
            "/tmp/.cookie",
            "--rpcuser",
            "u",
            "--rpcpassword",
            "p",
            "--intervalsys",
            "1000",
            "--intervalrpc",
            "2000",
        ])
        .unwrap();
        assert_eq!(cli.conf, Some(PathBuf::from("my.conf")));
        assert_eq!(cli.rpcconnect.as_deref(), Some("10.0.0.1"));
        assert_eq!(cli.rpcport, Some(18443));
        assert_eq!(cli.rpcclienttimeout, Some(0));
        assert_eq!(cli.rpccookiefile, Some(PathBuf::from("/tmp/.cookie")));
        assert_eq!(cli.rpcuser.as_deref(), Some("u"));
        assert_eq!(cli.rpcpassword.as_deref(), Some("p"));
        assert_eq!(cli.intervalsys, Some(1000));
        assert_eq!(cli.intervalrpc, Some(2000));
    }

    #[test]
    fn verbose_accepts_an_optional_boolean() {
        assert_eq!(parse(&["-v"]).unwrap().verbose, Some(true));
        assert_eq!(parse(&["--verbose"]).unwrap().verbose, Some(true));
        assert_eq!(parse(&["--verbose=true"]).unwrap().verbose, Some(true));
        assert_eq!(parse(&["--verbose=0"]).unwrap().verbose, Some(false));
        assert_eq!(parse(&["--verbose=off"]).unwrap().verbose, Some(false));
        assert!(parse(&["--verbose=maybe"]).is_err());
    }

    #[test]
    fn short_flags_match_the_original() {
        assert_eq!(parse(&["-?"]).unwrap_err().kind(), ErrorKind::DisplayHelp);
        assert_eq!(
            parse(&["--help"]).unwrap_err().kind(),
            ErrorKind::DisplayHelp
        );
        assert!(parse(&["-V"]).unwrap().version);
        assert!(parse(&["--confgen"]).unwrap().confgen);
        // `-h` is not an option of the original.
        assert!(parse(&["-h"]).is_err());
    }

    #[test]
    fn unknown_options_and_invalid_values_are_rejected() {
        assert_eq!(
            parse(&["--nope"]).unwrap_err().kind(),
            ErrorKind::UnknownArgument
        );
        assert!(parse(&["extra"]).is_err());
        assert!(parse(&["--rpcport", "0"]).is_err());
        assert!(parse(&["--rpcport", "70000"]).is_err());
        assert!(parse(&["--intervalsys", "0"]).is_err());
        assert!(parse(&["--rpcclienttimeout", "-1"]).is_err());
    }

    #[test]
    fn help_lists_options_defaults_and_environment() {
        let help = Cli::command().render_help().to_string();
        assert!(help.starts_with(HEADER), "{help}");
        for option in [
            "--conf",
            "--rpcconnect",
            "--rpcclienttimeout",
            "--rpccookiefile",
            "--rpcuser",
            "--rpcpassword",
            "--rpcport",
            "--intervalsys",
            "--intervalrpc",
            "--confgen",
            "-v, --verbose",
            "-?, --help",
            "-V, --version",
        ] {
            assert!(help.contains(option), "missing {option}");
        }
        for env in [
            "CONF",
            "RPCCONNECT",
            "RPCCLIENTTIMEOUT",
            "RPCCOOKIEFILE",
            "RPCUSER",
            "RPCPASSWORD",
            "RPCPORT",
            "INTERVALSYS",
            "INTERVALRPC",
            "VERBOSE",
        ] {
            assert!(help.contains(&format!("[env: {env}")), "missing env {env}");
        }
        for default in [
            DEFAULT_CONF.to_owned(),
            DEFAULT_RPCCONNECT.to_owned(),
            DEFAULT_RPCCLIENTTIMEOUT.to_string(),
            DEFAULT_RPCPORT.to_string(),
            DEFAULT_INTERVALSYS.to_string(),
            DEFAULT_INTERVALRPC.to_string(),
        ] {
            assert!(
                help.contains(&format!("(default: {default})")),
                "missing default {default}"
            );
        }
    }
}
